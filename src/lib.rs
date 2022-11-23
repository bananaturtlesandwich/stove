use miniquad::*;
mod actor;
mod asset_utils;
mod map_utils;
mod rendering;

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
            for index in map_utils::get_actors(map) {
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
        let map = match std::env::args().nth(1) {
            Some(path) => match asset_utils::open_asset(path, version) {
                Ok(asset) => Some(asset),
                Err(e) => {
                    notifs.error(e.to_string());
                    None
                }
            },
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
        };
        update_actors!(stove);
        stove
    }
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
                egui::SidePanel::left("sidepanel").resizable(false).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.menu_button("file", |ui| {
                            if ui.button("open").clicked() {
                                if let Ok(Some(path)) = native_dialog::FileDialog::new()
                                    .add_filter("unreal map file", &["umap"])
                                    .show_open_single_file()
                                {
                                    match asset_utils::open_asset(path, self.version) {
                                        Ok(asset) => {
                                            self.map = Some(asset);
                                            update_actors!(self);
                                        }
                                        Err(e) => {
                                            self.notifs.error(e.to_string());
                                        }
                                    }
                                }
                            }
                            if ui.button("save as").clicked(){
                                match &self.map{
                                    Some(map) => if let Ok(Some(path)) = native_dialog::FileDialog::new()
                                        .add_filter("unreal map file", &["umap"])
                                        .show_save_single_file(){
                                            match asset_utils::save_asset(map, path){
                                                Ok(_) => self.notifs.success("saved map"),
                                                Err(e) => self.notifs.error(e.to_string()),
                                            };
                                        },
                                    None => {
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
                            ui.menu_button("keymap", |ui|{
                                egui::Grid::new("keymap").striped(true).show(ui,|ui|{
                                    ui.label("camera");
                                    ui.label("right-click + wasd + drag");
                                    ui.end_row();
                                    ui.label("focus");
                                    ui.label("F");
                                    ui.end_row();
                                    ui.label("hide ui");
                                    ui.label("H");
                                    ui.end_row();
                                    ui.label("duplicate");
                                    ui.label("ctrl + D")
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
                        ui.push_id("actors",|ui|egui::ScrollArea::vertical().auto_shrink([false;2]).max_height(ui.available_height()*0.5).show_rows(ui,ui.text_style_height(&egui::TextStyle::Body),self.actors.len(),|ui,range|{
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
                        }));
                        if let Some(selected)=self.selected{
                            ui.add_space(10.0);
                            egui::ScrollArea::vertical().auto_shrink([false;2]).show(ui,|ui|{
                                self.actors[selected].show(map,ui);
                                // otherwise the scroll area bugs out at the bottom
                                ui.add_space(1.0);
                            });
                        }
                    }
                    scissor=ui.available_width() as i32;
                });
                self.notifs.show(ctx);
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
                            glam::EulerRot::ZYX,
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
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
        self.egui.mouse_button_down_event(ctx, mb, x, y);
        if self.egui.egui_ctx().wants_pointer_input() {
            return;
        }
        self.camera.handle_mouse_down(mb);
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
        self.egui.mouse_button_up_event(ctx, mb, x, y);
        if self.egui.egui_ctx().wants_pointer_input() {
            return;
        }
        self.camera.handle_mouse_up(mb);
        // i think this picking code must be some of the hackiest shit i've ever written
        // funnily enough it's probably more performant than raycasting
        if mb == MouseButton::Left {
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
        self.camera.handle_key_down(keycode);
    }

    fn key_up_event(&mut self, _: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        self.egui.key_up_event(keycode, keymods);
        self.camera.handle_key_up(keycode);
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
                        .success(format!("cloned {}", &self.actors[index].name));
                }
                None => {
                    self.notifs.error("nothing selected to clone");
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
