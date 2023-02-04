#[cfg(not(target_family = "wasm"))]
use discord_rich_presence::{activity::*, DiscordIpc};
use miniquad::*;
use unreal_asset::{
    engine_version::EngineVersion::{self, *},
    types::PackageIndex,
};

mod actor;
mod asset;
mod extras;
mod rendering;

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
    map: Option<unreal_asset::Asset>,
    version: unreal_asset::engine_version::EngineVersion,
    egui: egui_miniquad::EguiMq,
    actors: Vec<actor::Actor>,
    selected: Option<usize>,
    cubes: rendering::Cube,
    meshes: hashbrown::HashMap<String, rendering::Mesh>,
    ui: bool,
    donor: Option<(unreal_asset::Asset, Vec<actor::Actor>)>,
    filepath: String,
    open_dialog: egui_file::FileDialog,
    transplant_dialog: egui_file::FileDialog,
    save_dialog: egui_file::FileDialog,
    pak_dialog: egui_file::FileDialog,
    held: Vec<KeyCode>,
    last_mouse_pos: glam::Vec2,
    grab: Grab,
    paks: Vec<String>,
    #[cfg(not(target_family = "wasm"))]
    client: Option<discord_rich_presence::DiscordIpcClient>,
}

fn home_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_family = "wasm")]
    return None;
    #[cfg(not(target_family = "wasm"))]
    std::env::var_os(
        #[cfg(target_family = "windows")]
        "USERPROFILE",
        #[cfg(target_family = "unix")]
        "HOME",
    )
    .filter(|home| !home.is_empty())
    .map(std::path::PathBuf::from)
}

