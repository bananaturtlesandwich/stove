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
    // holds distance from camera
    Location(f32),
    Rotation,
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
    cube: rendering::Cube,
    ui: bool,
    donor: Option<(unreal_asset::Asset, Vec<actor::Actor>)>,
    filepath: String,
    open_dialog: egui_file::FileDialog,
    transplant_dialog: egui_file::FileDialog,
    save_dialog: egui_file::FileDialog,
    held: Vec<KeyCode>,
    last_mouse_pos: glam::Vec2,
    grab: Grab,
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
    .and_then(|home| (!home.is_empty()).then_some(home))
    .map(std::path::PathBuf::from)
}

fn config() -> Option<std::path::PathBuf> {
    home_dir().map(convert).map(|path| path.join("stove"))
}

#[cfg(target_family = "windows")]
fn convert(path: std::path::PathBuf) -> std::path::PathBuf {
    path.join("AppData").join("Local")
}

#[cfg(target_family = "unix")]
fn convert(path: std::path::PathBuf) -> std::path::PathBuf {
    path.join(".config")
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

// the only way i could get it to compile and look nice :p
macro_rules! update_actors {
    ($self: expr) => {
        $self.actors.clear();
        $self.selected = None;
        if let Some(map) = &$self.map {
            for index in actor::get_actors(map) {
                match actor::Actor::new(map, index) {
                    Ok(actor) => {
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
        let mut notifs = egui_notify::Toasts::new();
        let config = config();
        let version = EngineVersion::try_from(match config {
            Some(cfg) => {
                if !cfg.exists() && std::fs::create_dir(&cfg).is_err() {
                    notifs.error("failed to create config directory");
                }
                std::fs::read_to_string(cfg.join("VERSION"))
                    .unwrap_or_else(|_| "0".to_string())
                    .parse()
                    .unwrap_or_default()
            }
            None => 0,
        })
        .unwrap_or(EngineVersion::UNKNOWN);
        let mut filepath = String::new();
        let map = match std::env::args().nth(1) {
            Some(path) => match asset::open(path.clone(), version) {
                Ok(asset) => {
                    filepath = path;
                    Some(asset)
                }
                Err(e) => {
                    notifs.error(e.to_string());
                    None
                }
            },
            None => None,
        };
        #[cfg(not(target_family = "wasm"))]
        let mut client = None;
        #[cfg(not(target_family = "wasm"))]
        if let Ok(mut cl) = discord_rich_presence::DiscordIpcClient::new("1059578289737433249") {
            if cl.connect().is_ok()
                && cl
                    .set_activity(match filepath.as_str() {
                        "" => default_activity(),
                        name => default_activity()
                            .details("currently editing:")
                            .state(name.split('\\').last().unwrap_or_default()),
                    })
                    .is_ok()
            {
                client = Some(cl);
            }
        }
        let home = home_dir();

        let mut stove = Self {
            camera: rendering::Camera::default(),
            notifs,
            map,
            version,
            egui: egui_miniquad::EguiMq::new(ctx),
            actors: Vec::new(),
            selected: None,
            cube: rendering::Cube::new(ctx),
            ui: true,
            donor: None,
            open_dialog: egui_file::FileDialog::open_file(home.clone())
                .resizable(false)
                .filter(Box::new(filter)),
            transplant_dialog: egui_file::FileDialog::open_file(home.clone())
                .resizable(false)
                .filter(Box::new(filter)),
            save_dialog: egui_file::FileDialog::save_file(home)
                .resizable(false)
                .filter(Box::new(filter)),
            filepath,
            held: Vec::new(),
            last_mouse_pos: glam::Vec2::ZERO,
            grab: Grab::None,
            #[cfg(not(target_family = "wasm"))]
            client,
        };
        update_actors!(stove);
        stove
    }
}

fn filter(path: &std::path::Path) -> bool {
    path.extension().map(|ext| ext.to_str()) == Some(Some("umap"))
}

impl EventHandler for Stove {
    fn update(&mut self, _: &mut Context) {
        self.camera.update_times();
        self.camera.move_cam(&self.held)
    }

    fn draw(&mut self, ctx: &mut Context) {
        ctx.begin_default_pass(PassAction::Clear {
            color: Some((0.15, 0.15, 0.15, 1.0)),
            depth: Some(1.0),
            stencil: None,
        });
        if let Some(map) = &mut self.map {
            self.cube.apply(ctx);
            for (i, actor) in self.actors.iter().enumerate() {
                self.cube.draw(
                    ctx,
                    actor.model_matrix(map),
                    self.camera.view_matrix(),
                    match self.selected == Some(i) {
                        true => glam::vec3(1.0, 1.0, 0.5),
                        false => glam::vec3(0.0, 1.0, 0.5),
                    },
                );
            }
        }
        ctx.end_render_pass();
        if !self.ui {
            ctx.commit_frame();
            return;
        }
        self.egui.run(ctx, |mqctx, ctx| {
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
                            match self.map.is_some(){
                                true => self.save_dialog.open(),
                                false => {
                                    self.notifs.error("no map to save");
                                },
                            }
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
                                if ui.selectable_value(&mut self.version, version, name).clicked(){
                                    if let Some(path) = config() {
                                        std::fs::write(path.join("VERSION"), (self.version as u8).to_string()).unwrap();
                                    }
                                }
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
                            .auto_shrink([false;2])
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
                            update_actors!(self);
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
            self.save_dialog.show(ctx);
            if self.save_dialog.selected(){
                if let Some(path) = self.save_dialog.path(){
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
        self.egui.draw(ctx);
        ctx.commit_frame();
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
            Grab::Rotation => (),
            Grab::Scale3D(coords) => self.actors[self.selected.unwrap()].mul_scale(
                self.map.as_mut().unwrap(),
                glam::Vec3::ONE
                    + (coords.distance(glam::vec2(x, y)) - coords.distance(self.last_mouse_pos))
                        * 0.01,
            ),
        }
        self.last_mouse_pos = glam::vec2(x, y);
    }

    fn mouse_wheel_event(&mut self, _: &mut Context, dx: f32, dy: f32) {
        self.egui.mouse_wheel_event(dx, dy);
        // a scaling speed increase is better because unreal maps can get massive
        if !self.egui.egui_ctx().is_pointer_over_area() {
            self.camera.speed = (self.camera.speed as f32
                * match dy.is_sign_negative() {
                    true => 100.0 / -dy,
                    false => dy / 100.0,
                })
            .clamp(5.0, 25000.0) as u16;
        }
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
        self.egui.mouse_button_down_event(ctx, mb, x, y);
        if self.egui.egui_ctx().is_pointer_over_area() {
            return;
        }
        match mb {
            // THE HACKIEST MOUSE PICKING TO EVER EXIST
            MouseButton::Left => {
                if let Some(map) = self.map.as_mut() {
                    // normalise mouse coordinates to NDC
                    let (width, height) = ctx.screen_size();
                    let mouse = glam::vec2(x * 2.0 / width - 1.0, 1.0 - y * 2.0 / height);
                    let proj = rendering::PROJECTION * self.camera.view_matrix();
                    if let Some((pos, distance)) = self
                        .actors
                        .iter()
                        .map(|actor| {
                            let proj = proj * actor.location(map).extend(1.0);
                            // get NDC coordinates of actor
                            let actor = glam::vec2(proj.x / proj.w.abs(), proj.y / proj.w.abs());
                            mouse.distance(actor)
                        })
                        .enumerate()
                        .min_by(|(_, x), (_, y)| x.total_cmp(y))
                    {
                        self.selected = (distance < 0.05).then_some(pos)
                    }
                }
            }
            MouseButton::Right => self.camera.enable_move(),
            _ => (),
        };
        // grabby time
        if let Some(selected) = self.selected {
            self.grab = match mb {
                MouseButton::Right => Grab::Rotation,
                MouseButton::Left => Grab::Location(
                    self.actors[selected]
                        .location(self.map.as_ref().unwrap())
                        .distance(self.camera.position),
                ),
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

    #[cfg(not(target_family = "wasm"))]
    fn quit_requested_event(&mut self, _ctx: &mut Context) {
        if let Some(client) = &mut self.client {
            client.close().unwrap_or_default();
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
