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
    let key = match hex::decode(appdata.aes.trim_start_matches("0x")) {
        Ok(key) if !appdata.aes.is_empty() => Some(key),
        Ok(_) => None,
        Err(_) => {
            notif.send(Notif {
                message: "aes key is invalid hex".into(),
                kind: Warning,
            });
            None
        }
    };
    #[cfg(target_os = "windows")]
    #[link(name = "oo2core_win64", kind = "static")]
    extern "C" {
        fn OodleLZ_Decompress(
            compBuf: *const u8,
            compBufSize: usize,
            rawBuf: *mut u8,
            rawLen: usize,
            fuzzSafe: u32,
            checkCRC: u32,
            verbosity: u32,
            decBufBase: u64,
            decBufSize: usize,
            fpCallback: u64,
            callbackUserData: u64,
            decoderMemory: *mut u8,
            decoderMemorySize: usize,
            threadPhase: u32,
        ) -> i32;
    }
    let mut paks: Vec<_> = appdata
        .paks
        .iter()
        .filter_map(|dir| std::fs::read_dir(dir).ok())
        .flatten()
        .filter_map(Result::ok)
        .map(|dir| dir.path())
        .filter_map(|path| {
            use aes::cipher::KeyInit;
            let mut pak_file = std::io::BufReader::new(std::fs::File::open(path).ok()?);
            let mut pak = repak::PakBuilder::new();
            if let Some(key) = key
                .as_deref()
                .and_then(|bytes| aes::Aes256::new_from_slice(bytes).ok())
            {
                pak = pak.key(key);
            }
            #[cfg(target_os = "windows")]
            {
                pak = pak.oodle(|| {
                    Ok(|comp_buf, raw_buf| unsafe {
                        OodleLZ_Decompress(
                            comp_buf.as_ptr(),
                            comp_buf.len(),
                            raw_buf.as_mut_ptr(),
                            raw_buf.len(),
                            1,
                            1,
                            0,
                            0,
                            0,
                            0,
                            0,
                            std::ptr::null_mut(),
                            0,
                            3,
                        )
                    })
                });
            }
            let pak = pak.reader(&mut pak_file).ok()?;
            Some((pak_file, pak))
        })
        .collect();
    let cache = config()
        .filter(|_| appdata.cache)
        .map(|path| path.join("cache"));
    let version = appdata.version();
    let mut batch = std::collections::BTreeMap::<_, Vec<_>>::new();
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
        match batch.get_mut(&path) {
            Some(vec) => vec.push(actor),
            None => {
                batch.insert(path, vec![actor]);
            }
        }
    }
    for path in batch.keys().cloned().collect::<Vec<_>>() {
        let Some(path) = path else { continue };
        match paks.iter_mut().find_map(|(pak_file, pak)| {
            asset::get(
                pak,
                pak_file,
                cache.as_deref(),
                &path,
                version,
                |asset, _| Ok(extras::get_mesh_info(asset)?),
            )
            .ok()
            .map(|mesh| (mesh, pak_file, pak))
        }) {
            Some(((positions, indices, uvs, mats, _mat_data), pak_file, pak)) => {
                registry.0.insert(path.clone(), (
                    meshes.add(
                        Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList, default())
                            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs.into_iter().map(|uv| uv[0]).collect::<Vec<_>>())
                            .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
                    ),
                    match appdata.textures {
                        true => {
                            let mats: Vec<_> = mats
                                .into_iter()
                                .map(|path| {
                                    match asset::get(
                                        pak,
                                        pak_file,
                                        cache.as_deref(),
                                        &path,
                                        version,
                                        |mat, _| Ok(extras::get_tex_paths(mat)),
                                    ) {
                                        Ok(paths) => {
                                            paths.into_iter().find_map(|path|
                                                match asset::get(
                                                    pak,
                                                    pak_file,
                                                    cache.as_deref(),
                                                    &path,
                                                    version,
                                                    |tex, bulk| {
                                                        Ok(extras::get_tex_info(tex, bulk)?)
                                                    },
                                                ) {
                                                    Ok((false, x, y, data)) => Some((x,y,data)),
                                                    Ok((true, ..)) => None,
                                                    Err(e) => {
                                                        notif.send(
                                                            Notif {
                                                                message: format!(
                                                                    "{}: {e}",
                                                                    path.split('/')
                                                                        .last()
                                                                        .unwrap_or_default()
                                                                ),
                                                                kind: Warning
                                                            }
                                                        );
                                                        None
                                                    }
                                                }
                                            )
                                        },
                                        _ => None,
                                    }
                                })
                                .collect();
                                mats.into_iter().flatten().map(|(width, height, data)| {
                                    materials.add(unlit::Unlit {
                                        texture: images.add(Image {
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
                                                format: bevy::render::render_resource::TextureFormat::Rgba8Unorm,
                                                usage: bevy::render::render_resource::TextureUsages::TEXTURE_BINDING,
                                                view_formats: &[bevy::render::render_resource::TextureFormat::Rgba8Unorm],
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
                                        }),
                                    })
                                }).collect()
                        },
                        false => vec![consts.grid.clone_weak()],
                    }
                ));
            }
            None => {
                notif.send(Notif {
                    message: format!("mesh not found at {path}"),
                    kind: Warning,
                });
                let removed = batch.remove(&Some(path));
                if let Some(actors) = removed {
                    match batch.get_mut(&None) {
                        Some(vec) => vec.extend(actors),
                        None => {
                            batch.insert(None, actors);
                        }
                    }
                }
                continue;
            }
        }
    }
    for (path, actors) in batch {
        match path {
            Some(path) => {
                let (mesh, material) = &registry.0[&path];
                for actor in actors {
                    commands.spawn((
                        MaterialMeshBundle {
                            mesh: mesh.clone_weak(),
                            material: material
                                .first()
                                .map(Handle::clone_weak)
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
                            consts.bounds.clone_weak(),
                            SpatialBundle {
                                visibility: Visibility::Hidden,
                                transform: actor.transform(&asset),
                                ..default()
                            },
                            bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                            actor,
                        ))
                        .with_children(|parent| {
                            parent.spawn(MaterialMeshBundle {
                                mesh: consts.cube.clone_weak(),
                                material: consts.unselected.clone_weak(),
                                visibility: Visibility::Visible,
                                ..default()
                            });
                        });
                }
                continue;
            }
        }
    }
    map.0 = Some((asset, path.clone()));
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
    let Some((map, path)) = &mut map.0 else {
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

pub fn add_pak(_: Trigger<triggers::AddPak>, mut appdata: ResMut<AppData>) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("add pak folder")
        .pick_folder()
        .and_then(|path| path.to_str().map(str::to_string))
    {
        appdata.paks.push(path)
    }
}

pub fn transplant(
    _: Trigger<triggers::Transplant>,
    mut notif: EventWriter<Notif>,
    appdata: ResMut<AppData>,
    mut transplant: NonSendMut<Transplant>,
) {
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
