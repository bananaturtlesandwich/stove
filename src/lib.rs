use std::rc::Rc;

#[cfg(not(target_family = "wasm"))]
use discord_rich_presence::{activity::*, DiscordIpc};
use eframe::egui;
use unreal_asset::{
    engine_version::EngineVersion::{self, *},
    types::PackageIndex,
};

mod actor;
mod asset;
mod extras;
mod rendering;

#[derive(PartialEq)]
enum Grab {
    None,
    // actor distance from camera
    Location(f32),
    Rotation,
    // actor screen coords
    Scale3D(glam::Vec2),
}

pub struct Stove {
    camera: rendering::Camera,
    notifs: egui_notify::Toasts,
    map: Option<unreal_asset::Asset<std::fs::File>>,
    version: usize,
    actors: Vec<actor::Actor>,
    selected: Vec<usize>,
    cubes: rendering::Cube,
    // axes: rendering::Axes,
    // meshes: hashbrown::HashMap<String, rendering::Mesh>,
    ui: bool,
    transplant: Option<(
        unreal_asset::Asset<std::fs::File>,
        Vec<actor::Actor>,
        Vec<usize>,
    )>,
    filepath: String,
    open_dialog: egui_file::FileDialog,
    transplant_dialog: egui_file::FileDialog,
    save_dialog: egui_file::FileDialog,
    pak_dialog: egui_file::FileDialog,
    last_mouse_pos: glam::Vec2,
    grab: Grab,
    paks: Vec<String>,
    distance: f32,
    fullscreen: bool,
    aes: String,
    use_cache: bool,
    script: String,
    locbuf: glam::DVec3,
    filter: glam::Vec3,
    #[cfg(not(target_family = "wasm"))]
    client: Option<discord_rich_presence::DiscordIpcClient>,
    #[cfg(not(target_family = "wasm"))]
    autoupdate: bool,
}

#[cfg(not(target_family = "wasm"))]
fn default_activity() -> Activity<'static> {
    Activity::new()
        .state("idle")
        .assets(Assets::new().large_image("pot"))
        .buttons(vec![Button::new(
            "homepage",
            "https://github.com/bananaturtlesandwich/stove",
        )])
}

#[cfg(target_os = "windows")]
const EXE: &str = "stove.exe";
#[cfg(target_os = "linux")]
const EXE: &str = "stove-linux";
#[cfg(target_os = "macos")]
const EXE: &str = "stove-macos";

#[cfg(not(target_family = "wasm"))]
fn auto_update() {
    let api = autoupdater::apis::github::GithubApi::new("bananaturtlesandwich", "stove")
        .current_version(env!("CARGO_PKG_VERSION"))
        .prerelease(true);
    if let Ok(Some(asset)) = api.get_newer(None::<autoupdater::Sort>) {
        use autoupdater::apis::DownloadApiTrait;
        if api
            .download(
                &asset
                    .assets
                    .into_iter()
                    .find(|asset| asset.name == EXE)
                    .unwrap(),
                None::<autoupdater::Download>,
            )
            .is_ok()
        {
            std::process::Command::new(EXE)
                .args(std::env::args().skip(1))
                .spawn()
                .unwrap();
            std::process::exit(0);
        }
    }
}

enum Wrapper {
    File(std::fs::File),
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
    dirs::config_local_dir().map(|path| path.join("stove"))
}

