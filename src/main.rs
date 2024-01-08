#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]
use bevy::prelude::*;

mod asset;

type Asset = unreal_asset::Asset<std::io::BufReader<std::fs::File>>;

struct Map(Option<Asset>);

#[derive(Event)]
enum Notifs {
    Static(&'static str),
    Dynamic(String),
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "stove".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_non_send_resource(Map(None))
        .add_event::<Notifs>()
        // set window icon
        .add_systems(Startup, |windows: NonSend<bevy::winit::WinitWindows>| {
            let icon = winit::window::Icon::from_rgba(
                include_bytes!("../assets/pot.rgba").to_vec(),
                64,
                64,
            )
            .unwrap();
            for window in windows.windows.values() {
                window.set_window_icon(Some(icon.clone()))
            }
        })
        .add_systems(
            Startup,
            |mut notifs: EventWriter<Notifs>, mut map: NonSendMut<Map>| {
                let Some(path) = std::env::args().nth(1) else {
                    return;
                };
                let path = std::path::PathBuf::from(path);
                if !path.exists() {
                    notifs.send(Notifs::Static("the given path does not exist"));
                    return;
                }
                match asset::open(
                    &path,
                    unreal_asset::engine_version::EngineVersion::VER_UE5_1,
                ) {
                    Ok(asset) => map.0 = Some(asset),
                    Err(e) => notifs.send(Notifs::Dynamic(e.to_string())),
                }
            },
        )
        .run();
}
