use super::*;

pub fn ui(
    mut ctx: bevy_egui::EguiContexts,
    mut appdata: ResMut<AppData>,
    mut command: EventWriter<Command>,
    mut notifs: ResMut<Notifs>,
) {
    notifs.0.show(ctx.ctx_mut());
    egui::SidePanel::left("sidepanel").show(ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.menu_button("file", |ui| {
                if ui
                    .add(egui::Button::new("open").shortcut_text("ctrl + o"))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("open map")
                        .add_filter("maps", &["umap"])
                        .pick_file()
                    {
                        command.send(Command::Open(path))
                    }
                }
                if ui
                    .add(egui::Button::new("save").shortcut_text("ctrl + s"))
                    .clicked()
                {
                    command.send(Command::SaveAs(false))
                }
                if ui
                    .add(egui::Button::new("save as").shortcut_text("ctrl + shift + s"))
                    .clicked()
                {
                    command.send(Command::SaveAs(true))
                }
            });
            egui::ComboBox::from_id_source("version")
                .show_index(ui, &mut appdata.version, VERSIONS.len(), |i| VERSIONS[i].1.to_string());
            ui.menu_button("paks", |ui| {
                egui::TextEdit::singleline(&mut appdata.aes)
                    .clip_text(false)
                    .hint_text("aes key if needed")
                    .show(ui);
                let mut remove_at = None;
                egui::ScrollArea::vertical().show_rows(
                    ui,
                    ui.text_style_height(&egui::TextStyle::Body),
                    appdata.paks.len(),
                    |ui, range| {
                        for i in range {
                            ui.horizontal(|ui| {
                                ui.label(&appdata.paks[i]);
                                if ui.button("x").clicked() {
                                    remove_at = Some(i);
                                }
                            });
                        }
                    },
                );
                if let Some(i) = remove_at {
                    appdata.paks.remove(i);
                }
                if ui
                    .add(egui::Button::new("add pak folder").shortcut_text("alt + o"))
                    .clicked()
                {
                    command.send(Command::AddPak)
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
                    ui.label("cache meshes:");
                    ui.add(egui::Checkbox::without_text(&mut appdata.cache));
                });
                if ui.button("clear cache").clicked() {
                    match config() {
                        Some(cache) => match std::fs::remove_dir_all(cache.join("cache")) {
                            Ok(()) => command.send(Command::Notif {
                                message: "cleared cache".into(),
                                kind: egui_notify::ToastLevel::Info
                            }),
                            Err(e) => command.send(Command::Notif {
                                message: e.to_string(),
                                kind: egui_notify::ToastLevel::Error
                            }),
                        },
                        None => command.send(Command::Notif {
                            message: "cache does not exist".into(),
                            kind: egui_notify::ToastLevel::Warning
                        }),
                    };
                }
                ui.horizontal(|ui| {
                    ui.label("render distance:");
                    ui.add(
                        egui::widgets::DragValue::new(&mut appdata.distance)
                            .clamp_range(0..=100000)
                    )
                });
                ui.label("post-save commands");
                ui.text_edit_multiline(&mut appdata.script);
            });
        });
    });
}