impl Stove {
    pub fn new(ctx: &eframe::CreationContext) -> Self {
        let Some(wgpu) = ctx.wgpu_render_state.as_ref() else { panic!("wgpu failed to initialise") };
        let mut notifs = egui_notify::Toasts::new();
        #[cfg(not(target_family = "wasm"))]
        if std::fs::remove_file(format!("{EXE}.old")).is_ok() {
            notifs.success(format!(
                "successfully updated to {}",
                env!("CARGO_PKG_VERSION")
            ));
        }
        let retrieve = |key: &str| ctx.storage.and_then(|storage| storage.get_string(key));
        let version = retrieve("VERSION")
            .and_then(|ver| ver.parse::<usize>().ok())
            .unwrap_or_default();
        let paks = retrieve("PAKS")
            .map(|paks| paks.split_inclusive(',').map(ToString::to_string).collect())
            .unwrap_or_default();
        let distance = retrieve("DIST")
            .and_then(|dist| dist.parse().ok())
            .unwrap_or(10000.0);
        let aes = retrieve("AES").unwrap_or_default();
        let use_cache = retrieve("CACHE")
            .and_then(|cache| cache.parse().ok())
            .unwrap_or(true);
        let script = retrieve("SCRIPT").unwrap_or_default();
        let autoupdate = retrieve("AUTOUPDATE")
            .and_then(|autoupdate| autoupdate.parse().ok())
            .unwrap_or(false);
        let mut home = dirs::home_dir();
        let mut filepath = String::new();
        let map = std::env::args().nth(1).and_then(|path| {
            match asset::open(&path, VERSIONS[version].0) {
                Ok(asset) => {
                    home = Some(
                        std::path::PathBuf::from(&path)
                            .parent()
                            .unwrap()
                            .to_path_buf(),
                    );
                    filepath = path;
                    Some(asset)
                }
                Err(e) => {
                    notifs.error(e.to_string());
                    None
                }
            }
        });
        #[cfg(not(target_family = "wasm"))]
        if autoupdate {
            std::thread::spawn(auto_update);
        }
        #[cfg(not(target_family = "wasm"))]
        let client = discord_rich_presence::DiscordIpcClient::new("1059578289737433249")
            .ok()
            .and_then(|mut cl| {
                (cl.connect().is_ok()
                    && cl
                        .set_activity(match filepath.as_str() {
                            "" => default_activity(),
                            name => default_activity()
                                .details("currently editing:")
                                .state(name.split('\\').last().unwrap_or_default()),
                        })
                        .is_ok())
                .then_some(cl)
            });

        let mut stove = Self {
            camera: rendering::Camera::default(),
            notifs,
            map,
            version,
            actors: Vec::new(),
            selected: Vec::new(),
            cubes: rendering::Cube::new(wgpu.device.as_ref()),
            // axes: rendering::Axes::new(ctx),
            // meshes: hashbrown::HashMap::new(),
            ui: true,
            transplant: None,
            open_dialog: egui_file::FileDialog::open_file(home.clone())
                .title("open map")
                .default_size((384.0, 256.0))
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .filter(Box::new(filter)),
            transplant_dialog: egui_file::FileDialog::open_file(home.clone())
                .title("transplant actor")
                .default_size((384.0, 256.0))
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .filter(Box::new(filter)),
            save_dialog: egui_file::FileDialog::save_file(home.clone())
                .title("save as")
                .default_size((384.0, 256.0))
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .filter(Box::new(filter)),
            pak_dialog: egui_file::FileDialog::select_folder(home)
                .title("add pak folder")
                .default_size((384.0, 256.0))
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0)),
            filepath,
            last_mouse_pos: glam::Vec2::ZERO,
            grab: Grab::None,
            paks,
            distance,
            fullscreen: false,
            aes,
            use_cache,
            script,
            locbuf: glam::DVec3::ZERO,
            filter: glam::Vec3::ONE,
            #[cfg(not(target_family = "wasm"))]
            client,
            #[cfg(not(target_family = "wasm"))]
            autoupdate,
        };
        stove.refresh();
        if stove.map.is_none() {
            stove.open_dialog.open()
        }
        stove
    }

    fn version(&self) -> EngineVersion {
        VERSIONS[self.version].0
    }

    fn refresh(&mut self) {
        self.actors.clear();
        self.selected.clear();
        let Some(map) = self.map.as_ref() else {return};
        // let key = match hex::decode(self.aes.trim_start_matches("0x")) {
        //     Ok(key) if !self.aes.is_empty() => Some(key),
        //     Ok(_) => None,
        //     Err(_) => {
        //         self.notifs.error("aes key is invalid hex");
        //         None
        //     }
        // };
        // let paks: Vec<_> = self
        //     .paks
        //     .iter()
        //     .filter_map(|dir| std::fs::read_dir(dir).ok())
        //     .flatten()
        //     .filter_map(Result::ok)
        //     .map(|dir| dir.path())
        //     .filter_map(|path| unpak::Pak::new_any(path, key.as_deref()).ok())
        //     .collect();
        // let cache = config().map(|path| path.join("cache"));
        for index in actor::get_actors(map) {
            match actor::Actor::new(map, index) {
                Ok(actor) => {
                    // if let actor::DrawType::Mesh(path) = &actor.draw_type {
                    //     if !self.meshes.contains_key(path) {
                    //         let (mesh, bulk) =
                    //             (path.to_string() + ".uasset", path.to_string() + ".uexp");
                    //         let mesh_path = mesh.trim_start_matches('/');
                    //         for pak in paks.iter() {
                    //             let info = match cache.as_ref() {
                    //                 Some(cache)
                    //                     if self.use_cache
                    //                         && (cache.join(mesh_path).exists() ||
                    //                     // try to create cache if it doesn't exist
                    //                     (
                    //                         std::fs::create_dir_all(cache.join(mesh_path).parent().unwrap()).is_ok() &&
                    //                         pak.read_to_file(&mesh, cache.join(mesh_path)).is_ok() &&
                    //                         // we don't care whether this is successful in case there is no uexp
                    //                         pak.read_to_file(&bulk, cache.join(bulk.trim_start_matches('/'))).map_or(true,|_| true)
                    //                     )) =>
                    //                 {
                    //                     let Ok(mesh) =
                    //                         asset::open(cache.join(mesh_path), self.version())
                    //                     else {
                    //                         continue;
                    //                     };
                    //                     extras::get_mesh_info(mesh)
                    //                 }
                    //                 // if the cache cannot be created fall back to storing in memory
                    //                 _ => {
                    //                     let Ok(mesh) = pak.get(&mesh) else { continue };
                    //                     let Ok(mesh) = unreal_asset::Asset::new(
                    //                         std::io::Cursor::new(mesh),
                    //                         pak.get(&bulk).ok().map(std::io::Cursor::new),
                    //                         self.version(),
                    //                         None
                    //                     ) else {
                    //                         continue;
                    //                     };
                    //                     extras::get_mesh_info(mesh)
                    //                 }
                    //             };
                    //             match info {
                    //                 Ok((positions, indices)) => {
                    //                     self.meshes.insert(
                    //                         path.to_string(),
                    //                         rendering::Mesh::new(ctx, positions, indices),
                    //                     );
                    //                     break;
                    //                 }
                    //                 Err(e) => {
                    //                     self.notifs.error(format!(
                    //                         "{}: {e}",
                    //                         path.split('/').last().unwrap_or_default()
                    //                     ));
                    //                 }
                    //             }
                    //         }
                    //     }
                    // }
                    self.actors.push(actor);
                }
                Err(e) => {
                    self.notifs.warning(e.to_string());
                }
            }
        }
    }

    // fn refresh(&mut self, ctx: &mut Context) {
    //     self.actors.clear();
    //     self.selected.clear();
    //     if let Some(map) = self.map.as_ref() {
    //         let key = match hex::decode(self.aes.trim_start_matches("0x")) {
    //             Ok(key) if !self.aes.is_empty() => Some(key),
    //             Ok(_) => None,
    //             Err(_) => {
    //                 self.notifs.error("aes key is invalid hex");
    //                 None
    //             }
    //         };
    //         let paks: Vec<_> = self
    //             .paks
    //             .iter()
    //             .filter_map(|dir| std::fs::read_dir(dir).ok())
    //             .flatten()
    //             .filter_map(Result::ok)
    //             .map(|dir| dir.path())
    //             .filter_map(|path| unpak::Pak::new_any(path, key.as_deref()).ok())
    //             .collect();
    //         let cache = config()
    //             .filter(|_| self.use_cache)
    //             .map(|path| path.join("cache"));
    //         let version = self.version();
    //         for index in actor::get_actors(map) {
    //             match actor::Actor::new(map, index) {
    //                 Ok(actor) => {
    //                     if let actor::DrawType::Mesh(path) = &actor.draw_type {
    //                         if !self.meshes.contains_key(path) {
    //                             fn get<T>(
    //                                 pak: &unpak::Pak,
    //                                 cache: Option<&std::path::Path>,
    //                                 path: &str,
    //                                 version: EngineVersion,
    //                                 func: impl Fn(
    //                                     unreal_asset::Asset<Wrapper>,
    //                                     Option<Wrapper>,
    //                                 )
    //                                     -> Result<T, unreal_asset::error::Error>,
    //                             ) -> Result<T, unreal_asset::error::Error>
    //                             {
    //                                 let make = |ext: &str| path.to_string() + ext;
    //                                 let (mesh, exp, bulk, uptnl) = (
    //                                     make(".uasset"),
    //                                     make(".uexp"),
    //                                     make(".ubulk"),
    //                                     make(".uptnl"),
    //                                 );
    //                                 let cache_path = |path: &str| {
    //                                     cache.unwrap().join(path.trim_start_matches('/'))
    //                                 };
    //                                 match cache {
    //                                     Some(cache)
    //                                         if cache.join(&path).exists() ||
    //                                         // try to create cache if it doesn't exist
    //                                         (
    //                                             std::fs::create_dir_all(cache_path(&path).parent().unwrap()).is_ok() &&
    //                                             pak.read_to_file(&mesh, cache_path(&path)).is_ok() &&
    //                                             // we don't care whether these are successful in case they don't exist
    //                                             pak.read_to_file(&exp, cache_path(&exp)).map_or(true,|_| true) &&
    //                                             pak.read_to_file(&bulk, cache_path(&bulk)).map_or(true,|_| true) &&
    //                                             pak.read_to_file(&uptnl, cache_path(&uptnl)).map_or(true,|_| true)
    //                                         ) =>
    //                                     {
    //                                         func(
    //                                             unreal_asset::Asset::new(
    //                                                 Wrapper::File(std::fs::File::open(
    //                                                     cache_path(&mesh),
    //                                                 )?),
    //                                                 std::fs::File::open(cache_path(&exp))
    //                                                     .ok()
    //                                                     .map(Wrapper::File),
    //                                                 version,
    //                                                 None,
    //                                             )?,
    //                                             std::fs::File::open(cache_path(&bulk))
    //                                                 .ok()
    //                                                 .map_or_else(
    //                                                     || {
    //                                                         std::fs::File::open(cache_path(&uptnl))
    //                                                             .ok()
    //                                                     },
    //                                                     Some,
    //                                                 )
    //                                                 .map(Wrapper::File),
    //                                         )
    //                                     }
    //                                     // if the cache cannot be created fall back to storing in memory
    //                                     _ => func(
    //                                         unreal_asset::Asset::new(
    //                                             Wrapper::Bytes(std::io::Cursor::new(
    //                                                 pak.get(&mesh).map_err(|_| {
    //                                                     unreal_asset::error::Error::no_data(
    //                                                         "files not found".to_string(),
    //                                                     )
    //                                                 })?,
    //                                             )),
    //                                             pak.get(&exp)
    //                                                 .ok()
    //                                                 .map(std::io::Cursor::new)
    //                                                 .map(Wrapper::Bytes),
    //                                             version,
    //                                             None,
    //                                         )?,
    //                                         pak.get(&bulk)
    //                                             .ok()
    //                                             .map_or_else(|| pak.get(&uptnl).ok(), Some)
    //                                             .map(std::io::Cursor::new)
    //                                             .map(Wrapper::Bytes),
    //                                     ),
    //                                 }
    //                             }
    //                             for pak in paks.iter() {
    //                                 // currently doesn't read bulk data but we'll get there
    //                                 match get(pak, cache.as_deref(), path, version, |asset, _| {
    //                                     Ok(extras::get_mesh_info(asset)?)
    //                                 }) {
    //                                     Ok((positions, indices, uvs, mats, mat_data)) => {
    //                                         let mats = mats
    //                                             .into_iter()
    //                                             .map(|path| {
    //                                                 match get(
    //                                                     pak,
    //                                                     cache.as_deref(),
    //                                                     &path,
    //                                                     version,
    //                                                     |mat, _| Ok(extras::get_tex_path(mat)),
    //                                                 ) {
    //                                                     Ok(Some(path)) => match get(
    //                                                         pak,
    //                                                         cache.as_deref(),
    //                                                         &path,
    //                                                         version,
    //                                                         |tex, bulk| {
    //                                                             Ok(extras::get_tex_info(tex, bulk)?)
    //                                                         },
    //                                                     ) {
    //                                                         Ok(o) => o,
    //                                                         Err(e) => {
    //                                                             self.notifs.warning(format!(
    //                                                                 "{}: {e}",
    //                                                                 path.split('/')
    //                                                                     .last()
    //                                                                     .unwrap_or_default()
    //                                                             ));
    //                                                             (1, 1, vec![255, 50, 125, 255])
    //                                                         }
    //                                                     },
    //                                                     _ => (1, 1, vec![125, 50, 255, 255]),
    //                                                 }
    //                                             })
    //                                             .collect();
    //                                         self.meshes.insert(
    //                                             path.to_string(),
    //                                             rendering::Mesh::new(
    //                                                 ctx, positions, indices, uvs, mats, mat_data,
    //                                             ),
    //                                         );
    //                                         break;
    //                                     }
    //                                     Err(e) => {
    //                                         self.notifs.error(format!(
    //                                             "{}: {e}",
    //                                             path.split('/').last().unwrap_or_default()
    //                                         ));
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     self.actors.push(actor);
    //                 }
    //                 Err(e) => {
    //                     self.notifs.warning(e.to_string());
    //                 }
    //             }
    //         }
    //     }
    // }

    fn save(&mut self) {
        match self.map.as_mut() {
            Some(map) => match asset::save(map, &self.filepath) {
                Ok(_) => self.notifs.success("map saved"),
                Err(e) => self.notifs.error(e.to_string()),
            },
            None => self.notifs.error("no map to save"),
        };
        // literally no idea why std::process::Command doesn't work
        #[cfg(target_os = "windows")]
        const PATH: &str = "./script.bat";
        #[cfg(not(target_os = "windows"))]
        const PATH: &str = "./script.sh";
        for line in self.script.lines() {
            if let Err(e) = std::fs::write(PATH, line) {
                self.notifs
                    .error(format!("failed to make save script: {e}"));
            }
            match std::process::Command::new(PATH)
                .stdout(std::process::Stdio::piped())
                .output()
            {
                Ok(out) => self
                    .notifs
                    .success(String::from_utf8(out.stdout).unwrap_or_default()),
                Err(e) => self.notifs.error(format!("failed to run save script: {e}")),
            };
        }
        if !self.script.is_empty() {
            if let Err(e) = std::fs::remove_file(PATH) {
                self.notifs
                    .error(format!("failed to remove save script: {e}"));
            }
        }
    }

    fn open_save_dialog(&mut self) {
        match self.map.is_some() {
            true => self.try_open(|stove| &mut stove.save_dialog),
            false => {
                self.notifs.error("no map to save");
            }
        }
    }

    fn view_projection(&self, ctx: &eframe::Frame) -> glam::Mat4 {
        let size = ctx.info().window_info.size;
        glam::Mat4::perspective_lh(45_f32.to_radians(), size.x / size.y, 1.0, self.distance)
            * self.camera.view_matrix()
    }

    fn focus(&mut self) {
        if self.selected.is_empty() {
            self.notifs.error("nothing selected to focus on");
        }
        let Some((pos, sca)) = self.avg_transform() else { return };
        self.camera.set_focus(pos, sca)
    }

    fn try_open(&mut self, dialog: impl Fn(&mut Self) -> &mut egui_file::FileDialog) {
        macro_rules! open {
            ($dialog: ident) => {
                self.$dialog.state() != egui_file::State::Open
            };
        }
        match open!(open_dialog)
            && open!(save_dialog)
            && open!(pak_dialog)
            && open!(transplant_dialog)
            && self.transplant.is_none()
        {
            true => dialog(self).open(),
            false => {
                self.notifs.error("another dialog is currently open");
            }
        }
    }

    fn get_avg_base<T>(
        &self,
        get: impl Fn(&actor::Actor, &unreal_asset::Asset<std::fs::File>) -> T,
        add: impl Fn(T, T) -> T,
        div: impl Fn(T, f32) -> T,
    ) -> Option<T> {
        let map = self.map.as_ref()?;
        let len = self.selected.len() as f32;
        self.selected
            .iter()
            .map(|&i| get(&self.actors[i], map))
            .reduce(add)
            .map(|acc| div(acc, len))
    }

    fn get_avg<T: std::ops::Add<T, Output = T> + std::ops::Div<f32, Output = T>>(
        &self,
        get: impl Fn(&actor::Actor, &unreal_asset::Asset<std::fs::File>) -> T,
    ) -> Option<T> {
        self.get_avg_base(get, |a, b| a + b, |a, b| a / b)
    }

    fn avg_raw_loc(&self) -> Option<glam::DVec3> {
        self.get_avg_base(
            |actor, map| actor.get_raw_location(map),
            |a, b| a + b,
            |a, b| a / b as f64,
        )
    }

    fn avg_transform(&self) -> Option<(glam::Vec3, glam::Vec3)> {
        self.get_avg_base(
            |actor, map| (actor.location(map), actor.scale(map)),
            |(accpos, accsca), (pos, sca)| ((accpos + pos), (accsca + sca)),
            |(pos, sca), len| {
                let len = len as f32;
                (pos / len, sca / len)
            },
        )
    }
}

