use miniquad::*;
mod rendering;
mod actor;
mod asset_utils;
mod map_utils;

pub struct Stove {
    camera: rendering::camera::Camera,
    notifs: egui_notify::Toasts,
    map: Option<unreal_asset::Asset>,
    version: i32,
    egui: egui_miniquad::EguiMq,
    actors: Vec<actor::Actor>,
    selected: Option<usize>,
    cube: rendering::cube::Cube
}

fn config() -> std::path::PathBuf {
    dirs::config_dir().unwrap().join("stove")
}

// the only way i could get it to compile and look nice :p
macro_rules! update_actors{
    ($self: expr)=>{
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
    }
}

impl Stove {
    pub fn new(ctx: &mut GraphicsContext) -> Self {
        let mut notifs = egui_notify::Toasts::new();
        let config = config();
        if !config.exists() && std::fs::create_dir(&config).is_err() {
            notifs.error("failed to create config directory");
        }
        let version = std::fs::read_to_string(config.join("VERSION"))
            .unwrap_or("0".to_string())
            .parse()
            .unwrap();
        let map = match std::env::args().nth(1) {
            Some(path) => match asset_utils::open_asset(path, unreal_asset::ue4version::VER_UE4_25)
            {
                Ok(asset) => Some(asset),
                Err(e) => {
                    notifs.error(e.to_string());
                    None
                }
            },
            None => None,
        };
        let mut stove = Self {
            camera: rendering::camera::Camera::default(),
            notifs,
            map,
            version,
            egui: egui_miniquad::EguiMq::new(ctx),
            actors: Vec::new(),
            selected: None,
            cube: rendering::cube::Cube::new(ctx)
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
        ctx.begin_default_pass(PassAction::Clear {
            color: Some((0.15, 0.15, 0.15, 1.0)),
            depth: Some(1.0),
            stencil: None,
        });
        if let Some(map)=&mut self.map{
            for (i,actor) in self.actors.iter().enumerate(){
                self.cube.draw(
                    ctx,
                    glam::Mat4::from_translation(actor.get_translation(map)),
                    self.camera.view_matrix(),
                    match self.selected == Some(i){
                        true => glam::vec3(1.0,1.0,0.5),
                        false => glam::vec3(0.0,1.0,0.5)
                    }
                );
            }
        }
        ctx.end_render_pass();
        self.egui.run(ctx, |mqctx, ctx| {
            egui::SidePanel::left("sidepanel").show(ctx, |ui| {
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
                                Some(map) => 
                                if let Ok(Some(path)) = native_dialog::FileDialog::new()
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
                ui.add_space(10.0);
                if let Some(map)=&mut self.map{
                    ui.push_id("actors",|ui|egui::ScrollArea::vertical().auto_shrink([false;2]).show_rows(ui,ui.text_style_height(&egui::TextStyle::Body),self.actors.len(),|ui,range|{
                        for i in range{
                            let is_selected=Some(i)==self.selected;
                            if ui.selectable_label(is_selected,self.actors[i].name(map)).on_hover_text(self.actors[i].class(map)).clicked(){
                                self.selected=(!is_selected).then_some(i);
                                if let Some(i)=self.selected{
                                    self.camera.set_focus(self.actors[i].get_translation(map));
                                }
                            }
                        };
                    }));
                    ui.add_space(1.0);
                    if let Some(selected)=self.selected{
                        egui::SidePanel::right("properties").show(ctx, |ui|{
                            egui::ScrollArea::vertical().auto_shrink([false;2]).show(ui,|ui|{
                                self.actors[selected].show(map,ui); 
                             });
                        });
                        egui_gizmo::Gizmo::new("gizmo")
                            .view_matrix(self.camera.view_matrix().to_cols_array_2d())
                            .model_matrix(glam::Mat4::from_translation(self.actors[selected].get_translation(map)).to_cols_array_2d())
                            .projection_matrix(glam::Mat4::perspective_infinite_lh(45.0, 1920.0/1080.0, 10.0).to_cols_array_2d())
                            .interact(ui);
                    }
                }
            });
            self.notifs.show(ctx);
        });
        self.egui.draw(ctx);
        ctx.commit_frame();
    }
    // boilerplate >n<
    fn mouse_motion_event(&mut self, _: &mut Context, x: f32, y: f32) {
        self.egui.mouse_motion_event(x, y);
        self.camera.handle_mouse_motion(x,y);
    }

    fn mouse_wheel_event(&mut self, _: &mut Context, dx: f32, dy: f32) {
        self.egui.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
        self.egui.mouse_button_down_event(ctx, mb, x, y);
        self.camera.handle_mouse_down(mb);
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
        self.egui.mouse_button_up_event(ctx, mb, x, y);
        self.camera.handle_mouse_up(mb);
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
