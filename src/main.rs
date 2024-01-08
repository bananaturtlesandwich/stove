#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]
use bevy::prelude::*;
use egui_notify::ToastLevel::{Error, Info, Success, Warning};

mod asset;
mod persistence;
mod startup;
mod ui;

type Asset = unreal_asset::Asset<std::io::BufReader<std::fs::File>>;

struct Map(Option<Asset>);

#[derive(Event)]
struct Notif {
    message: String,
    kind: egui_notify::ToastLevel,
}

#[derive(Default, Resource)]
struct Notifs(egui_notify::Toasts);

#[derive(Resource)]
struct AppData {
    version: usize,
    paks: Vec<String>,
    distance: f32,
    aes: String,
    cache: bool,
    script: String,
}

fn config() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|path| path.join("stove"))
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "stove".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bevy_egui::EguiPlugin)
        .insert_non_send_resource(Map(None))
        .init_resource::<Notifs>()
        .add_event::<Notif>()
        // set window icon
        .add_systems(PreStartup, startup::set_icon)
        // commands aren't applied immediately without this
        .add_systems(Startup, (persistence::load, apply_deferred).chain())
        .add_systems(
            PostUpdate,
            persistence::write.after(bevy_egui::EguiSet::InitContexts),
        )
        // allow open with...
        .add_systems(Startup, startup::check_args.after(persistence::load))
        // check for updates
        .add_systems(Startup, startup::check_updates)
        // show notifications
        .add_systems(
            Update,
            |mut ctx: bevy_egui::EguiContexts,
             mut queue: EventReader<Notif>,
             mut notifs: ResMut<Notifs>| {
                for notif in queue.read() {
                    notifs.0.add(egui_notify::Toast::custom(
                        notif.message.clone(),
                        notif.kind.clone(),
                    ));
                }
                notifs.0.show(ctx.ctx_mut());
            },
        )
        .add_systems(Update, ui::ui)
        .run();
}

use unreal_asset::engine_version::EngineVersion::*;

const VERSIONS: [(unreal_asset::engine_version::EngineVersion, &str); 33] = [
    (UNKNOWN, "unknown"),
    (VER_UE4_OLDEST_LOADABLE_PACKAGE, "oldest"),
    (VER_UE4_0, "4.0"),
    (VER_UE4_1, "4.1"),
    (VER_UE4_2, "4.2"),
    (VER_UE4_3, "4.3"),
    (VER_UE4_4, "4.4"),
    (VER_UE4_5, "4.5"),
    (VER_UE4_6, "4.6"),
    (VER_UE4_7, "4.7"),
    (VER_UE4_8, "4.8"),
    (VER_UE4_9, "4.9"),
    (VER_UE4_10, "4.10"),
    (VER_UE4_11, "4.11"),
    (VER_UE4_12, "4.12"),
    (VER_UE4_13, "4.13"),
    (VER_UE4_14, "4.14"),
    (VER_UE4_15, "4.15"),
    (VER_UE4_16, "4.16"),
    (VER_UE4_17, "4.17"),
    (VER_UE4_18, "4.18"),
    (VER_UE4_19, "4.19"),
    (VER_UE4_20, "4.20"),
    (VER_UE4_21, "4.21"),
    (VER_UE4_22, "4.22"),
    (VER_UE4_23, "4.23"),
    (VER_UE4_24, "4.24"),
    (VER_UE4_25, "4.25"),
    (VER_UE4_26, "4.26"),
    (VER_UE4_27, "4.27"),
    (VER_UE5_0, "5.0"),
    (VER_UE5_1, "5.1"),
    (VER_UE5_2, "5.2"),
];