fn update_dialogs(path: &std::path::Path, dialogs: [&mut egui_file::FileDialog; 3]) {
    let mut path = path.to_path_buf();
    if path.is_file() {
        path.pop();
    }
    for dialog in dialogs {
        dialog.set_path(path.as_path())
    }
}

fn filter(path: &std::path::Path) -> bool {
    path.extension().and_then(std::ffi::OsStr::to_str) == Some("umap")
}

fn select(ui: &mut egui::Ui, selected: &mut Vec<usize>, i: usize) {
    ui.input(
        |input| match selected.iter().position(|entry| entry == &i) {
            Some(i) => {
                selected.remove(i);
            }
            None if input.modifiers.shift && selected.last().is_some_and(|last| last != &i) => {
                let last_selected = *selected.last().unwrap();
                for i in match i < last_selected {
                    true => i..last_selected,
                    false => last_selected + 1..i + 1,
                } {
                    selected.push(i)
                }
            }
            _ => selected.push(i),
        },
    )
}

impl eframe::App for Stove {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        use eframe::wgpu::*;
        self.camera.update_times();
        ctx.input(|input| self.handle_input(input));
        if let Some((map, wgpu)) = self.map.as_ref().zip(frame.wgpu_render_state()) {
            let vp = self.view_projection(frame);
            let mut encoder = wgpu
                .device
                .create_command_encoder(&CommandEncoderDescriptor { label: None });
            self.cubes.copy(
                self.actors
                    .iter()
                    .enumerate()
                    .map(|(i, actor)| {
                        (
                            actor.model_matrix(map),
                            self.selected.contains(&i) as i32 as f32,
                        )
                    })
                    .unzip(),
                &[vp],
                &wgpu.queue,
            );
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: None,
            });
            self.cubes.draw(&mut pass);
            // for (actor, mesh) in self
            //     .actors
            //     .iter()
            //     .filter_map(|actor| match &actor.draw_type {
            //         actor::DrawType::Mesh(key) => self.meshes.get(key).map(|mesh| (actor, mesh)),
            //         actor::DrawType::Cube => None,
            //     })
            // {
            //     mesh.draw(mqctx, vp * actor.model_matrix(map));
            // }
            // if !self.selected.is_empty() {
            //     if self.grab != Grab::None {
            //         if let Some((loc, sca)) = self.avg_transform() {
            //             self.axes.draw(
            //                 mqctx,
            //                 &self.filter,
            //                 vp * glam::Mat4::from_translation(loc) * glam::Mat4::from_scale(sca),
            //             )
            //         }
            //     }
            // }
            drop(pass);
            wgpu.queue.submit(Some(encoder.finish()));
        }
        egui::SidePanel::left("sidepanel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("file", |ui| {
                    if ui.add(egui::Button::new("open").shortcut_text("ctrl + o")).clicked() {
                        self.try_open(|stove| &mut stove.open_dialog)
                    }
                    if ui.add(egui::Button::new("save").shortcut_text("ctrl + s")).clicked(){
                        self.save()
                    }
                    if ui.add(egui::Button::new("save as").shortcut_text("ctrl + shift + s")).clicked(){
                        self.open_save_dialog()
                    }
                });
                egui::ComboBox::from_id_source("version")
                    .show_index(ui, &mut self.version, 33, |i| VERSIONS[i].1.to_string());
                ui.menu_button("paks", |ui| {
                    egui::TextEdit::singleline(&mut self.aes)
                        .clip_text(false)
                        .hint_text("aes key if needed")
                        .show(ui);
                    let mut remove_at = None;
                    egui::ScrollArea::vertical().show_rows(
                        ui,
                        ui.text_style_height(&egui::TextStyle::Body),
                        self.paks.len(),
                        |ui, range| for i in range {
                            ui.horizontal(|ui| {
                                ui.label(&self.paks[i]);
                                if ui.button("x").clicked(){
                                    remove_at = Some(i);
                                }
                            });
                        }
                    );
                    if let Some(i) = remove_at {
                        self.paks.remove(i);
                    }
                    if ui.add(egui::Button::new("add pak folder").shortcut_text("alt + o")).clicked() {
                        self.try_open(|stove| &mut stove.pak_dialog);
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
                        ui.label("autoupdate:");
                        ui.add(egui::Checkbox::without_text(&mut self.autoupdate));
                    });
                    ui.horizontal(|ui|{
                        ui.label("cache meshes:");
                        ui.add(egui::Checkbox::without_text(&mut self.use_cache));
                    });
                    ui.horizontal(|ui| {
                        ui.label("render distance:");
                        ui.add(
                            egui::widgets::DragValue::new(&mut self.distance)
                                .clamp_range(0..=100000)
                        )
                    });
                    ui.label("post-save commands");
                    ui.text_edit_multiline(&mut self.script);
                });
            });
            if let Some(map) = self.map.as_mut() {
                ui.add_space(10.0);
                ui.push_id("actors", |ui| egui::ScrollArea::both()
                    .auto_shrink([false, true])
                    .max_height(ui.available_height() * 0.5)
                    .show_rows(
                        ui,
                        ui.text_style_height(&egui::TextStyle::Body),
                        self.actors.len(),
                        |ui, range|{
                            ui.with_layout(egui::Layout::default().with_cross_justify(true), |ui|
                                for i in range {
                                    let is_selected = self.selected.contains(&i);
                                    if ui.selectable_label(
                                        is_selected,
                                        &self.actors[i].name
                                    )
                                    .on_hover_text(&self.actors[i].class)
                                    .clicked() {
                                        ui.input(|state| if !state.modifiers.shift && !state.modifiers.ctrl{
                                            self.selected.clear()
                                        });
                                        select(ui, &mut self.selected, i);
                                    }
                                }
                            )
                        ;
                    })
                );
                if let Some(&selected) = self.selected.last() {
                    ui.add_space(10.0);
                    ui.push_id("properties", |ui| egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show(ui,|ui| {
                            self.actors[selected].show(map, ui);
                            // otherwise the scroll area bugs out at the bottom
                            ui.add_space(1.0);
                        })
                    );
                }
            }
        });
        let mut open = true;
        let mut transplanted = None;
        if let Some((map, (donor, actors, selected))) =
            self.map.as_mut().zip(self.transplant.as_mut())
        {
            egui::Window::new("transplant actor")
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .resizable(false)
                .collapsible(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    // putting the button below breaks the scroll area somehow
                    ui.add_enabled_ui(!selected.is_empty(), |ui| {
                        if ui
                            .vertical_centered_justified(|ui| ui.button("transplant selected"))
                            .inner
                            .clicked()
                        {
                            transplanted = Some(map.asset_data.exports.len()..selected.len());
                            for actor in selected.iter().map(|i| &actors[*i]) {
                                let insert = map.asset_data.exports.len() as i32 + 1;
                                actor.transplant(map, donor);
                                self.actors.push(
                                    actor::Actor::new(map, PackageIndex::new(insert)).unwrap(),
                                );
                                self.notifs.success(format!("transplanted {}", actor.name));
                            }
                        }
                    });
                    egui::ScrollArea::both().auto_shrink([false; 2]).show_rows(
                        ui,
                        ui.text_style_height(&egui::TextStyle::Body),
                        actors.len(),
                        |ui, range| {
                            ui.with_layout(egui::Layout::default().with_cross_justify(true), |ui| {
                                for (i, actor) in range.clone().zip(actors[range].iter()) {
                                    if ui
                                        .selectable_label(selected.contains(&i), &actor.name)
                                        .on_hover_text(&actor.class)
                                        .clicked()
                                    {
                                        select(ui, selected, i)
                                    }
                                }
                            })
                        },
                    );
                });
        }
        if let Some(len) = transplanted.as_mut() {
            self.selected.extend(len);
            self.focus();
            self.transplant = None;
        }
        if !open {
            self.transplant = None;
        }
        self.notifs.show(ctx);
        self.open_dialog.show(ctx);
        if self.open_dialog.selected() {
            if let Some(path) = self.open_dialog.path() {
                update_dialogs(
                    path,
                    [
                        &mut self.save_dialog,
                        &mut self.pak_dialog,
                        &mut self.transplant_dialog,
                    ],
                );
                match asset::open(path, self.version()) {
                    Ok(asset) => {
                        self.filepath = path.to_str().unwrap_or_default().to_string();
                        #[cfg(not(target_family = "wasm"))]
                        if let Some(client) = self.client.as_mut() {
                            if client
                                .set_activity(
                                    default_activity().details("currently editing:").state(
                                        self.filepath.split('\\').last().unwrap_or_default(),
                                    ),
                                )
                                .is_err()
                            {
                                client.close().unwrap_or_default();
                                self.client = None;
                            }
                        }
                        self.map = Some(asset);
                        self.refresh();
                    }
                    Err(e) => {
                        self.notifs.error(e.to_string());
                    }
                }
            }
        }
        self.transplant_dialog.show(ctx);
        if self.transplant_dialog.selected() {
            if let Some(path) = self.transplant_dialog.path() {
                update_dialogs(
                    path,
                    [
                        &mut self.open_dialog,
                        &mut self.save_dialog,
                        &mut self.pak_dialog,
                    ],
                );
                match asset::open(path, self.version()) {
                    Ok(donor) => {
                        // no need for verbose warnings here
                        let actors: Vec<_> = actor::get_actors(&donor)
                            .into_iter()
                            .filter_map(|index| actor::Actor::new(&donor, index).ok())
                            .collect();
                        let selected = Vec::with_capacity(actors.len());
                        self.transplant = Some((donor, actors, selected));
                    }
                    Err(e) => {
                        self.notifs.error(e.to_string());
                    }
                }
            }
        }
        self.pak_dialog.show(ctx);
        if self.pak_dialog.selected() {
            if let Some(path) = self.pak_dialog.path() {
                update_dialogs(
                    path,
                    [
                        &mut self.open_dialog,
                        &mut self.save_dialog,
                        &mut self.transplant_dialog,
                    ],
                );
                if let Some(path) = path.to_str().map(str::to_string) {
                    self.paks.push(path);
                }
            }
        }
        self.save_dialog.show(ctx);
        if self.save_dialog.selected() {
            if let Some(path) = self.save_dialog.path() {
                update_dialogs(
                    path,
                    [
                        &mut self.open_dialog,
                        &mut self.pak_dialog,
                        &mut self.transplant_dialog,
                    ],
                );
                self.filepath = path
                    .with_extension("umap")
                    .to_str()
                    .unwrap_or_default()
                    .to_string();
                self.save()
            }
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("VERSION", self.version.to_string());
        storage.set_string(
            "PAKS",
            self.paks
                .iter()
                .cloned()
                .reduce(|acc, pak| acc + "," + &pak)
                .unwrap_or_default(),
        );
        storage.set_string("DIST", self.distance.to_string());
        storage.set_string("AES", self.aes.clone());
        storage.set_string("CACHE", self.use_cache.to_string());
        storage.set_string("SCRIPT", self.script.clone());
        storage.set_string("AUTOUPDATE", self.autoupdate.to_string());
    }

    #[cfg(not(target_family = "wasm"))]
    fn on_close_event(&mut self) -> bool {
        if let Some(client) = &mut self.client {
            return client.close().is_ok();
        }
        true
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.15, 0.15, 0.15, 1.0]
    }
}

