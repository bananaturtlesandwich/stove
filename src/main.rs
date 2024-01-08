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
    paks: Vec<std::path::PathBuf>,
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
        .run();
}
