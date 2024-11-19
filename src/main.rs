#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity, clippy::too_many_arguments, clippy::unit_arg)]
use bevy::prelude::*;
#[allow(unused_imports)]
#[cfg(debug_assertions)]
use bevy_dylib;
use egui_notify::ToastLevel::{Error, Info, Success, Warning};

mod action;
mod actor;
mod asset;
mod dialog;
mod extras;
mod input;
mod persistence;
mod picking;
mod startup;
mod triggers;
mod ui;
mod unlit;
mod wire;

type Asset = unreal_asset::Asset<Wrapper>;
type Export = unreal_asset::Export<unreal_asset::types::PackageIndex>;

#[derive(Default)]
struct Map(Option<(Asset, Option<std::path::PathBuf>, Vec<String>, Vec<String>)>);

#[derive(Default)]
struct Transplant(Option<(Asset, Vec<actor::Actor>, Vec<usize>)>);

#[derive(Event)]
struct Notif {
    message: String,
    kind: egui_notify::ToastLevel,
}

#[derive(Default, Resource)]
struct Notifs(egui_notify::Toasts);

#[derive(Default, Resource)]
struct Registry {
    meshes: std::collections::BTreeMap<String, (Handle<Mesh>, Option<String>)>,
    mats: std::collections::BTreeMap<String, Handle<unlit::Unlit>>,
}

#[derive(Default, Resource)]
struct Focus(Option<Vec3>);

#[derive(Default, Resource)]
struct AppData {
    version: usize,
    paks: Vec<(String, String)>,
    pak: Option<usize>,
    cache: bool,
    textures: bool,
    wireframe: bool,
    script: String,
    query: String,
    cap: bool,
    rate: f64,
}

#[derive(Clone)]
enum GamePath {
    Loose(std::path::PathBuf),
    Packed(String),
}

#[derive(Default, Resource)]
struct Content {
    game: String,
    folder: std::path::PathBuf,
    maps: Vec<(String, GamePath)>,
    paks: Vec<(std::path::PathBuf, repak::PakReader)>,
}

impl AppData {
    fn version(&self) -> unreal_asset::engine_version::EngineVersion {
        VERSIONS[self.version].0
    }
}

#[derive(Resource)]
struct Constants {
    cube: Handle<Mesh>,
    bounds: Handle<Mesh>,
    unselected: Handle<wire::Wire>,
    selected: Handle<wire::Wire>,
    grid: Handle<unlit::Unlit>,
}

#[derive(Default, Resource)]
enum Drag {
    #[default]
    None,
    Translate(Vec3),
    Scale(Vec2),
    Rotate(Vec2, Vec2),
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Resource)]
enum Lock {
    #[default]
    XYZ,
    XY,
    YZ,
    ZX,
    X,
    Y,
    Z,
}

#[derive(Default, Resource)]
struct Buffer(Vec3);

#[derive(Default, Resource)]
struct Hidden(bool);

#[derive(Default, Resource)]
struct FromContent(bool);

#[derive(Default, Resource)]
struct Client(Option<discord_rich_presence::DiscordIpcClient>);

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

fn activity() -> discord_rich_presence::activity::Activity<'static> {
    use discord_rich_presence::activity::*;
    Activity::new()
        .state("idle")
        .assets(Assets::new().large_image("pot"))
        .buttons(vec![Button::new(
            "homepage",
            "https://github.com/bananaturtlesandwich/stove",
        )])
}

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "stove".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::render::RenderPlugin {
                    render_creation: bevy::render::settings::RenderCreation::Automatic(
                        bevy::render::settings::WgpuSettings {
                            features: bevy::render::settings::WgpuFeatures::POLYGON_MODE_LINE,
                            ..default()
                        },
                    ),
                    ..default()
                }),
            unlit::UnlitPlugin,
            wire::WirePlugin,
            bevy_egui::EguiPlugin,
            smooth_bevy_cameras::LookTransformPlugin,
            smooth_bevy_cameras::controllers::unreal::UnrealCameraPlugin {
                override_input_system: true,
            },
            bevy_mod_raycast::deferred::DeferredRaycastingPlugin::<()>::default(),
            bevy_mod_outline::OutlinePlugin,
            bevy_mod_outline::AutoGenerateOutlineNormalsPlugin,
            bevy_framepace::FramepacePlugin,
            bevy::pbr::wireframe::WireframePlugin,
        ))
        .init_non_send_resource::<Map>()
        .init_non_send_resource::<Transplant>()
        .init_resource::<Notifs>()
        .init_resource::<Registry>()
        .init_resource::<Focus>()
        .init_resource::<Drag>()
        .init_resource::<Lock>()
        .init_resource::<Buffer>()
        .init_resource::<Hidden>()
        .init_resource::<FromContent>()
        .init_resource::<Client>()
        .init_resource::<Content>()
        .insert_resource(bevy::pbr::wireframe::WireframeConfig {
            global: false,
            default_color: bevy::color::palettes::css::WHITE.into(),
        })
        .add_event::<Notif>()
        .add_systems(PreStartup, startup::set_icon)
        .add_systems(
            Startup,
            (
                startup::check_updates,
                startup::discord,
                startup::camera,
                startup::consts,
                (persistence::load, startup::check_args).chain(),
            ),
        )
        .add_systems(
            Update,
            (
                persistence::write,
                |mut drops: EventReader<bevy::window::FileDragAndDrop>, mut commands: Commands| {
                    for drop in drops.read() {
                        if let bevy::window::FileDragAndDrop::DroppedFile { path_buf, .. } = drop {
                            commands.trigger(triggers::Open(Some(path_buf.clone())));
                        }
                    }
                },
                ui::sidebar,
                ui::notifs,
                input::shortcuts,
                action::approach,
            ),
        )
        // post update because egui isn't built until update
        .add_systems(
            PostUpdate,
            ((picking::pick, picking::drag).chain(), input::camera),
        )
        .observe(dialog::open)
        .observe(dialog::from_content)
        .observe(dialog::save_as)
        .observe(dialog::add_pak)
        .observe(dialog::transplant_from)
        .observe(dialog::transplant_into)
        .observe(action::duplicate)
        .observe(action::delete)
        .observe(action::focus)
        .observe(action::copy)
        .observe(action::paste)
        .observe(action::deselect)
        .observe(action::fullscreen)
        .observe(action::hide)
        .observe(action::load_paks)
        .run()
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
