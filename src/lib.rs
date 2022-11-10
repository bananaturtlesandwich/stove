use miniquad::*;
mod actor;
mod asset_utils;
mod map_utils;

pub struct Stove {
    notifs: egui_notify::Toasts,
    map: Option<unreal_asset::Asset>,
    version: i32,
    egui: egui_miniquad::EguiMq,
    actors: Vec<actor::Actor>,
    selected: Option<usize>,
}

fn config() -> std::path::PathBuf {
    dirs::config_dir().unwrap().join("stove")
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
        let mut temp = Self {
            notifs,
            map,
            version,
            egui: egui_miniquad::EguiMq::new(ctx),
            actors: Vec::new(),
            selected: None,
        };
        temp.update_actors();
        temp
    }
    fn update_actors(&mut self) {
        self.actors.clear();
        self.selected = None;
        if let Some(map) = &self.map {
            for index in map_utils::get_actors(map) {
                match actor::Actor::new(map, index) {
                    Ok(actor) => {
                        self.actors.push(actor);
                    }
                    Err(e) => {
                        self.notifs.error(e.to_string());
                    }
                }
            }
        }
    }
}

impl EventHandler for Stove {
    fn update(&mut self, _: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.begin_default_pass(PassAction::Clear {
            color: Some((0.9, 1.0, 0.9, 1.0)),
            depth: Some(1.0),
            stencil: None,
        });
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
                                        // why this compile but calling update_actors to do the same doesn't?
                                        self.actors.clear();
                                        self.selected=None;
                                        if let Some(map) = &self.map {
                                            for index in map_utils::get_actors(map) {
                                                match actor::Actor::new(map, index) {
                                                    Ok(actor) => {
                                                        self.actors.push(actor);
                                                    }
                                                    Err(e) => {
                                                        self.notifs.error(e.to_string());
                                                    }
                                                }
                                            }
                                        }
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
                                None => {self.notifs.error("no map to save");},
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
                                ui.label("stove is an editor for cooked unreal map files running on my spaghetti code - feel free to help untangle it on");
                                ui.hyperlink_to("github","https://github.com/bananaturtlesandwich/stove");
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
                            if ui.selectable_label(is_selected,self.actors[i].name(map)).clicked(){
                                self.selected=(!is_selected).then_some(i);
                            }
                        };
                        ui.add_space(1.0);
                    }));
                    if let Some(selected)=self.selected{
                        egui::SidePanel::right("properties").show(ctx, |ui|{
                            egui::ScrollArea::vertical().auto_shrink([false;2]).show(ui,|ui|{
                                self.actors[selected].show(map,ui); 
                             });
                        });
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
    }

    fn mouse_wheel_event(&mut self, _: &mut Context, dx: f32, dy: f32) {
        self.egui.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
        self.egui.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, mb: MouseButton, x: f32, y: f32) {
        self.egui.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(&mut self, _: &mut Context, character: char, _: KeyMods, _: bool) {
        self.egui.char_event(character);
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods, _: bool) {
        self.egui.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(&mut self, _: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        self.egui.key_up_event(keycode, keymods);
    }
    // unfortunately i don't believe file drop is implemented for miniquad at the current time
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
