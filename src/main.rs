#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]
use bevy::prelude::*;

mod asset;

type Asset = unreal_asset::Asset<std::io::BufReader<std::fs::File>>;

struct Map(Option<Asset>);

#[derive(Event)]
enum Notif {
    Static(&'static str),
    Dynamic(String),
}

#[derive(Default, Resource)]
struct Notifs(egui_notify::Toasts);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "stove".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bevy_egui::EguiPlugin)
        .insert_non_send_resource(Map(None))
        .init_resource::<Notifs>()
        .add_event::<Notif>()
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
        // allow open with...
        .add_systems(
            Startup,
            |mut notifs: EventWriter<Notif>, mut map: NonSendMut<Map>| {
                let Some(path) = std::env::args().nth(1) else {
                    return;
                };
                let path = std::path::PathBuf::from(path);
                if !path.exists() {
                    notifs.send(Notif::Static("the given path does not exist"));
                    return;
                }
                match asset::open(
                    &path,
                    unreal_asset::engine_version::EngineVersion::VER_UE5_1,
                ) {
                    Ok(asset) => map.0 = Some(asset),
                    Err(e) => notifs.send(Notif::Dynamic(e.to_string())),
                }
            },
        )
        // show notifications
        .add_systems(
            Update,
            |mut ctx: bevy_egui::EguiContexts,
             mut queue: EventReader<Notif>,
             mut notifs: ResMut<Notifs>| {
                for notif in queue.read() {
                    match notif {
                        Notif::Static(message) => notifs.0.info(message.to_string()),
                        Notif::Dynamic(err) => notifs.0.error(err),
                    };
                }
                notifs.0.show(ctx.ctx_mut());
            },
        )
        .run();
}
