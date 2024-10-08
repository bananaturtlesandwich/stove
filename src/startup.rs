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

pub fn discord(mut client: ResMut<Client>) {
    use discord_rich_presence::DiscordIpc;
    client.0 = discord_rich_presence::DiscordIpcClient::new("1059578289737433249")
        .ok()
        .and_then(|mut cl| {
            (cl.connect().is_ok() && cl.set_activity(activity()).is_ok()).then_some(cl)
        });
}

pub fn camera(mut commands: Commands) {
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
}

pub fn consts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<unlit::Unlit>>,
    mut wire: ResMut<Assets<wire::Wire>>,
    mut images: ResMut<Assets<Image>>,
) {
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
                data: include_bytes!("../assets/proto.rgba").into(),
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
                    format: bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                    usage: bevy::render::render_resource::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb],
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
