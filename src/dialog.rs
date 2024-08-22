use unreal_asset::exports::ExportBaseTrait;

use super::*;

pub fn open(
    trigger: Trigger<triggers::Open>,
    mut commands: Commands,
    actors: Query<Entity, With<actor::Actor>>,
    mut notif: EventWriter<Notif>,
    appdata: ResMut<AppData>,
    mut client: ResMut<Client>,
    mut map: NonSendMut<Map>,
    mut registry: ResMut<Registry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<unlit::Unlit>>,
    mut images: ResMut<Assets<Image>>,
    paks: Res<Paks>,
    consts: Res<Constants>,
) {
    let Some(path) = trigger.event().0.clone().or_else(|| {
        rfd::FileDialog::new()
            .set_title("open map")
            .add_filter("maps", &["umap"])
            .pick_file()
    }) else {
        return;
    };
    let asset = match asset::open(&path, appdata.version()) {
        Ok(asset) => asset,
        Err(e) => {
            notif.send(Notif {
                message: e.to_string(),
                kind: Error,
            });
            return;
        }
    };
    for actor in actors.iter() {
        commands.entity(actor).despawn_recursive();
    }
    let cache = config()
        .filter(|_| appdata.cache)
        .map(|path| path.join("cache"));
    let version = appdata.version();
    let mut batch = std::collections::BTreeMap::<_, Vec<_>>::new();
    let mut export_names: Vec<_> = asset
        .asset_data
        .exports
        .iter()
        .map(|ex| ex.get_base_export().object_name.get_owned_content())
        .collect();
    for i in actor::get_actors(&asset) {
        let (path, actor) = match actor::Actor::new(&asset, i) {
            Ok(actor) => actor,
            Err(e) => {
                notif.send(Notif {
                    message: e.to_string(),
                    kind: Warning,
                });
                continue;
            }
        };
        export_names[i.index as usize - 1] = actor.name.clone();
        match batch.get_mut(&path) {
            Some(vec) => vec.push(actor),
            None => {
                batch.insert(path, vec![actor]);
            }
        }
    }
    let keys = batch.keys().flatten().cloned().collect::<Vec<_>>();
    std::thread::scope(|s| {
        let threads: Vec<_> = keys
            .into_iter()
            .map(|path| {
                s.spawn(|| {
                    // capture path
                    let path = path;
                    match asset::get(&paks, cache.as_deref(), &path, version, |asset, _| {
                        Ok(extras::get_mesh_info(asset)?)
                    }) {
                        Some((positions, indices, uvs, mats, _mat_data)) => Ok((
                            path,
                            Mesh::new(
                                bevy::render::render_resource::PrimitiveTopology::TriangleList,
                                default(),
                            )
                            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                            .with_inserted_attribute(
                                Mesh::ATTRIBUTE_UV_0,
                                uvs.into_iter().map(|uv| uv[0]).collect::<Vec<_>>(),
                            )
                            .with_inserted_indices(bevy::render::mesh::Indices::U32(indices)),
                            mats,
                        )),
                        None => Err(path),
                    }
                })
            })
            .collect();
        for thread in threads {
            match thread.join() {
                Ok(Ok((path, mesh, mats))) => {
                    // hard to multithread material loading since the material might not parse
                    let mut tex = None;
                    if appdata.textures {
                        'outer: for mat in mats {
                            if registry.mats.contains_key(&mat) {
                                break;
                            }
                            let Some(paths) =
                                asset::get(&paks, cache.as_deref(), &mat, version, |mat, _| {
                                    Ok(extras::get_tex_paths(mat))
                                })
                            else {
                                continue;
                            };
                            for path in paths {
                                if let Some((false, width, height, data)) = asset::get(
                                    &paks,
                                    cache.as_deref(),
                                    &path,
                                    version,
                                    |tex, bulk| Ok(extras::get_tex_info(tex, bulk)?),
                                ) {
                                    tex = Some((path, Image {
                                        data,
                                        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
                                            label: None,
                                            size: bevy::render::render_resource::Extent3d {
                                                width,
                                                height,
                                                depth_or_array_layers: 1,
                                            },
                                            mip_level_count: 1,
                                            sample_count: 1,
                                            dimension: bevy::render::render_resource::TextureDimension::D2,
                                            format: bevy::render::render_resource::TextureFormat::Bgra8Unorm,
                                            usage:
                                                bevy::render::render_resource::TextureUsages::TEXTURE_BINDING,
                                            view_formats: &[
                                                bevy::render::render_resource::TextureFormat::Bgra8Unorm,
                                            ],
                                        },
                                        sampler: bevy::render::texture::ImageSampler::Descriptor(
                                            bevy::render::texture::ImageSamplerDescriptor {
                                                address_mode_u: bevy::render::texture::ImageAddressMode::Repeat,
                                                address_mode_v: bevy::render::texture::ImageAddressMode::Repeat,
                                                address_mode_w: bevy::render::texture::ImageAddressMode::Repeat,
                                                ..default()
                                            },
                                        ),
                                        ..default()
                                    }));
                                    break 'outer;
                                }
                            }
                        }
                    }
                    registry.meshes.insert(
                        path.clone(),
                        (meshes.add(mesh), tex.as_ref().map(|(path, _)| path.clone())),
                    );
                    if let Some((path, tex)) = tex {
                        registry.mats.insert(
                            path,
                            materials.add(unlit::Unlit {
                                texture: images.add(tex),
                            }),
                        );
                    }
                }
                Ok(Err(path)) => {
                    notif.send(Notif {
                        message: format!("couldn't find the mesh at {path}"),
                        kind: egui_notify::ToastLevel::Warning,
                    });
                    batch.remove(&Some(path));
                }
                Err(_) => continue,
            }
        }
    });
    for (path, actors) in batch {
        match path {
            Some(ref path) => {
                let (mesh, material) = &registry.meshes[path];
                for actor in actors {
                    commands.spawn((
                        MaterialMeshBundle {
                            mesh: mesh.clone_weak(),
                            material: material
                                .as_ref()
                                .map(|mat| registry.mats[mat].clone_weak())
                                .unwrap_or(consts.grid.clone_weak()),
                            transform: actor.transform(&asset),
                            ..default()
                        },
                        bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                        actor,
                    ));
                }
            }
            None => {
                for actor in actors {
                    commands
                        .spawn((
                            MaterialMeshBundle {
                                mesh: consts.cube.clone_weak(),
                                material: consts.unselected.clone_weak(),
                                transform: actor.transform(&asset),
                                ..default()
                            },
                            bevy::pbr::wireframe::NoWireframe,
                            actor,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                consts.bounds.clone_weak(),
                                SpatialBundle {
                                    visibility: Visibility::Hidden,
                                    ..default()
                                },
                                bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                            ));
                        });
                }
                continue;
            }
        }
    }
    let import_names = asset
        .imports
        .iter()
        .map(|ex| ex.object_name.get_owned_content())
        .collect();
    map.0 = Some((asset, path.clone(), export_names, import_names));
    notif.send(Notif {
        message: "map opened".into(),
        kind: Success,
    });
    use discord_rich_presence::DiscordIpc;
    if let (Some(client), Some(name)) = (client.0.as_mut(), path.to_str()) {
        let _ = client.set_activity(
            activity()
                .details("currently editing:")
                .state(name.split('\\').last().unwrap_or_default()),
        );
    }
}