//  impl  Even tHandler for Stove {
//     fn update(&mut self, _: &mut Context) {
//         self.camera.update_times();
//         self.camera.move_cam(&self.held)
//     }

//     fn draw(&mut self, mqctx: &mut Context) {
//         mqctx.begin_default_pass(PassAction::Clear {
//             color: Some((0.15, 0.15, 0.15, 1.0)),
//             depth: Some(1.0),
//             stencil: None,
//         });
//         mqctx.end_render_pass();
//         if !self.ui {
//             mqctx.commit_frame();
//             return;
//         }
//         egui().draw(mqctx);
//         mqctx.commit_frame();
//     }

impl Stove {
    fn handle_input(&mut self, input: &egui::InputState) {
        self.camera.move_cam(input);
    }
}

//     fn mouse_motion_event(&mut self, _: &mut Context, x: f32, y: f32) {
//         egui().mouse_motion_event(x, y);
//         let delta = glam::vec2(x - self.last_mouse_pos.x, y - self.last_mouse_pos.y);
//         self.camera.handle_mouse_motion(delta);
//         for i in self.selected.iter().copied() {
//             match self.grab {
//                 Grab::None => (),
//                 Grab::Location(dist) => self.actors[i].add_location(
//                     self.map.as_mut().unwrap(),
//                     self.filter
//                         // move across the camera view plane
//                         * (
//                             self.camera.left()
//                             * -delta.x
//                             + self.camera.front.cross(self.camera.left())
//                             * delta.y
//                         )
//                         // scale by consistent distance
//                         * dist
//                         // scale to match mouse cursor
//                         * 0.1,
//                 ),
//                 Grab::Rotation => self.actors[i].combine_rotation(
//                     self.map.as_mut().unwrap(),
//                     glam::Quat::from_axis_angle(
//                         self.filter * self.camera.front,
//                         match delta.x.abs() > delta.y.abs() {
//                             true => -delta.x,
//                             false => delta.y,
//                         } * 0.01,
//                     ),
//                 ),
//                 Grab::Scale3D(coords) => self.actors[i].mul_scale(
//                     self.map.as_mut().unwrap(),
//                     glam::Vec3::ONE
//                         + self.filter
//                             * (coords.distance(glam::vec2(x, y))
//                                 - coords.distance(self.last_mouse_pos))
//                             * 0.005,
//                 ),
//             }
//         }
//         self.last_mouse_pos = glam::vec2(x, y);
//     }

