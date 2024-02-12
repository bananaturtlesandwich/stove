#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity, clippy::too_many_arguments, clippy::unit_arg)]
use bevy::prelude::*;
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
mod ui;

type Asset = unreal_asset::Asset<std::io::BufReader<std::fs::File>>;

#[derive(Default)]
struct Map(Option<(Asset, std::path::PathBuf)>);

#[derive(Default)]
struct Transplant(Option<(Asset, Vec<actor::Actor>, Vec<usize>)>);

#[derive(Event)]
struct Notif {
    message: String,
    kind: egui_notify::ToastLevel,
}

#[derive(Event)]
enum Action {
    Duplicate,
    Delete,
    Focus,
    Copy,
    Paste,
}

#[derive(Event)]
enum Dialog {
    Open(Option<std::path::PathBuf>),
    SaveAs(bool),
    AddPak,
    Transplant,
}

#[derive(Default, Resource)]
struct Notifs(egui_notify::Toasts);

#[derive(Default, Resource)]
struct Registry(std::collections::BTreeMap<String, (Handle<Mesh>, Vec<Handle<StandardMaterial>>)>);

#[derive(Default, Resource)]
struct Focus(Option<Vec3>);

#[derive(Default, Resource)]
struct AppData {
    version: usize,
    paks: Vec<String>,
    aes: String,
    cache: bool,
    textures: bool,
    script: String,
    query: String,
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
    grid: Handle<StandardMaterial>,
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

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "stove".into(),
                    ..default()
                }),
                ..default()
            }),
            bevy_egui::EguiPlugin,
            smooth_bevy_cameras::LookTransformPlugin,
            smooth_bevy_cameras::controllers::unreal::UnrealCameraPlugin {
                override_input_system: true,
            },
            bevy_mod_raycast::deferred::DeferredRaycastingPlugin::<()>::default(),
            bevy_mod_outline::OutlinePlugin,
            bevy_mod_outline::AutoGenerateOutlineNormalsPlugin,
        ))
        .init_non_send_resource::<Map>()
        .init_non_send_resource::<Transplant>()
        .init_resource::<Notifs>()
        .init_resource::<Registry>()
        .init_resource::<Focus>()
        .init_resource::<Drag>()
        .init_resource::<Lock>()
        .init_resource::<Buffer>()
        .init_resource::<Client>()
        .add_event::<Notif>()
        .add_event::<Action>()
        .add_event::<Dialog>()
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
             mut dialog: EventWriter<Dialog>| {
                for drop in drops.read() {
                    if let bevy::window::FileDragAndDrop::DroppedFile { path_buf, .. } = drop {
                        dialog.send(Dialog::Open(Some(path_buf.clone())))
                    }
                }
            },
        )
        .add_systems(Update, ui::ui)
        .add_systems(Update, input::shortcuts)
        // post update because egui isn't built until update
        .add_systems(PostUpdate, picking::pick)
        .add_systems(PostUpdate, picking::drag.after(picking::pick))
        .add_systems(PostUpdate, input::camera)
        .add_systems(
            Update,
            |mut notif: EventReader<Notif>,
             mut notifs: ResMut<Notifs>,
             mut ctx: bevy_egui::EguiContexts| {
                for Notif { message, kind } in notif.read() {
                    notifs
                        .0
                        .add(egui_notify::Toast::custom(message, kind.clone()));
                }
                notifs.0.show(ctx.ctx_mut());
            },
        )
        .add_systems(Update, dialog::respond)
        .add_systems(Update, action::follow)
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
