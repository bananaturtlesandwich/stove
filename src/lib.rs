use miniquad::*;
mod asset_utils;

pub struct Stove {
    map: Option<unreal_asset::Asset>,
    version: i32,
    egui: egui_miniquad::EguiMq,
    notifs: egui_notify::Toasts,
}

impl Stove {
    pub fn new(ctx: &mut GraphicsContext) -> Self {
        let mut notifs = egui_notify::Toasts::new();
        let map = std::env::args().skip(1).next();
        let map = match map {
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
        let egui = egui_miniquad::EguiMq::new(ctx);
        let version = egui
            .egui_ctx()
            .memory()
            .data
            .get_persisted(egui::Id::new("version"))
            .unwrap_or_default();
        Self {
            map,
            version,
            egui,
            notifs,
        }
    }
}

impl EventHandler for Stove {
    fn update(&mut self, ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.begin_default_pass(PassAction::Clear {
            color: Some((0.8, 1.0, 0.8, 1.0)),
            depth: Some(1.0),
            stencil: None,
        });
        ctx.end_render_pass();
        self.egui.run(ctx, |_, ctx| {
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
                                    }
                                    Err(e) => {
                                        self.notifs.error(e.to_string());
                                    }
                                }
                            }
                        }
                    });
                    ui.menu_button("options", |ui| {
                        ui.menu_button("settings", |ui| {});
                        ui.menu_button("about", |ui| {});
                    });
                    egui::ComboBox::new("version", "").show_ui(ui, |ui| {});
                });
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
}