fn config() -> Option<std::path::PathBuf> {
    home_dir().map(|path| {
        path.join(
            #[cfg(target_family = "windows")]
            "AppData/Local/stove",
            #[cfg(target_family = "unix")]
            ".config/stove",
        )
    })
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

macro_rules! refresh {
    ($self: expr, $ctx: expr) => {
        $self.actors.clear();
        $self.meshes.clear();
        let mut paks: Vec<_> = $self
            .paks
            .iter()
            .filter_map(|dir| std::fs::read_dir(dir).ok())
            .flatten()
            .filter_map(Result::ok)
            .filter_map(|path| std::fs::OpenOptions::new().read(true).open(&path.path()).ok())
            .filter_map(|file| {
                unpak::Pak::new(
                    std::io::BufReader::new(file),
                    unpak::Version::FrozenIndex,
                    None,
                )
                .ok()
            })
            .collect();
        $self.selected = None;
        if let Some(map) = &$self.map {
            for index in actor::get_actors(map) {
                match actor::Actor::new(map, index) {
                    Ok(actor) => {
                        if let actor::DrawType::Mesh(path) = &actor.draw_type {
                            if !$self.meshes.contains_key(path) {
                                for pak in paks.iter_mut() {
                                    let Ok(mesh) = pak.get(&format!("{path}.uasset")) else {continue};
                                    let mesh_bulk = pak.get(&format!("{path}.uexp")).ok();
                                    let mut mesh = unreal_asset::Asset::new(mesh, mesh_bulk);
                                    mesh.set_engine_version($self.version);
                                    let Ok(()) = mesh.parse_data() else {continue};
                                    match extras::get_mesh_info(mesh) {
                                        Ok(positions) => {
                                            $self.meshes.insert(path.to_string(), rendering::Mesh::new($ctx, positions));
                                        },
                                        Err(e)=>{
                                            $self.notifs.error(format!("{path}: {e}"));
                                        }
                                    }
                                }
                            }
                        }
                        $self.actors.push(actor);
                    }
                    Err(e) => {
                        $self.notifs.warning(e.to_string());
                    }
                }
            }
        }
    };
}

impl Stove {
    pub fn new(ctx: &mut GraphicsContext) -> Self {
        unsafe {
            gl::glEnable(gl::GL_PROGRAM_POINT_SIZE);
        }
        let mut notifs = egui_notify::Toasts::new();
        let (version, paks) = match config() {
            Some(ref cfg) => {
                if !cfg.exists() && std::fs::create_dir(&cfg).is_err() {
                    notifs.error("failed to create config directory");
                }
                (
                    EngineVersion::try_from(
                        std::fs::read_to_string(cfg.join("VERSION"))
                            .unwrap_or_else(|_| "0".to_string())
                            .parse::<i32>()
                            .unwrap_or_default(),
                    )
                    .unwrap_or(EngineVersion::UNKNOWN),
                    // for some reason doing lines and collect here gives the compiler a seizure
                    std::fs::read_to_string(cfg.join("PAKS")).unwrap_or_default(),
                )
            }
            None => (EngineVersion::UNKNOWN, String::default()),
        };
        let paks = paks.lines().map(str::to_string).collect();
        let mut filepath = String::new();
        let map =
            std::env::args()
                .nth(1)
                .and_then(|path| match asset::open(path.clone(), version) {
                    Ok(asset) => {
                        filepath = path;
                        Some(asset)
                    }
                    Err(e) => {
                        notifs.error(e.to_string());
                        None
                    }
                });
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
        let home = home_dir();

        let mut stove = Self {
            camera: rendering::Camera::default(),
            notifs,
            map,
            version,
            egui: egui_miniquad::EguiMq::new(ctx),
            actors: Vec::new(),
            selected: None,
            cubes: rendering::Cube::new(ctx),
            meshes: hashbrown::HashMap::new(),
            ui: true,
            donor: None,
            open_dialog: egui_file::FileDialog::open_file(home.clone())
                .resizable(false)
                .filter(Box::new(filter)),
            transplant_dialog: egui_file::FileDialog::open_file(home.clone())
                .resizable(false)
                .filter(Box::new(filter)),
            save_dialog: egui_file::FileDialog::save_file(home.clone())
                .resizable(false)
                .filter(Box::new(filter)),
            pak_dialog: egui_file::FileDialog::select_folder(home)
                .resizable(false)
                .filter(Box::new(filter)),
            filepath,
            held: Vec::new(),
            last_mouse_pos: glam::Vec2::ZERO,
            grab: Grab::None,
            paks,
            #[cfg(not(target_family = "wasm"))]
            client,
        };
        refresh!(stove, ctx);
        if stove.map.is_none() {
            stove.open_dialog.open()
        }
        stove
    }
}

fn filter(path: &std::path::Path) -> bool {
    path.extension().and_then(std::ffi::OsStr::to_str) == Some("umap")
}

impl EventHandler for Stove {
    fn update(&mut self, _: &mut Context) {
        self.camera.update_times();
        self.camera.move_cam(&self.held)
    }

    fn draw(&mut self, mqctx: &mut Context) {
        mqctx.begin_default_pass(PassAction::Clear {
            color: Some((0.15, 0.15, 0.15, 1.0)),
            depth: Some(1.0),
            stencil: None,
        });
        let vp = rendering::PROJECTION * self.camera.view_matrix();
        if let Some(map) = &self.map {
            self.cubes.draw(
                mqctx,
                &self
                    .actors
                    .iter()
                    .map(|actor| actor.model_matrix(map))
                    .collect::<Vec<_>>(),
                &(
                    vp,
                    match self.selected {
                        Some(i) => [1, i as i32],
                        None => [0, 0],
                    },
                ),
            );
            for (actor, mesh) in self
                .actors
                .iter()
                .filter_map(|actor| match &actor.draw_type {
                    actor::DrawType::Mesh(key) => self.meshes.get(key).map(|mesh| (actor, mesh)),
                    actor::DrawType::Cube => None,
                })
            {
                mesh.draw(mqctx, &(vp * actor.model_matrix(map)));
            }
        }
        mqctx.end_render_pass();
        if !self.ui {
            mqctx.commit_frame();
            return;
        }
        self.egui.run(mqctx, |mqctx, ctx| {
            egui::SidePanel::left("sidepanel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("file", |ui| {
                        if ui.add(egui::Button::new("open").shortcut_text("ctrl + O")).clicked() {
                            self.open_dialog.open();
                        }
                        if ui.add(egui::Button::new("save").shortcut_text("ctrl + S")).clicked(){
                            match &mut self.map{
                                Some(map) => match asset::save(map,&self.filepath){
                                    Ok(_) => self.notifs.success("map saved"),
                                    Err(e) => self.notifs.error(e.to_string()),
                                },
                                None => {
                                    self.notifs.error("no map to save")
                                },
                            };
                        }
                        if ui.add(egui::Button::new("save as").shortcut_text("ctrl + shift+ S")).clicked(){
                            match self.map.is_some() {
                                true => self.save_dialog.open(),
                                false => {
                                    self.notifs.error("no map to save");
                                },
                            }
                        }
                    });
                    ui.menu_button("paks", |ui| {
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
                        if ui.add(egui::Button::new("select folder").shortcut_text("alt + O")).clicked() {
                            self.pak_dialog.open();
                        }
                    });
                    ui.menu_button("options", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("theme:");
                            egui::global_dark_light_mode_buttons(ui);
                        });
                        fn binding(ui:&mut egui::Ui,action:&str,binding:&str){
                            ui.label(action);
                            ui.label(binding);
                            ui.end_row();
                        }
                        ui.menu_button("shortcuts", |ui|{
                            egui::Grid::new("shortcuts").striped(true).show(ui,|ui|{
                                ui.heading("file");
                                ui.end_row();
                                binding(ui,"open map","ctrl + O");
                                binding(ui,"save map","ctrl + S");
                                binding(ui,"save map as","ctrl + shift + S");
                                binding(ui,"add folder","alt + O");
                                ui.heading("camera");
                                ui.end_row();
                                binding(ui,"move","wasd");
                                binding(ui,"rotate","right-click + drag");
                                binding(ui,"change speed","scroll wheel");
                                ui.heading("viewport");
                                ui.end_row();
                                binding(ui,"exit","escape");
                                binding(ui,"hide ui","H");
                                binding(ui,"select","left-click");
                                binding(ui,"transplant","ctrl + T");
                                ui.heading("actor");
                                ui.end_row();
                                binding(ui,"focus","F");
                                binding(ui,"move","left-click + drag");
                                binding(ui,"rotate","right-click + drag");
                                binding(ui,"scale","middle-click + drag");
                                binding(ui,"duplicate","ctrl + D");
                                binding(ui,"delete","delete");
                            });
                        });
                        ui.menu_button("about",|ui|{
                            ui.horizontal_wrapped(|ui|{
                                let size=ui.fonts().glyph_width(&egui::TextStyle::Body.resolve(ui.style()), ' ');
                                ui.spacing_mut().item_spacing.x=size;
                                ui.label("stove is an editor for cooked unreal map files running on my spaghetti code - feel free to help untangle it on");
                                ui.hyperlink_to("github","https://github.com/bananaturtlesandwich/stove");
                                ui.label(egui::special_emojis::GITHUB.to_string());
                            });
                        });
                        if ui.add(egui::Button::new("delete").shortcut_text("delete")).clicked(){
                            match self.selected {
                                Some(index) => {
                                    self.selected = None;
                                    self.actors[index].delete(self.map.as_mut().unwrap());
                                    self.notifs
                                        .success(format!("deleted {}", &self.actors[index].name));
                                    self.actors.remove(index);
                                }
                                None => {
                                    self.notifs.error("nothing selected to delete");
                                }
                            }
                        }
                        if ui.add(egui::Button::new("duplicate").shortcut_text("ctrl + D")).clicked(){
                            match self.selected {
                                Some(index) => {
                                    let map = self.map.as_mut().unwrap();
                                    let insert = map.exports.len() as i32 + 1;
                                    self.selected = Some(self.actors.len());
                                    self.actors[index].duplicate(map);
                                    self.actors.push(
                                        actor::Actor::new(
                                            map,
                                            PackageIndex::new(insert),
                                        )
                                        .unwrap(),
                                    );
                                    self.notifs
                                        .success(format!("duplicated {}", &self.actors[index].name));
                                }
                                None => {
                                    self.notifs.error("nothing selected to duplicate");
                                }
                            }
                        }
                        if ui.add(egui::Button::new("transplant").shortcut_text("ctrl + T")).clicked(){
                            match &self.map.is_some() {
                                true => {
                                    self.transplant_dialog.open();
                                }
                                false => {
                                    self.notifs.error("no map to transplant to");
                                }
                            }
                        }
                        if ui.add(egui::Button::new("exit").shortcut_text("escape")).clicked(){
                            mqctx.request_quit();
                        }
                    });
                    egui::ComboBox::from_id_source("version")
                        .selected_text(*VERSIONS.iter().find_map(|(version,name)|(version==&self.version).then_some(name)).unwrap_or(&"unknown"))
                        .show_ui(ui, |ui| {
                            for (version,name) in VERSIONS {
                                ui.selectable_value(&mut self.version, version, name);
                            }
                        });
                });
                if let Some(map) = &mut self.map {
                    ui.add_space(10.0);
                    ui.push_id("actors", |ui| egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_height(ui.available_height() * 0.5)
                        .show_rows(
                            ui,
                            ui.text_style_height(&egui::TextStyle::Body),
                            self.actors.len(),
                            |ui,range|{
                            for i in range {
                                let is_selected = Some(i) == self.selected;
                                if ui.selectable_label(
                                    is_selected,
                                    &self.actors[i].name
                                )
                                .on_hover_text(&self.actors[i].class)
                                .clicked(){
                                    self.selected=(!is_selected).then_some(i);
                                }
                            };
                        })
                    );
                    if let Some(selected) = self.selected{
                        ui.add_space(10.0);
                        ui.push_id("properties", |ui| egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui,|ui|{
                                self.actors[selected].show(map,ui);
                                // otherwise the scroll area bugs out at the bottom
                                ui.add_space(1.0);
                            })
                        );
                    }
                }
            });
            if let Some(map) = &mut self.map {
                let mut open = true;
                let mut transplanted = false;
                if let Some((donor, actors)) = &self.donor{
                    egui::Window::new("transplant actor")
                        .anchor(egui::Align2::CENTER_CENTER, (0.0,0.0))
                        .resizable(false)
                        .collapsible(false)
                        .open(&mut open)
                        .show(ctx, |ui|{
                        egui::ScrollArea::vertical()
                            .auto_shrink([false;2])
                            .show_rows(
                                ui,
                                ui.text_style_height(&egui::TextStyle::Body),
                                actors.len(),
                                |ui,range|{
                                    for i in range {
                                        if ui.selectable_label(false, &actors[i].name).on_hover_text(&actors[i].class).clicked(){
                                            let insert = map.exports.len() as i32 + 1;
                                            actors[i].transplant(map, donor);
                                            let selected = self.actors.len();
                                            self.actors.push(
                                                actor::Actor::new(
                                                    map,
                                                    PackageIndex::new(insert),
                                                )
                                                .unwrap(),
                                            );
                                            self.selected = Some(selected);
                                            self.camera.set_focus(
                                                self.actors[selected].location(&map),
                                                self.actors[selected].scale(&map),
                                            );
                                            self.notifs.success(format!("transplanted {}", actors[i].name));
                                            transplanted = true;
                                        }
                                    }
                                }
                            );
                        }
                    );
                }
                if transplanted || !open {
                    self.donor = None;
                }
            }
            self.notifs.show(ctx);
            self.open_dialog.show(ctx);
            if self.open_dialog.selected() {
                if let Some(path) = self.open_dialog.path() {
                    match asset::open(path.clone(), self.version) {
                        Ok(asset) => {
                            self.filepath = path.to_str().unwrap_or_default().to_string();
                            #[cfg(not(target_family = "wasm"))]
                            if let Some(client) = &mut self.client{
                                if client.set_activity(
                                            default_activity()
                                                .details("currently editing:")
                                                .state(self.filepath.split('\\').last().unwrap_or_default())
                                        ).is_err() {
                                        client.close().unwrap_or_default();
                                        self.client = None;
                                    }
                            }
                            self.map = Some(asset);
                            refresh!(self, mqctx);
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
                    match asset::open(path, self.version) {
                        Ok(donor) => {
                            // no need for verbose warnings here
                            let actors = actor::get_actors(&donor)
                                .into_iter()
                                .filter_map(|index| actor::Actor::new(&donor, index).ok())
                                .collect();
                            self.donor = Some((donor, actors));
                        }
                        Err(e) => {
                            self.notifs.error(e.to_string());
                        }
                    }
                }
            }
            self.pak_dialog.show(ctx);
            if self.pak_dialog.selected() {
                if let Some(path) = self.pak_dialog.path().and_then(|path|path.to_str().map(str::to_string)) {
                    self.paks.push(path.to_string());
                }
            }
            self.save_dialog.show(ctx);
            if self.save_dialog.selected() {
                if let Some(path) = self.save_dialog.path() {
                    match asset::save(self.map.as_mut().unwrap(), path.clone()){
                        Ok(_) => {
                            self.filepath = path.to_str().unwrap_or_default().to_string();
                            self.notifs.success("map saved")
                        },
                        Err(e) => self.notifs.error(e.to_string()),
                    };
                }
            }
        });
        self.egui.draw(mqctx);
        mqctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, _: &mut Context, x: f32, y: f32) {
        self.egui.mouse_motion_event(x, y);
        let delta = glam::vec2(x - self.last_mouse_pos.x, y - self.last_mouse_pos.y);
        self.camera.handle_mouse_motion(delta);
        match self.grab {
            Grab::None => (),
            Grab::Location(dist) => self.actors[self.selected.unwrap()].add_location(
                self.map.as_mut().unwrap(),
                // move across the camera view plane
                (self.camera.left() * -delta.x
                    + self.camera.front.cross(self.camera.left()) * delta.y)
                    // scale by consistent distance
                    * dist
                    // scale to match mouse cursor
                    * 0.1,
            ),
            Grab::Rotation => self.actors[self.selected.unwrap()].combine_rotation(
                self.map.as_mut().unwrap(),
                glam::Quat::from_axis_angle(
                    self.camera.front,
                    match delta.x.abs() > delta.y.abs() {
                        true => -delta.x,
                        false => delta.y,
                    } * 0.01,
                ),
            ),
            Grab::Scale3D(coords) => self.actors[self.selected.unwrap()].mul_scale(
                self.map.as_mut().unwrap(),
                glam::Vec3::ONE
                    + (coords.distance(glam::vec2(x, y)) - coords.distance(self.last_mouse_pos))
                        * 0.005,
            ),
        }
        self.last_mouse_pos = glam::vec2(x, y);
    }

    fn mouse_wheel_event(&mut self, _: &mut Context, dx: f32, dy: f32) {
        self.egui.mouse_wheel_event(dx, dy);
        // a logarithmic speed increase is better because unreal maps can get massive
        if !self.egui.egui_ctx().is_pointer_over_area() {
            self.camera.speed = (self.camera.speed as f32
                * match dy.is_sign_negative() {
                    true => 100.0 / -dy,
                    false => dy / 100.0,
                })
            .clamp(5.0, 50000.0) as u16;
        }
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
        self.egui.mouse_button_down_event(ctx, mb, x, y);
        if self.egui.egui_ctx().is_pointer_over_area() {
            return;
        }
        // THE HACKIEST MOUSE PICKING EVER CONCEIVED
        let pick = self
            .map
            .as_mut()
            .and_then(|map| {
                // normalise mouse coordinates to NDC
                let (width, height) = ctx.screen_size();
                let mouse = glam::vec2(x * 2.0 / width - 1.0, 1.0 - y * 2.0 / height);
                let proj = rendering::PROJECTION * self.camera.view_matrix();
                self.actors
                    .iter()
                    .map(|actor| {
                        let proj = proj * actor.location(map).extend(1.0);
                        // get NDC coordinates of actor
                        let actor = glam::vec2(proj.x / proj.w.abs(), proj.y / proj.w.abs());
                        mouse.distance(actor)
                    })
                    .enumerate()
                    // get closest pick
                    .min_by(|(_, x), (_, y)| x.total_cmp(y))
            })
            .and_then(|(pos, distance)| (distance < 0.05).then_some(pos));
        match self.selected == pick && pick.is_some() {
            // grabby time
            true => {
                if let Some(selected) = self.selected {
                    self.grab = match mb {
                        MouseButton::Left => Grab::Location(
                            self.actors[selected]
                                .location(self.map.as_ref().unwrap())
                                .distance(self.camera.position),
                        ),
                        MouseButton::Right => Grab::Rotation,
                        MouseButton::Middle => Grab::Scale3D({
                            // convert to mouse coordinates
                            let proj = rendering::PROJECTION
                                * self.camera.view_matrix()
                                * self.actors[selected]
                                    .location(self.map.as_ref().unwrap())
                                    .extend(1.0);
                            let (width, height) = ctx.screen_size();
                            glam::vec2(
                                (proj.x / proj.w.abs() + 1.0) * width * 0.5,
                                (1.0 - proj.y / proj.w.abs()) * height * 0.5,
                            )
                        }),
                        MouseButton::Unknown => Grab::None,
                    }
                }
            }
            false => {
                if mb == MouseButton::Right {
                    self.camera.enable_move()
                }
            }
        }
        if mb == MouseButton::Left {
            self.selected = pick;
        }
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
        self.egui.mouse_button_up_event(ctx, mb, x, y);
        if mb == MouseButton::Right {
            self.camera.disable_move()
        }
        self.grab = Grab::None;
    }

    // boilerplate >n<
    fn char_event(&mut self, _: &mut Context, character: char, _: KeyMods, _: bool) {
        self.egui.char_event(character);
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods, _: bool) {
        self.egui.key_down_event(ctx, keycode, keymods);
        if !self.egui.egui_ctx().is_pointer_over_area()
            && !keymods.ctrl
            && !self.held.contains(&keycode)
        {
            self.held.push(keycode)
        }
    }

    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        self.egui.key_up_event(keycode, keymods);
        if let Some(pos) = self.held.iter().position(|k| k == &keycode) {
            self.held.remove(pos);
        }
        match keycode {
            KeyCode::Delete => match self.selected {
                Some(index) => {
                    self.selected = None;
                    self.actors[index].delete(self.map.as_mut().unwrap());
                    self.notifs
                        .success(format!("deleted {}", &self.actors[index].name));
                    self.actors.remove(index);
                }
                None => {
                    self.notifs.error("nothing selected to delete");
                }
            },
            KeyCode::F => match self.selected {
                Some(selected) => self.camera.set_focus(
                    self.actors[selected].location(self.map.as_ref().unwrap()),
                    self.actors[selected].scale(self.map.as_ref().unwrap()),
                ),
                None => {
                    self.notifs.error("nothing selected to focus on");
                }
            },
            KeyCode::H => self.ui = !self.ui,
            KeyCode::D if keymods.ctrl => match self.selected {
                Some(index) => {
                    let map = self.map.as_mut().unwrap();
                    let insert = map.exports.len() as i32 + 1;
                    self.selected = Some(self.actors.len());
                    self.actors[index].duplicate(map);
                    self.actors
                        .push(actor::Actor::new(map, PackageIndex::new(insert)).unwrap());
                    self.notifs
                        .success(format!("duplicated {}", &self.actors[index].name));
                }
                None => {
                    self.notifs.error("nothing selected to duplicate");
                }
            },
            KeyCode::T if keymods.ctrl => match &self.map.is_some() {
                true => {
                    self.transplant_dialog.open();
                }
                false => {
                    self.notifs.error("no map to transplant to");
                }
            },
            KeyCode::O if keymods.ctrl => self.open_dialog.open(),
            KeyCode::O if keymods.alt => self.pak_dialog.open(),
            KeyCode::S if keymods.ctrl => match keymods.shift {
                true => match self.map.is_some() {
                    true => self.save_dialog.open(),
                    false => {
                        self.notifs.error("no map to save");
                    }
                },
                false => match &mut self.map {
                    Some(map) => match asset::save(map, &self.filepath) {
                        Ok(_) => {
                            self.notifs.success("map saved");
                        }
                        Err(e) => {
                            self.notifs.error(e.to_string());
                        }
                    },
                    None => {
                        self.notifs.error("no map to save");
                    }
                },
            },
            KeyCode::Escape => ctx.request_quit(),
            _ => (),
        }
    }
}

impl Drop for Stove {
    fn drop(&mut self) {
        #[cfg(not(target_family = "wasm"))]
        if let Some(client) = &mut self.client {
            client.close().unwrap_or_default();
        }
        if let Some(path) = config() {
            std::fs::write(path.join("VERSION"), (self.version as u8).to_string())
                .unwrap_or_default();
            std::fs::write(path.join("PAKS"), self.paks.join("\n")).unwrap_or_default();
        }
    }
}

const VERSIONS: [(EngineVersion, &str); 31] = [
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
];
