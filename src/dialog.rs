use super::*;

pub fn respond(
    mut commands: Commands,
    actors: Query<Entity, With<actor::Actor>>,
    mut dialogs: EventReader<Dialog>,
    mut notif: EventWriter<Notif>,
    mut appdata: ResMut<AppData>,
    mut client: ResMut<Client>,
    mut map: NonSendMut<Map>,
    mut transplant: NonSendMut<Transplant>,
    mut registry: ResMut<Registry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    consts: Res<Constants>,
) {
    for event in dialogs.read() {
        match event {
            Dialog::Open(path) => {
                let Some(path) = path.clone().or_else(|| {
                    rfd::FileDialog::new()
                        .set_title("open map")
                        .add_filter("maps", &["umap"])
                        .pick_file()
                }) else {
                    continue;
                };
                match asset::open(&path, appdata.version()) {
                    Ok(asset) => {
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
                        #[link(name = "oo2core_win64", kind = "static")]
                        extern "C" {
                            fn OodleLZ_Decompress(
                                compBuf: *mut u8,
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
                                let mut pak_file =
                                    std::io::BufReader::new(std::fs::File::open(path).ok()?);
                                let mut pak = repak::PakBuilder::new();
                                if let Some(key) = key
                                    .as_deref()
                                    .and_then(|bytes| aes::Aes256::new_from_slice(bytes).ok())
                                {
                                    pak = pak.key(key);
                                }
                                #[cfg(target_os = "windows")]
                                {
                                    pak = pak.oodle(|| OodleLZ_Decompress);
                                }
                                let pak = pak.reader(&mut pak_file).ok()?;
                                Some((pak_file, pak))
                            })
                            .collect();
                        let cache = config()
                            .filter(|_| appdata.cache)
                            .map(|path| path.join("cache"));
                        let version = appdata.version();
                        for i in actor::get_actors(&asset) {
                            match actor::Actor::new(&asset, i) {
                                Ok(mut actor) => {
                                    if let actor::DrawType::Mesh(path) = &actor.draw_type {
                                        if !registry.0.contains_key(path) {
                                            match paks.iter_mut().find_map(|(pak_file, pak)| {
                                                asset::get(
                                                    pak,
                                                    pak_file,
                                                    cache.as_deref(),
                                                    path,
                                                    version,
                                                    |asset, _| Ok(extras::get_mesh_info(asset)?),
                                                )
                                                .ok()
                                                .map(|mesh| (mesh, pak_file, pak))
                                            }) {
                                                Some((
                                                    (positions, indices, uvs, mats, _mat_data),
                                                    pak_file,
                                                    pak,
                                                )) => {
                                                    let mats: Vec<_> = mats
                                                                    .into_iter()
                                                                    .map(|path| {
                                                                        match asset::get(
                                                                            pak,
                                                                            pak_file,
                                                                            cache.as_deref(),
                                                                            &path,
                                                                            version,
                                                                            |mat, _| Ok(extras::get_tex_path(mat)),
                                                                        ) {
                                                                            Ok(Some(path)) => match asset::get(
                                                                                pak,
                                                                                pak_file,
                                                                                cache.as_deref(),
                                                                                &path,
                                                                                version,
                                                                                |tex, bulk| {
                                                                                    Ok(extras::get_tex_info(tex, bulk)?)
                                                                                },
                                                                            ) {
                                                                                Ok(o) => Some(o),
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
                                                                            },
                                                                            _ => None,
                                                                        }
                                                                    })
                                                                    .collect();
                                                    registry.0.insert(path.clone(), (
                                                        meshes.add(
                                                            Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList)
                                                                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                                                                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs.into_iter().map(|uv| uv[0]).collect::<Vec<_>>())
                                                                .with_indices(Some(bevy::render::mesh::Indices::U32(indices)))
                                                        ),
                                                        mats.into_iter().flatten().map(|(width, height, data)| {
                                                            materials.add(StandardMaterial {
                                                                base_color_texture: Some(images.add(Image {
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
                                                                })),
                                                                ..default()
                                                            })
                                                        }).collect()
                                                    ));
                                                }
                                                None => {
                                                    notif.send(Notif {
                                                        message: format!(
                                                            "mesh not found for {}",
                                                            actor.name
                                                        ),
                                                        kind: Warning,
                                                    });
                                                    actor.draw_type = actor::DrawType::Cube;
                                                }
                                            }
                                        }
                                    }
                                    match &actor.draw_type {
                                        actor::DrawType::Mesh(path) => {
                                            let (mesh, material) = &registry.0[path];
                                            commands.spawn((
                                                MaterialMeshBundle {
                                                    mesh: mesh.clone_weak(),
                                                    material: material.first().map(Handle::clone_weak).unwrap_or(consts.grid.clone_weak()),
                                                    transform: actor.transform(&asset),
                                                    ..default()
                                                },
                                                bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                                                actor
                                            ));
                                        }
                                        actor::DrawType::Cube => {
                                            commands.spawn((
                                                PbrBundle {
                                                    mesh: consts.bounds.clone_weak(),
                                                    transform: actor.transform(&asset),
                                                    visibility: Visibility::Hidden,
                                                    ..default()
                                                },
                                                bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                                                actor
                                            // child because it's LineList which picking can't do
                                            )).with_children(|parent| {
                                                parent.spawn(
                                                    PbrBundle {
                                                        mesh: consts.cube.clone_weak(),
                                                        visibility: Visibility::Visible,
                                                        ..default()
                                                    },
                                                );
                                            });
                                        }
                                    }
                                }
                                Err(e) => notif.send(Notif {
                                    message: e.to_string(),
                                    kind: Warning,
                                }),
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
                    Err(e) => notif.send(Notif {
                        message: e.to_string(),
                        kind: Error,
                    }),
                }
            }
            Dialog::SaveAs(ask) => {
                let Some((map, path)) = &mut map.0 else {
                    notif.send(Notif {
                        message: "no map to save".into(),
                        kind: Error,
                    });
                    continue;
                };
                if *ask {
                    if let Some(new) = rfd::FileDialog::new()
                        .set_title("save map as")
                        .add_filter("maps", &["umap"])
                        .save_file()
                    {
                        *path = new;
                    }
                }
                match asset::save(map, path) {
                    Ok(_) => notif.send(Notif {
                        message: "map saved".into(),
                        kind: Success,
                    }),
                    Err(e) => notif.send(Notif {
                        message: e.to_string(),
                        kind: Error,
                    }),
                }
            }
            Dialog::AddPak => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("add pak folder")
                    .pick_folder()
                    .and_then(|path| path.to_str().map(str::to_string))
                {
                    appdata.paks.push(path)
                }
            }
            Dialog::Transplant => {
                let Some(path) = rfd::FileDialog::new()
                    .set_title("open map")
                    .add_filter("maps", &["umap"])
                    .pick_file()
                else {
                    continue;
                };
                match asset::open(path, appdata.version()) {
                    Ok(donor) => {
                        // no need for verbose warnings here
                        let actors: Vec<_> = actor::get_actors(&donor)
                            .into_iter()
                            .filter_map(|index| actor::Actor::new(&donor, index).ok())
                            .collect();
                        let selected = Vec::with_capacity(actors.len());
                        transplant.0 = Some((donor, actors, selected));
                    }
                    Err(e) => notif.send(Notif {
                        message: e.to_string(),
                        kind: Error,
                    }),
                }
            }
        }
    }
}
