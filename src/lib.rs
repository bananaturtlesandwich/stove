use std::thread;
use miniquad::*;

mod actor;
mod asset;
mod rendering;
mod rpc;

pub struct Stove {
    camera: rendering::Camera,
    notifs: egui_notify::Toasts,
    map: Option<unreal_asset::Asset>,
    version: i32,
    egui: egui_miniquad::EguiMq,
    actors: Vec<actor::Actor>,
    selected: Option<usize>,
    cube: rendering::Cube,
    ui: bool,
    donor: Option<(unreal_asset::Asset, Vec<actor::Actor>)>,
    open_dialog: egui_file::FileDialog,
    transplant_dialog: egui_file::FileDialog,
    save_dialog: egui_file::FileDialog,
}

fn config() -> std::path::PathBuf {
    dirs::config_dir().unwrap().join("stove")
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
        if !config.exists() && std::fs::create_dir(&config).is_err() {
            notifs.error("failed to create config directory");
        }
        let version = std::fs::read_to_string(config.join("VERSION"))
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .unwrap();
        let mut open_dialog = egui_file::FileDialog::open_file(None)
            .resizable(false)
            .filter(Box::new(filter));
        let map = match std::env::args().nth(1) {
            Some(path) => {
                open_dialog = egui_file::FileDialog::open_file(Some(path.clone().into()))
                    .resizable(false)
                    .filter(Box::new(filter));
                match asset::open(path, version) {
                    Ok(asset) => Some(asset),
                    Err(e) => {
                        notifs.error(e.to_string());
                        None
                    }
                }
            }
            None => None,
        };
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
            transplant_dialog: egui_file::FileDialog::open_file(open_dialog.path())
                .resizable(false)
                .filter(Box::new(filter)),
            save_dialog: egui_file::FileDialog::save_file(open_dialog.path())
                .resizable(false)
                .filter(Box::new(filter)),
            open_dialog,
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
        self.camera.move_cam()
    }

    fn draw(&mut self, ctx: &mut Context) {
        let mut scissor = 0;
        if self.ui {
            self.egui.run(ctx, |mqctx, ctx| {
                egui::SidePanel::left("sidepanel").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.menu_button("file", |ui| {
                            if ui.button("open").clicked() {
                                self.open_dialog.open();
                            }
                            if ui.button("save as").clicked(){
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
                                    ui.heading("camera");
                                    ui.end_row();
                                    binding(ui,"move","wasd");
                                    binding(ui,"rotate","right-click + drag");
                                    binding(ui,"change speed", "scroll wheel");
                                    ui.heading("viewport");
                                    ui.end_row();
                                    binding(ui, "open map", "ctrl + O");
                                    binding(ui, "save map as", "ctrl + S");
                                    binding(ui,"hide ui","H");
                                    binding(ui,"select","click");
                                    binding(ui, "transplant", "ctrl + T");
                                    ui.heading("actor");
                                    ui.end_row();
                                    binding(ui,"focus","F");
                                    binding(ui,"duplicate","ctrl + D");
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
                            if ui.button("exit").clicked(){
                                mqctx.request_quit();
                            }
                        });
                        egui::ComboBox::from_id_source("version")
                            .selected_text(
                                VERSIONS
                                    .iter()
                                    .find(|version| version.1 == self.version)
                                    .unwrap()
                                    .0,
                            )
                            .show_ui(ui, |ui| {
                                for version in VERSIONS {
                                    if ui.selectable_value(&mut self.version, version.1, version.0).clicked(){
                                        if let Err(e)=std::fs::write(config().join("VERSION"),version.1.to_string()) {
                                            self.notifs.error(e.to_string());
                                        }
                                    }
                                }
                            });
                    });
                    if let Some(map)=&mut self.map {
                        ui.add_space(10.0);
                        ui.push_id("actors", |ui| egui::ScrollArea::vertical()
                            .auto_shrink([false;2])
                            .max_height(ui.available_height()*0.5)
                            .show_rows(
                                ui,
                                ui.text_style_height(&egui::TextStyle::Body),
                                self.actors.len(),
                                |ui,range|{
                                for i in range{
                                    let is_selected=Some(i)==self.selected;
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
                        if let Some(selected)=self.selected{
                            ui.add_space(10.0);
                            ui.push_id("properties", |ui|egui::ScrollArea::vertical()
                                .auto_shrink([false;2])
                                .show(ui,|ui|{
                                    self.actors[selected].show(map,ui);
                                    // otherwise the scroll area bugs out at the bottom
                                    ui.add_space(1.0);
                                })
                            );
                        }
                        let mut open = true;
                        let mut transplanted = false;
                        if let Some((donor, actors))=&self.donor{
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
                                            for i in range{
                                                if ui.selectable_label(false, &actors[i].name).on_hover_text(&actors[i].class).clicked(){
                                                    let insert = map.exports.len() as i32 + 1;
                                                    actors[i].transplant(map, donor);
                                                    self.actors.push(
                                                        actor::Actor::new(
                                                            map,
                                                            unreal_asset::unreal_types::PackageIndex::new(insert),
                                                        )
                                                        .unwrap(),
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
                    scissor=ui.available_width() as i32;
                });
                self.notifs.show(ctx);
                self.open_dialog.show(ctx);
                if self.open_dialog.selected() {
                    if let Some(path) = self.open_dialog.path() {
                        match asset::open(path.clone(), self.version) {
                            Ok(asset) => {
                                self.map = Some(asset);
                                update_actors!(self);
                                let file_name = path.clone()
                                    .file_name()
                                    .unwrap()
                                    .to_os_string()
                                    .into_string()
                                    .unwrap()
                                    .to_string();
                                thread::spawn(|| {
                                    rpc::rpc(file_name).expect("uh oh");
                                });
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
                    if let Some(path)=self.save_dialog.path(){
                        match asset::save(self.map.as_mut().unwrap(), path){
                            Ok(_) => self.notifs.success("saved map"),
                            Err(e) => self.notifs.error(e.to_string()),
                        };
                    }
                }
            })
        }
        ctx.begin_default_pass(PassAction::Clear {
            color: Some((0.15, 0.15, 0.15, 1.0)),
            depth: Some(1.0),
            stencil: None,
        });
        let (width, height) = ctx.display().screen_size();
        ctx.apply_scissor_rect(scissor, 0, width as i32, height as i32);
        if let Some(map) = &mut self.map {
            for (i, actor) in self.actors.iter().enumerate() {
                let rot = actor.get_rotation(map);
                self.cube.draw(
                    ctx,
                    glam::Mat4::from_scale_rotation_translation(
                        actor.get_scale(map),
                        glam::Quat::from_euler(
                            glam::EulerRot::XYZ,
                            rot.x.to_radians(),
                            rot.y.to_radians(),
                            rot.z.to_radians(),
                        ),
                        actor.get_translation(map),
                    ),
                    self.camera.view_matrix(),
                    match self.selected == Some(i) {
                        true => glam::vec3(1.0, 1.0, 0.5),
                        false => glam::vec3(0.0, 1.0, 0.5),
                    },
                );
            }
        }
        ctx.end_render_pass();
        if self.ui {
            self.egui.draw(ctx);
        }
        ctx.commit_frame();
    }
    // boilerplate >n<
    fn mouse_motion_event(&mut self, _: &mut Context, x: f32, y: f32) {
        self.egui.mouse_motion_event(x, y);
        self.camera.handle_mouse_motion(x, y);
    }

    fn mouse_wheel_event(&mut self, _: &mut Context, dx: f32, dy: f32) {
        self.egui.mouse_wheel_event(dx, dy);
        if !self.egui.egui_ctx().is_pointer_over_area() {
            self.camera.speed = (self.camera.speed as f32 + dy / 10.0).clamp(5.0, 250.0) as u8;
        }
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
        self.egui.mouse_button_down_event(ctx, mb, x, y);
        if !self.egui.egui_ctx().is_pointer_over_area() {
            self.camera.handle_mouse_down(mb);
        }
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
        self.egui.mouse_button_up_event(ctx, mb, x, y);
        self.camera.handle_mouse_up(mb);
        // i think this picking code must be some of the hackiest shit i've ever written
        // funnily enough it's probably more performant than raycasting
        if !self.egui.egui_ctx().is_pointer_over_area() && mb == MouseButton::Left {
            if let Some(map) = self.map.as_ref() {
                // convert the mouse coordinates to uv
                let (width, height) = ctx.screen_size();
                let coords = (x / width, 1.0 - y / height);
                let proj = self.camera.projection() * self.camera.view_matrix();
                self.selected = self.actors.iter().position(|actor| {
                    let proj = proj * actor.get_translation(map).extend(1.0);
                    // convert the actor position to uv
                    let uv = (
                        0.5 * (proj.x / proj.w.abs() + 1.0),
                        0.5 * (proj.y / proj.w.abs() + 1.0),
                    );
                    (uv.0 - coords.0).abs() < 0.01 && (uv.1 - coords.1).abs() < 0.01
                });
            }
        }
    }

    fn char_event(&mut self, _: &mut Context, character: char, _: KeyMods, _: bool) {
        self.egui.char_event(character);
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods, _: bool) {
        self.egui.key_down_event(ctx, keycode, keymods);
        if !self.egui.egui_ctx().is_pointer_over_area() && !keymods.ctrl {
            self.camera.handle_key_down(keycode);
        }
    }

    fn key_up_event(&mut self, _: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        self.egui.key_up_event(keycode, keymods);
        if !self.egui.egui_ctx().is_pointer_over_area() {
            self.camera.handle_key_up(keycode);
        }
        match keycode {
            KeyCode::F => {
                if let Some(selected) = self.selected {
                    self.camera.set_focus(
                        self.actors[selected].get_translation(self.map.as_ref().unwrap()),
                    )
                }
            }
            KeyCode::H => self.ui = !self.ui,
            KeyCode::D if keymods.ctrl => match self.selected {
                Some(index) => {
                    let map = self.map.as_mut().unwrap();
                    let insert = map.exports.len() as i32 + 1;
                    self.selected = Some(self.actors.len());
                    self.actors[index].duplicate(map);
                    self.actors.push(
                        actor::Actor::new(
                            map,
                            unreal_asset::unreal_types::PackageIndex::new(insert),
                        )
                        .unwrap(),
                    );
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
            KeyCode::S if keymods.ctrl => match self.map.is_some() {
                true => self.save_dialog.open(),
                false => {
                    self.notifs.error("no map to save");
                }
            },
            _ => (),
        }
    }
}

use unreal_asset::ue4version::*;
const VERSIONS: &[(&str, i32)] = &[
    ("unknown", UNKNOWN),
    ("oldest", VER_UE4_OLDEST_LOADABLE_PACKAGE),
    ("4.0", VER_UE4_0),
    ("4.1", VER_UE4_1),
    ("4.2", VER_UE4_2),
    ("4.3", VER_UE4_3),
    ("4.4", VER_UE4_4),
    ("4.5", VER_UE4_5),
    ("4.6", VER_UE4_6),
    ("4.7", VER_UE4_7),
    ("4.8", VER_UE4_8),
    ("4.9/10", VER_UE4_9),
    ("4.11", VER_UE4_11),
    ("4.12", VER_UE4_12),
    ("4.13", VER_UE4_13),
    ("4.14", VER_UE4_14),
    ("4.15", VER_UE4_15),
    ("4.16/17", VER_UE4_16),
    ("4.18", VER_UE4_18),
    ("4.19/20", VER_UE4_19),
    ("4.21/22/23", VER_UE4_21),
    ("4.24/25", VER_UE4_24),
    ("4.26", VER_UE4_26),
    ("4.27", VER_UE4_27),
];