//     fn mouse_wheel_event(&mut self, _: &mut Context, dx: f32, dy: f32) {
//         egui().mouse_wheel_event(dx, dy);
//         // a logarithmic speed increase is better because unreal maps can get massive
//         if !egui().egui_ctx().is_pointer_over_area() {
//             self.camera.speed = (self.camera.speed as f32
//                 * match dy.is_sign_negative() {
//                     true => 100.0 / -dy,
//                     false => dy / 100.0,
//                 })
//             .clamp(5.0, 50000.0) as u16;
//         }
//     }

//     fn mouse_button_down_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
//         egui().mouse_button_down_event(ctx, mb, x, y);
//         if egui().egui_ctx().is_pointer_over_area() {
//             return;
//         }
//         let proj = self.view_projection(ctx);
//         // THE HACKIEST MOUSE PICKING EVER CONCEIVED
//         let pick = self
//             .map
//             .as_ref()
//             .and_then(|map| {
//                 // convert mouse coordinates to NDC
//                 let (width, height) = ctx.screen_size();
//                 let mouse = glam::vec2(x * 2.0 / width - 1.0, 1.0 - y * 2.0 / height);
//                 self.actors
//                     .iter()
//                     .map(|actor| mouse.distance(actor.coords(map, proj)))
//                     .enumerate()
//                     // get closest pick
//                     .min_by(|(_, x), (_, y)| x.total_cmp(y))
//             })
//             .and_then(|(pos, distance)| (distance < 0.05).then_some(pos));
//         match pick {
//             // grabby time
//             Some(pick) if self.selected.contains(&pick) => {
//                 if let Some(map) = self.map.as_mut() {
//                     if self.held.contains(&KeyCode::LeftAlt)
//                         || self.held.contains(&KeyCode::RightAlt)
//                     {
//                         let insert = self.actors.len();
//                         for i in self.selected.iter().copied() {
//                             let insert = map.asset_data.exports.len() as i32 + 1;
//                             self.actors[i].duplicate(map);
//                             self.notifs
//                                 .success(format!("duplicated {}", &self.actors[i].name));
//                             self.actors
//                                 .push(actor::Actor::new(map, PackageIndex::new(insert)).unwrap());
//                         }
//                         let len = self.actors.len();
//                         self.selected.clear();
//                         for i in insert..len {
//                             self.selected.push(i);
//                         }
//                     }
//                     self.grab = match mb {
//                         MouseButton::Left => Grab::Location(
//                             self.get_avg(|actor, map| actor.location(map))
//                                 .unwrap()
//                                 .distance(self.camera.position),
//                         ),
//                         MouseButton::Right => Grab::Rotation,
//                         MouseButton::Middle => Grab::Scale3D({
//                             let (width, height) = ctx.screen_size();
//                             let (x, y) = self
//                                 .get_avg(|actor, map| actor.coords(map, proj))
//                                 .unwrap()
//                                 .into();
//                             glam::vec2((x + 1.0) * width * 0.5, (1.0 - y) * height * 0.5)
//                         }),
//                         MouseButton::Unknown => Grab::None,
//                     };
//                 }
//             }
//             Some(pick) if mb == MouseButton::Left => {
//                 if !self.held.contains(&KeyCode::LeftShift)
//                     && !self.held.contains(&KeyCode::RightShift)
//                 {
//                     self.selected.clear()
//                 }
//                 self.selected.push(pick)
//             }
//             None if mb == MouseButton::Left => self.selected.clear(),
//             _ if mb == MouseButton::Right => self.camera.enable_move(),
//             _ => (),
//         }
//     }

