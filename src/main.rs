#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]
use bevy::prelude::*;

mod asset;

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
        // check for updates
        .add_systems(Startup, |mut notifs: EventWriter<Notif>| {
            use update_informer::Check;
            if let Ok(Some(new)) = update_informer::new(
                update_informer::registry::GitHub,
                "bananaturtlesandwich/stove",
                env!("CARGO_PKG_VERSION"),
            )
            .check_version()
            {
                notifs.send(Notif {
                    // yes i'm petty and hate the v prefix
                    message: format!(
                        "{}.{}.{} now available!",
                        new.semver().major,
                        new.semver().minor,
                        new.semver().patch
                    ),
                    kind: egui_notify::ToastLevel::Info,
                })
            }
        })
        // get app data
        .add_systems(
            Startup,
            |mut commands: Commands, mut ctx: bevy_egui::EguiContexts| {
                let mut appdata = AppData {
                    version: 0,
                    paks: vec![],
                    distance: 100000.0,
                    aes: String::new(),
                    cache: true,
                    script: String::new(),
                };
                ctx.ctx_mut().memory_mut(|storage| {
                    if let Some(config) = config()
                        .map(|config| config.join("config.ron"))
                        .and_then(|path| std::fs::read_to_string(path).ok())
                        .and_then(|str| ron::from_str::<egui::util::IdTypeMap>(&str).ok())
                    {
                        storage.data = config
                    }
                    let data = &mut storage.data;
                    fn retrieve<T: egui::util::id_type_map::SerializableAny>(
                        val: &mut T,
                        key: &str,
                        data: &mut egui::util::IdTypeMap,
                    ) {
                        if let Some(inner) = data.get_persisted(egui::Id::new(key)) {
                            *val = inner
                        }
                    }
                    retrieve(&mut appdata.version, "VERSION", data);
                    retrieve(&mut appdata.paks, "PAKS", data);
                    retrieve(&mut appdata.distance, "DIST", data);
                    retrieve(&mut appdata.aes, "AES", data);
                    retrieve(&mut appdata.cache, "CACHE", data);
                    retrieve(&mut appdata.script, "SCRIPT", data);
                });
                commands.insert_resource(appdata);
            },
        )
        // save app data
        .add_systems(
            PostUpdate,
            |mut ctx: bevy_egui::EguiContexts,
             appdata: Res<AppData>,
             exit: EventReader<bevy::app::AppExit>| {
                if exit.is_empty() {
                    return;
                }
                use egui::Id;
                ctx.ctx_mut().memory_mut(|storage| {
                    let storage = &mut storage.data;
                    storage.insert_persisted(Id::new("VERSION"), appdata.version);
                    storage.insert_persisted(Id::new("PAKS"), appdata.paks.clone());
                    storage.insert_persisted(Id::new("DIST"), appdata.distance);
                    storage.insert_persisted(Id::new("AES"), appdata.aes.clone());
                    storage.insert_persisted(Id::new("CACHE"), appdata.cache);
                    storage.insert_persisted(Id::new("SCRIPT"), appdata.script.clone());
                    if let Some(config) = config() {
                        let _ = std::fs::create_dir_all(&config);
                        if let Ok(data) = ron::to_string(&storage) {
                            let _ = std::fs::write(config.join("config.ron"), data);
                        }
                    }
                })
            },
        )
        // allow open with...
        .add_systems(
            Startup,
            |mut notifs: EventWriter<Notif>, mut map: NonSendMut<Map>| {
                let Some(path) = std::env::args().nth(1) else {
                    return;
                };
                let path = std::path::PathBuf::from(path);
                if !path.exists() {
                    notifs.send(Notif {
                        message: "the given path does not exist".into(),
                        kind: egui_notify::ToastLevel::Error,
                    });
                    return;
                }
                match asset::open(
                    &path,
                    unreal_asset::engine_version::EngineVersion::VER_UE5_1,
                ) {
                    Ok(asset) => map.0 = Some(asset),
                    Err(e) => notifs.send(Notif {
                        message: e.to_string(),
                        kind: egui_notify::ToastLevel::Error,
                    }),
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
                    notifs.0.add(egui_notify::Toast::custom(
                        notif.message.clone(),
                        notif.kind.clone(),
                    ));
                }
                notifs.0.show(ctx.ctx_mut());
            },
        )
        .add_systems(Update, |mut ctx: bevy_egui::EguiContexts, mut appdata: ResMut<AppData>, mut notif: EventWriter<Notif>| {
            egui::SidePanel::left("sidepanel").show(ctx.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("file", |ui| {
                    if ui.add(egui::Button::new("open").shortcut_text("ctrl + o")).clicked() {
                    }
                    if ui.add(egui::Button::new("save").shortcut_text("ctrl + s")).clicked(){
                    }
                    if ui.add(egui::Button::new("save as").shortcut_text("ctrl + shift + s")).clicked(){
                    }
                });
                egui::ComboBox::from_id_source("version")
                    .show_index(ui, &mut appdata.version, 33, |i| VERSIONS[i].1.to_string());
                ui.menu_button("paks", |ui| {
                    egui::TextEdit::singleline(&mut appdata.aes)
                        .clip_text(false)
                        .hint_text("aes key if needed")
                        .show(ui);
                    let mut remove_at = None;
                    egui::ScrollArea::vertical().show_rows(
                        ui,
                        ui.text_style_height(&egui::TextStyle::Body),
                        appdata.paks.len(),
                        |ui, range| for i in range {
                            ui.horizontal(|ui| {
                                ui.label(&appdata.paks[i]);
                                if ui.button("x").clicked(){
                                    remove_at = Some(i);
                                }
                            });
                        }
                    );
                    if let Some(i) = remove_at {
                        appdata.paks.remove(i);
                    }
                    if ui.add(egui::Button::new("add pak folder").shortcut_text("alt + o")).clicked() {
                    }
                });
                ui.menu_button("options", |ui| {
                    ui.menu_button("about",|ui|{
                        ui.horizontal_wrapped(|ui|{
                            let size = ui.fonts(|fonts| fonts.glyph_width(&egui::TextStyle::Body.resolve(ui.style()), ' '));
                            ui.spacing_mut().item_spacing.x = size;
                            ui.label("stove is an editor for cooked unreal map files running on my spaghetti code - feel free to help untangle it on");
                            ui.hyperlink_to("github","https://github.com/bananaturtlesandwich/stove");
                            ui.label(egui::special_emojis::GITHUB.to_string());
                        });
                    });
                    ui.menu_button("shortcuts", |ui|{
                        let mut section = |heading: &str, bindings: &[(&str,&str)]| {
                            ui.menu_button(heading, |ui| {
                                egui::Grid::new(heading).striped(true).show(ui, |ui| {
                                    for (action, binding) in bindings {
                                        ui.label(*action);
                                        ui.label(*binding);
                                        ui.end_row();
                                    }
                                })
                            })
                        };
                        section(
                            "file",
                            &[
                                ("open","ctrl + o"),
                                ("save", "ctrl + s"),
                                ("save as","ctrl + shift + s"),
                                ("add pak folder", "alt + o")
                            ]
                        );
                        section(
                            "camera",
                            &[
                                ("move","w + a + s + d"),
                                ("rotate", "right-drag"),
                                ("change speed", "scroll"),
                            ]
                        );
                        section(
                            "viewport",
                            &[
                                ("toggle fullscreen", "alt + enter"),
                                ("hide ui", "h"),
                                ("select", "left-click"),
                                ("transplant", "ctrl + t")
                            ]
                        );
                        section(
                            "actor",
                            &[
                                ("focus", "f"),
                                ("move", "left-drag"),
                                ("rotate", "right-drag"),
                                ("scale", "middle-drag"),
                                ("copy location", "ctrl + c"),
                                ("paste location", "ctrl + v"),
                                ("duplicate", "alt + left-drag"),
                                ("delete", "delete"),
                                ("lock x / y / z axis", "x / y / z"),
                                ("lock x / y / z plane", "shift + x / y / z"),
                            ]
                        )
                    });
                    ui.horizontal(|ui|{
                        ui.label("cache meshes:");
                        ui.add(egui::Checkbox::without_text(&mut appdata.cache));
                    });
                    if ui.button("clear cache").clicked() {
                        match config() {
                            Some(cache) => match std::fs::remove_dir_all(cache.join("cache")) {
                                Ok(()) => notif.send(Notif {
                                    message: "cleared cache".into(),
                                    kind: egui_notify::ToastLevel::Info
                                }),
                                Err(e) => notif.send(Notif {
                                    message: e.to_string(),
                                    kind: egui_notify::ToastLevel::Error
                                }),
                            },
                            None => notif.send(Notif {
                                message: "cache does not exist".into(),
                                kind: egui_notify::ToastLevel::Warning
                            }),
                        };
                    }
                    ui.horizontal(|ui| {
                        ui.label("render distance:");
                        ui.add(
                            egui::widgets::DragValue::new(&mut appdata.distance)
                                .clamp_range(0..=100000)
                        )
                    });
                    ui.label("post-save commands");
                    ui.text_edit_multiline(&mut appdata.script);
                });
            });
            });
        })
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
