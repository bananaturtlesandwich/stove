use super::*;

pub fn set_icon(windows: NonSend<bevy::winit::WinitWindows>) {
    let icon =
        winit::window::Icon::from_rgba(include_bytes!("../assets/pot.rgba").to_vec(), 64, 64)
            .unwrap();
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()))
    }
}

pub fn check_updates(mut events: EventWriter<Events>) {
    use update_informer::Check;
    if let Ok(Some(new)) = update_informer::new(
        update_informer::registry::GitHub,
        "bananaturtlesandwich/stove",
        env!("CARGO_PKG_VERSION"),
    )
    .check_version()
    {
        events.send(Events::Notif {
            // yes i'm petty and hate the v prefix
            message: format!(
                "{}.{}.{} now available!",
                new.semver().major,
                new.semver().minor,
                new.semver().patch
            ),
            kind: Info,
        })
    }
}

pub fn check_args(mut events: EventWriter<Events>) {
    let Some(path) = std::env::args().nth(1) else {
        return;
    };
    let path = std::path::PathBuf::from(path);
    if !path.exists() {
        events.send(Events::Notif {
            message: "the given path does not exist".into(),
            kind: Error,
        });
        return;
    }
    events.send(Events::Open(Some(path)))
}

pub fn initialise(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    use smooth_bevy_cameras::controllers::unreal::*;
    commands
        .spawn((
            Camera3dBundle {
                tonemapping: bevy::core_pipeline::tonemapping::Tonemapping::None,
                ..Default::default()
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
            Mesh::new(bevy::render::render_resource::PrimitiveTopology::LineList)
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
                .with_indices(Some(bevy::render::mesh::Indices::U16(vec![
                    0, 1, 0, 2, 1, 3, 2, 3, 4, 5, 4, 6, 5, 7, 6, 7, 4, 0, 5, 1, 6, 2, 7, 3,
                ]))),
        ),
        bounds: meshes.add(shape::Box::from_corners(Vec3::splat(-0.5), Vec3::splat(0.5)).into()),
    })
}