//     fn mouse_button_up_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
//         egui().mouse_button_up_event(ctx, mb, x, y);
//         if mb == MouseButton::Right {
//             self.camera.disable_move()
//         }
//         self.grab = Grab::None;
//     }

//     // boilerplate >n<
//     fn char_event(&mut self, _: &mut Context, character: char, _: KeyMods, _: bool) {
//         egui().char_event(character);
//     }

//     fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods, _: bool) {
//         egui().key_down_event(ctx, keycode, keymods);
//         if egui().egui_ctx().is_pointer_over_area() || keymods.ctrl || self.held.contains(&keycode)
//         {
//             return;
//         }
//         self.filter = match keycode {
//             KeyCode::X if keymods.shift => glam::vec3(0., 1., 1.),
//             KeyCode::Y if keymods.shift => glam::vec3(1., 0., 1.),
//             KeyCode::Z if keymods.shift => glam::vec3(1., 1., 0.),
//             KeyCode::X => glam::Vec3::X,
//             KeyCode::Y => glam::Vec3::Y,
//             KeyCode::Z => glam::Vec3::Z,
//             _ => glam::Vec3::ONE,
//         };
//         self.held.push(keycode)
//     }

//     fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods) {
//         egui().key_up_event(keycode, keymods);
//         if let Some(pos) = self.held.iter().position(|k| k == &keycode) {
//             self.held.remove(pos);
//         }
//         if egui().egui_ctx().wants_keyboard_input() {
//             return;
//         }
//         match keycode {
//             KeyCode::Delete => match self.selected.is_empty() {
//                 false => {
//                     self.selected
//                         .sort_unstable_by_key(|key| std::cmp::Reverse(*key));
//                     for i in self.selected.iter().copied() {
//                         self.actors[i].delete(self.map.as_mut().unwrap());
//                         self.notifs
//                             .success(format!("deleted {}", &self.actors[i].name));
//                         self.actors.remove(i);
//                     }
//                     self.selected.clear();
//                 }
//                 true => {
//                     self.notifs.error("nothing selected to delete");
//                 }
//             },
//             KeyCode::F => self.focus(),
//             KeyCode::H => self.ui = !self.ui,
//             KeyCode::T if keymods.ctrl => match self.map.is_some() {
//                 true => self.try_open(|stove| &mut stove.transplant_dialog),
//                 false => {
//                     self.notifs.error("no map to transplant to");
//                 }
//             },
//             KeyCode::O if keymods.ctrl => self.try_open(|stove| &mut stove.open_dialog),
//             KeyCode::O if keymods.alt => self.try_open(|stove| &mut stove.pak_dialog),
//             KeyCode::S if keymods.ctrl => match keymods.shift {
//                 true => self.open_save_dialog(),
//                 false => self.save(),
//             },
//             KeyCode::Enter if keymods.alt => {
//                 if !self.fullscreen {
//                     self.size = ctx.screen_size();
//                 }
//                 self.fullscreen = !self.fullscreen;
//                 ctx.set_fullscreen(self.fullscreen);
//                 if !self.fullscreen {
//                     ctx.set_window_size(self.size.0 as u32, self.size.1 as u32)
//                 }
//             }
//             KeyCode::C if keymods.ctrl => {
//                 match self.avg_raw_loc() {
//                     Some(pos) => {
//                         self.locbuf = pos;
//                         self.notifs.success("location copied")
//                     }
//                     None => self.notifs.error("no actor selected to copy from"),
//                 };
//             }
//             KeyCode::V if keymods.ctrl => {
//                 match self.avg_raw_loc().zip(self.map.as_mut()) {
//                     Some((pos, map)) => {
//                         let offset = (pos - self.locbuf).abs();
//                         for i in self.selected.iter().copied() {
//                             self.actors[i].add_raw_location(map, offset)
//                         }
//                         self.notifs.success("location pasted")
//                     }
//                     None => self.notifs.error("no actor selected to copy from"),
//                 };
//             }
//             KeyCode::X | KeyCode::Y | KeyCode::Z => self.filter = glam::Vec3::ONE,
//             _ => (),
//         }
//     }
// }

const VERSIONS: [(EngineVersion, &str); 33] = [
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
