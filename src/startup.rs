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
    events.send(Events::Open(path))
}

pub fn setup_camera(mut commands: Commands) {
    use smooth_bevy_cameras::controllers::unreal::*;
    commands
        .spawn(Camera3dBundle {
            tonemapping: bevy::core_pipeline::tonemapping::Tonemapping::None,
            ..Default::default()
        })
        .insert(UnrealCameraBundle::new(
            UnrealCameraController::default(),
            // for some reason it doesn't work at the origin
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
        ));
}
