#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity, clippy::too_many_arguments)]
use bevy::prelude::*;
use egui_notify::ToastLevel::{Error, Info, Success, Warning};

mod actor;
mod asset;
mod extras;
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
struct Registry(std::collections::BTreeMap<String, Handle<Mesh>>);

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

#[derive(Resource)]
struct Constants(
    Handle<Mesh>,
    Handle<bevy::pbr::wireframe::WireframeMaterial>,
);

enum Wrapper {
    File(std::io::BufReader<std::fs::File>),
    Bytes(std::io::Cursor<Vec<u8>>),
}

impl std::io::Read for Wrapper {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Wrapper::File(file) => file.read(buf),
            Wrapper::Bytes(bytes) => bytes.read(buf),
        }
    }
}

impl std::io::Seek for Wrapper {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match self {
            Wrapper::File(file) => file.seek(pos),
            Wrapper::Bytes(bytes) => bytes.seek(pos),
        }
    }
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
        .add_plugins(bevy::pbr::wireframe::WireframePlugin)
        .add_plugins(bevy_egui::EguiPlugin)
        .add_plugins(smooth_bevy_cameras::LookTransformPlugin)
        .add_plugins(smooth_bevy_cameras::controllers::unreal::UnrealCameraPlugin::default())
        .insert_non_send_resource(Map(None))
        .init_resource::<Notifs>()
        .init_resource::<Registry>()
        .add_event::<Events>()
        .add_systems(PreStartup, startup::set_icon)
        // commands aren't applied immediately without this
        .add_systems(Startup, (persistence::load, apply_deferred).chain())
        .add_systems(Update, persistence::write)
        .add_systems(Startup, startup::check_args.after(persistence::load))
        .add_systems(Startup, startup::check_updates)
        .add_systems(Startup, startup::initialise)
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
        .add_systems(Update, ui::ui)
        .add_systems(
            Update,
            |mut commands: Commands,
             actors: Query<Entity, With<actor::Actor>>,
             mut events: EventReader<Events>,
             mut notifs: ResMut<Notifs>,
             mut appdata: ResMut<AppData>,
             mut map: NonSendMut<Map>,
             mut registry: ResMut<Registry>,
             mut meshes: ResMut<Assets<Mesh>>, consts: Res<Constants>| {
                let mut queue = |message, kind| {
                    notifs.0.add(egui_notify::Toast::custom(message, kind));
                };
                for event in events.read() {
                    match event {
                        Events::Notif { message, kind } => queue(message.clone(), kind.clone()),
                        Events::Open(path) => {
                            match asset::open(path, VERSIONS[appdata.version].0) {
                                Ok(asset) => {
                                    for actor in actors.iter() {
                                        commands.entity(actor).despawn_recursive();
                                    }
                                    let key =
                                        match hex::decode(appdata.aes.trim_start_matches("0x"))
                                        {
                                            Ok(key) if !appdata.aes.is_empty() => Some(key),
                                            Ok(_) => None,
                                            Err(_) => {
                                                queue("aes key is invalid hex".into(), Warning);
                                                None
                                            }
                                        };
                                    #[link(name = "oo2core_win64", kind = "static")]
                                    extern "C" {
                                        fn OodleLZ_Decompress(
                                            compBuf: *mut u8,
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
                                    let mut paks: Vec<_> = appdata
                                        .paks
                                        .iter()
                                        .filter_map(|dir| std::fs::read_dir(dir).ok())
                                        .flatten()
                                        .filter_map(Result::ok)
                                        .map(|dir| dir.path())
                                        .filter_map(|path| {
                                            use aes::cipher::KeyInit;
                                            let mut pak_file = std::io::BufReader::new(
                                                std::fs::File::open(path).ok()?,
                                            );
                                            let mut pak = repak::PakBuilder::new();
                                            if let Some(key) =
                                                key.as_deref().and_then(|bytes| {
                                                    aes::Aes256::new_from_slice(bytes).ok()
                                                })
                                            {
                                                pak = pak.key(key);
                                            }
                                            #[cfg(target_os = "windows")]
                                            {
                                                pak = pak.oodle(|| OodleLZ_Decompress);
                                            }
                                            let pak = pak.reader(&mut pak_file).ok()?;
                                            Some((pak_file, pak))
                                        })
                                        .collect();
                                    let cache = config()
                                        .filter(|_| appdata.cache)
                                        .map(|path| path.join("cache"));
                                    let version = VERSIONS[appdata.version].0;
                                    for i in actor::get_actors(&asset) {
                                        match actor::Actor::new(&asset, i) {
                                            Ok(mut actor) => {
                                                if let actor::DrawType::Mesh(path) = &actor.draw_type{
                                                    if !registry.0.contains_key(path) {
                                                        match paks.iter_mut().find_map(|(pak_file , pak)| asset::get(
                                                                pak,
                                                                pak_file,
                                                                cache.as_deref(),
                                                                path,
                                                                version,
                                                                |asset, _| Ok(extras::get_mesh_info(asset)?),
                                                            ).ok()
                                                        ) {
                                                            Some((positions, indices, ..)) => {
                                                                registry.0.insert(path.clone(), meshes.add(Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList)
                                                                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                                                                    .with_indices(Some(bevy::render::mesh::Indices::U32(indices)))));
                                                            }
                                                            None => {
                                                                queue(format!("mesh not found for {}", actor.name), Warning);
                                                                actor.draw_type = actor::DrawType::Cube;
                                                            },
                                                        }
                                                    }
                                                }
                                                commands.spawn((MaterialMeshBundle {
                                                    mesh: match &actor.draw_type{
                                                        actor::DrawType::Mesh(path) => registry.0[path].clone_weak(),
                                                        actor::DrawType::Cube => consts.0.clone_weak(),
                                                    },
                                                    material: consts.1.clone_weak(),
                                                    transform: actor.transform(&asset),
                                                    ..default()
                                                }, actor));
                                            }
                                            Err(e) => queue(e.to_string(), Warning),
                                        }
                                    }
                                    map.0 = Some((asset, path.clone()));
                                    queue("map opened".into(), Success);
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