pub fn save_as(
    trigger: Trigger<triggers::SaveAs>,
    mut notif: EventWriter<Notif>,
    appdata: Res<AppData>,
    mut map: NonSendMut<Map>,
) {
    let Some((map, path, ..)) = &mut map.0 else {
        notif.send(Notif {
            message: "no map to save".into(),
            kind: Error,
        });
        return;
    };
    if trigger.event().0 {
        if let Some(new) = rfd::FileDialog::new()
            .set_title("save map as")
            .add_filter("maps", &["umap"])
            .save_file()
        {
            *path = new;
        }
    }
    match asset::save(map, path) {
        Ok(_) => {
            // literally no idea why std::process::Command doesn't work
            #[cfg(target_os = "windows")]
            const PATH: &str = "./script.bat";
            #[cfg(not(target_os = "windows"))]
            const PATH: &str = "./script.sh";
            for line in appdata.script.lines() {
                if let Err(e) = std::fs::write(PATH, line) {
                    notif.send(Notif {
                        message: format!("failed to make save script: {e}"),
                        kind: Error,
                    });
                }
                match std::process::Command::new(PATH)
                    .stdout(std::process::Stdio::piped())
                    .output()
                {
                    Ok(out) => notif.send(Notif {
                        message: String::from_utf8(out.stdout).unwrap_or_default(),
                        kind: Success,
                    }),
                    Err(e) => notif.send(Notif {
                        message: format!("failed to run save script: {e}"),
                        kind: Error,
                    }),
                };
            }
            if !appdata.script.is_empty() {
                if let Err(e) = std::fs::remove_file(PATH) {
                    notif.send(Notif {
                        message: format!("failed to remove save script: {e}"),
                        kind: Error,
                    });
                }
            }
            notif.send(Notif {
                message: "map saved".into(),
                kind: Success,
            });
        }
        Err(e) => {
            notif.send(Notif {
                message: e.to_string(),
                kind: Error,
            });
        }
    }
}

pub fn add_pak(_: Trigger<triggers::AddPak>, mut commands: Commands, mut appdata: ResMut<AppData>) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("add pak folder")
        .pick_folder()
        .and_then(|path| path.to_str().map(str::to_string))
    {
        appdata.pak = Some(appdata.paks.len());
        appdata.paks.push((path, String::new()));
        commands.trigger(triggers::LoadPaks);
    }
}

pub fn transplant(
    _: Trigger<triggers::Transplant>,
    mut notif: EventWriter<Notif>,
    appdata: ResMut<AppData>,
    map: NonSend<Map>,
    mut transplant: NonSendMut<Transplant>,
) {
    if map.0.is_none() {
        notif.send(Notif {
            message: "no map to transplant into".into(),
            kind: Error,
        });
        return;
    };
    let Some(path) = rfd::FileDialog::new()
        .set_title("open map")
        .add_filter("maps", &["umap"])
        .pick_file()
    else {
        return;
    };
    match asset::open(path, appdata.version()) {
        Ok(donor) => {
            // no need for verbose warnings here
            let actors: Vec<_> = actor::get_actors(&donor)
                .into_iter()
                .filter_map(|index| {
                    actor::Actor::new(&donor, index)
                        .ok()
                        .map(|(_, actor)| actor)
                })
                .collect();
            let selected = Vec::with_capacity(actors.len());
            transplant.0 = Some((donor, actors, selected));
        }
        Err(e) => {
            notif.send(Notif {
                message: e.to_string(),
                kind: Error,
            });
        }
    }
}
