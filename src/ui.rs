use super::*;

pub fn ui(
    mut ctx: bevy_egui::EguiContexts,
    mut appdata: ResMut<AppData>,
    mut commands: Commands,
    mut dialog: EventWriter<Dialog>,
    mut notif: EventWriter<Notif>,
    mut map: NonSendMut<Map>,
    mut transplant: NonSendMut<Transplant>,
    consts: Res<Constants>,
    actors: Query<(Entity, &actor::Actor)>,
    selected: Query<(Entity, &actor::Actor), With<actor::Selected>>,
    matched: Query<(Entity, &actor::Actor), With<actor::Matched>>,
) {
    egui::SidePanel::left("sidepanel").show(ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.menu_button("file", |ui| {
                if ui
                    .add(egui::Button::new("open").shortcut_text("ctrl + o"))
                    .clicked()
                {
                    dialog.send(Dialog::Open(None));
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("save").shortcut_text("ctrl + s"))
                    .clicked()
                {
                    dialog.send(Dialog::SaveAs(false));
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("save as").shortcut_text("ctrl + shift + s"))
                    .clicked()
                {
                    dialog.send(Dialog::SaveAs(true));
                    ui.close_menu();
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
                    dialog.send(Dialog::AddPak)
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
                ui.menu_button("shortcuts", shortcuts);
                ui.horizontal(|ui|{
                    ui.label("cache meshes:");
                    ui.add(egui::Checkbox::without_text(&mut appdata.cache));
                });
                if ui.button("clear cache").clicked() {
                    match config() {
                        Some(cache) => match std::fs::remove_dir_all(cache.join("cache")) {
                            Ok(()) => notif.send(Notif {
                                message: "cleared cache".into(),
                                kind: egui_notify::ToastLevel::Info
                            }),
                            Err(e) => notif.send(Notif {
                                message: e.to_string(),
                                kind: egui_notify::ToastLevel::Error
                            }),
                        },
                        None => notif.send(Notif {
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
        if ui.add(egui::TextEdit::singleline(&mut appdata.query).hint_text("🔎 search actors")).changed() {
            for (entity, _) in matched.iter() {
                commands.entity(entity).remove::<actor::Matched>();
            }
            for (entity, actor) in actors.iter() {
                if actor.name.to_ascii_lowercase().contains(&appdata.query.to_ascii_lowercase()) {
                    commands.entity(entity).insert(actor::Matched);
                }
            }
        }
        ui.add_space(10.0);
        egui::ScrollArea::both()
            .id_source("actors")
            .auto_shrink([false, true])
            .max_height(ui.available_height() * 0.5)
            .show_rows(
                ui,
                ui.text_style_height(&egui::TextStyle::Body),
                match appdata.query.is_empty() {
                    true => actors.iter().len(),
                    false => matched.iter().len(),
                },
                |ui, range| ui.with_layout(egui::Layout::default().with_cross_justify(true), |ui| {
                    let mut displayed: Vec<_> = match appdata.query.is_empty() {
                        true => actors.iter().collect(),
                        false => matched.iter().collect(),
                    };
                    displayed.sort_by_key(|(_, actor)| actor.export);
                    for (entity, actor) in displayed.into_iter().skip(range.start).take(range.end - range.start) {
                        let highlighted = selected.contains(entity);
                        if ui.selectable_label(
                            highlighted,
                            &actor.name
                        )
                        .on_hover_text(&actor.class)
                        .clicked() {
                            ui.input(|state| if !state.modifiers.shift && !state.modifiers.ctrl{
                                for (entity, _) in selected.iter() {
                                    commands.entity(entity).remove::<actor::SelectedBundle>();
                                }
                            });
                            match highlighted {
                                true => commands.entity(entity).remove::<actor::SelectedBundle>(),
                                // false if ui.input(|input| input.modifiers.shift) => todo!(),
                                false => commands.entity(entity).insert(actor::SelectedBundle::default()),
                            };
                        }
                    }
                    // otherwise the scroll area bugs out at the bottom
                    ui.add_space(10.0);
                })
            );
        if let (Ok((_, actor)), Some((map, _))) = (selected.get_single(), &mut map.0) {
            egui::ScrollArea::both()
                .id_source("properties")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    actor.show(map, ui);
                    // otherwise the scroll area bugs out at the bottom
                    ui.add_space(10.0);
                });
        }
    });
    let mut open = true;
    let mut transplanted = None;
    if let (Some((donor, others, selected)), Some((map, _))) = (&mut transplant.0, &mut map.0) {
        egui::Window::new("transplant actor")
            .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .open(&mut open)
            .show(ctx.ctx_mut(), |ui| {
                // putting the button below breaks the scroll area somehow
                ui.add_enabled_ui(!selected.is_empty(), |ui| {
                    if ui
                        .vertical_centered_justified(|ui| ui.button("transplant selected"))
                        .inner
                        .clicked()
                    {
                        let len = actors.iter().len();
                        transplanted = Some(len..len + selected.len());
                        for actor in selected.iter().map(|i| &others[*i]) {
                            let insert = unreal_asset::types::PackageIndex::new(
                                map.asset_data.exports.len() as i32 + 1,
                            );
                            actor.transplant(map, donor);
                            notif.send(Notif {
                                message: format!("transplanted {}", actor.name),
                                kind: Success,
                            });
                            let mut actor = actor::Actor::new(map, insert).unwrap();
                            actor.draw_type = actor::DrawType::Cube;
                            commands
                                .spawn((
                                    actor::Selected,
                                    PbrBundle {
                                        mesh: consts.bounds.clone_weak(),
                                        transform: actor.transform(map),
                                        visibility: Visibility::Hidden,
                                        ..default()
                                    },
                                    bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                                    actor, // child because it's LineList which picking can't do
                                ))
                                .with_children(|parent| {
                                    parent.spawn(PbrBundle {
                                        mesh: consts.cube.clone_weak(),
                                        visibility: Visibility::Visible,
                                        ..default()
                                    });
                                });
                        }
                    }
                });
                egui::ScrollArea::both().auto_shrink([false; 2]).show_rows(
                    ui,
                    ui.text_style_height(&egui::TextStyle::Body),
                    actors.iter().len(),
                    |ui, range| {
                        ui.with_layout(egui::Layout::default().with_cross_justify(true), |ui| {
                            for (i, actor) in range.clone().zip(others[range].iter()) {
                                if ui
                                    .selectable_label(selected.contains(&i), &actor.name)
                                    .on_hover_text(&actor.class)
                                    .clicked()
                                {
                                    ui.input(|input| {
                                        match selected.iter().position(|entry| entry == &i) {
                                            Some(i) => {
                                                selected.remove(i);
                                            }
                                            None if input.modifiers.shift
                                                && selected
                                                    .last()
                                                    .is_some_and(|last| last != &i) =>
                                            {
                                                let last_selected = *selected.last().unwrap();
                                                for i in match i < last_selected {
                                                    true => i..last_selected,
                                                    false => last_selected + 1..i + 1,
                                                } {
                                                    selected.push(i)
                                                }
                                            }
                                            _ => selected.push(i),
                                        }
                                    })
                                }
                            }
                        })
                    },
                );
            });
        if transplanted.is_some() || !open {
            transplant.0 = None
        }
    }
}

fn shortcuts(ui: &mut egui::Ui) {
    let mut section = |heading: &str, bindings: &[(&str, &str)]| {
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
            ("open", "ctrl + o"),
            ("save", "ctrl + s"),
            ("save as", "ctrl + shift + s"),
            ("add pak folder", "alt + o"),
        ],
    );
    section(
        "camera",
        &[
            ("move", "w + a + s + d"),
            ("rotate", "right-drag"),
            ("change speed", "scroll"),
        ],
    );
    section(
        "viewport",
        &[
            ("toggle fullscreen", "alt + enter"),
            ("hide ui", "h"),
            ("select", "left-click"),
            ("transplant", "ctrl + t"),
        ],
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
        ],
    );
}
