use super::*;

pub fn set_icon(windows: NonSend<bevy::winit::WinitWindows>) {
    let icon =
        winit::window::Icon::from_rgba(include_bytes!("../assets/pot.rgba").to_vec(), 64, 64)
            .unwrap();
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()))
    }
}

pub fn check_updates(mut notif: EventWriter<Notif>) {
    use update_informer::Check;
    if let Ok(Some(new)) = update_informer::new(
        update_informer::registry::GitHub,
        "bananaturtlesandwich/stove",
        env!("CARGO_PKG_VERSION"),
    )
    .check_version()
    {
        notif.send(Notif {
            // yes i'm petty and hate the v prefix
            message: format!(
                "{}.{}.{} now available!",
                new.semver().major,
                new.semver().minor,
                new.semver().patch
            ),
            kind: Info,
        });
    }
}

pub fn check_args(mut notif: EventWriter<Notif>, mut commands: Commands) {
    let Some(path) = std::env::args().nth(1) else {
        return;
    };
    let path = std::path::PathBuf::from(path);
    if !path.exists() {
        notif.send(Notif {
            message: "the given path does not exist".into(),
            kind: Error,
        });
        return;
    }
    commands.trigger(triggers::Open(Some(path)));
}

pub fn initialise(
    mut commands: Commands,
    mut client: ResMut<Client>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<unlit::Unlit>>,
    mut wire: ResMut<Assets<wire::Wire>>,
    mut images: ResMut<Assets<Image>>,
) {
    use discord_rich_presence::DiscordIpc;
    client.0 = discord_rich_presence::DiscordIpcClient::new("1059578289737433249")
        .ok()
        .and_then(|mut cl| {
            (cl.connect().is_ok() && cl.set_activity(activity()).is_ok()).then_some(cl)
        });
    use smooth_bevy_cameras::controllers::unreal::*;
    commands
        .spawn((
            Camera3dBundle {
                tonemapping: bevy::core_pipeline::tonemapping::Tonemapping::None,
                ..default()
            },
            bevy_mod_raycast::deferred::RaycastSource::<()>::new_cursor()
                .with_visibility(bevy_mod_raycast::immediate::RaycastVisibility::Ignore),
        ))
        .insert(UnrealCameraBundle::new(
            UnrealCameraController::default(),
            // for some reason it doesn't work at the origin
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
        ));
    commands.insert_resource(Constants {
        cube: meshes.add(
            Mesh::new(
                bevy::render::render_resource::PrimitiveTopology::LineList,
                default(),
            )
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_POSITION,
                vec![
                    // front verts
                    bevy::math::vec3(-0.5, -0.5, -0.5),
                    bevy::math::vec3(-0.5, 0.5, -0.5),
                    bevy::math::vec3(0.5, -0.5, -0.5),
                    bevy::math::vec3(0.5, 0.5, -0.5),
                    // back verts
                    bevy::math::vec3(-0.5, -0.5, 0.5),
                    bevy::math::vec3(-0.5, 0.5, 0.5),
                    bevy::math::vec3(0.5, -0.5, 0.5),
                    bevy::math::vec3(0.5, 0.5, 0.5),
                ],
            )
            .with_inserted_indices(bevy::render::mesh::Indices::U16(vec![
                0, 1, 0, 2, 1, 3, 2, 3, 4, 5, 4, 6, 5, 7, 6, 7, 4, 0, 5, 1, 6, 2, 7, 3,
            ])),
        ),
        bounds: meshes.add(Cuboid::from_corners(Vec3::splat(-0.5), Vec3::splat(0.5))),
        unselected: wire.add(wire::Wire { selected: false }),
        selected: wire.add(wire::Wire { selected: true }),
        grid: materials.add(unlit::Unlit {
            texture: images.add(Image {
                data: include_bytes!("../assets/DefaultWhiteGrid.rgba").into(),
                texture_descriptor: bevy::render::render_resource::TextureDescriptor {
                    label: None,
                    size: bevy::render::render_resource::Extent3d {
                        width: 128,
                        height: 128,
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
        }),
    })
}

pub fn load_paks(mut notif: EventWriter<Notif>, appdata: Res<AppData>, mut paks: ResMut<Paks>) {
    let Some(pak) = appdata.pak else { return };
    let key = match hex::decode(appdata.paks[pak].1.trim_start_matches("0x")) {
        Ok(key) if !appdata.paks[pak].1.is_empty() => Some(key),
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
    let path = &appdata.paks[pak].0;
    let Ok(files) = std::fs::read_dir(path) else {
        return;
    };
    paks.1 = files
        .filter_map(Result::ok)
        .map(|dir| dir.path())
        .filter_map(|path| {
            use aes::cipher::KeyInit;
            let mut pak_file = std::io::BufReader::new(std::fs::File::open(path.clone()).ok()?);
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
            Some((path, pak))
        })
        .collect();
    // obtain game name
    let Some((_, pak)) = paks.1.first() else {
        return;
    };
    let mut split = pak.mount_point().split('/').peekable();
    while let Some((game, content)) = split.next().zip(split.peek()) {
        if game != "Engine" && content == &"Content" {
            paks.0 = game.into();
            return;
        }
    }
    for entry in pak.files() {
        let mut split = entry.split('/').take(2);
        if let Some((game, content)) = split.next().zip(split.next()) {
            if game != "Engine" && content == "Content" {
                paks.0 = game.into();
                return;
            }
        }
    }
}
