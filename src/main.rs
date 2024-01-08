#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]
use bevy::prelude::*;
use egui_notify::ToastLevel::{Error, Info, Success};

mod actor;
mod asset;
mod persistence;
mod startup;
mod ui;

type Asset = unreal_asset::Asset<std::io::BufReader<std::fs::File>>;

struct Map(Option<(Asset, std::path::PathBuf)>);

#[derive(Event)]
enum Events {
    Notif {
        message: String,
        kind: egui_notify::ToastLevel,
    },
    Open(std::path::PathBuf),
    SaveAs(bool),
    AddPak,
}

#[derive(Default, Resource)]
struct Notifs(egui_notify::Toasts);

#[derive(Default, Resource)]
struct AppData {
    version: usize,
    paks: Vec<String>,
    distance: f32,
    aes: String,
    cache: bool,
    script: String,
    query: String,
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
        .add_event::<Events>()
        .add_systems(PreStartup, startup::set_icon)
        // commands aren't applied immediately without this
        .add_systems(Startup, (persistence::load, apply_deferred).chain())
        .add_systems(Update, persistence::write)
        .add_systems(Startup, startup::check_args.after(persistence::load))
        .add_systems(Startup, startup::check_updates)
        .add_systems(
            Update,
            |mut drops: EventReader<bevy::window::FileDragAndDrop>,
             mut events: EventWriter<Events>| {
                for drop in drops.read() {
                    if let bevy::window::FileDragAndDrop::DroppedFile { path_buf, .. } = drop {
                        events.send(Events::Open(path_buf.clone()))
                    }
                }
            },
        )
        .add_systems(
            Update,
            |mut events: EventReader<Events>,
             mut notifs: ResMut<Notifs>,
             mut appdata: ResMut<AppData>,
             mut map: NonSendMut<Map>| {
                let mut queue = |message, kind| {
                    notifs.0.add(egui_notify::Toast::custom(message, kind));
                };
                for event in events.read() {
                    match event {
                        Events::Notif { message, kind } => queue(message.clone(), kind.clone()),
                        Events::Open(path) => {
                            match asset::open(path, VERSIONS[appdata.version].0) {
                                Ok(asset) => {
                                    map.0 = Some((asset, path.clone()));
                                    queue("map opened".into(), Success)
                                }
                                Err(e) => queue(e.to_string(), Error),
                            }
                        }
                        Events::SaveAs(ask) => {
                            let Some((map, path)) = &mut map.0 else {
                                queue("no map to save".into(), Error);
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
                                Ok(_) => queue("map saved".into(), Success),
                                Err(e) => queue(e.to_string(), Error),
                            }
                        }
                        Events::AddPak => {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("add pak folder")
                                .pick_folder()
                                .and_then(|path| path.to_str().map(str::to_string))
                            {
                                appdata.paks.push(path)
                            }
                        }
                    }
                }
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
