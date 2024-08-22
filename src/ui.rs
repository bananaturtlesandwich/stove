use super::*;

pub fn sidebar(
    mut ctx: bevy_egui::EguiContexts,
    mut appdata: ResMut<AppData>,
    mut paks: ResMut<Paks>,
    mut commands: Commands,
    mut notif: EventWriter<Notif>,
    mut map: NonSendMut<Map>,
    mut transplant: NonSendMut<Transplant>,
    mut wire: ResMut<bevy::pbr::wireframe::WireframeConfig>,
    hidden: Res<Hidden>,
    consts: Res<Constants>,
    mut fps: ResMut<bevy_framepace::FramepaceSettings>,
    actors: Query<(Entity, &actor::Actor)>,
    mut selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
    mut cubes: Query<&mut Handle<wire::Wire>>,
    matched: Query<(Entity, &actor::Actor), With<actor::Matched>>,
) {
    if hidden.0 {
        return;
    }
    egui::SidePanel::left("sidepanel").show(ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.menu_button("file", |ui| {
                if ui
                    .add(egui::Button::new("open").shortcut_text("ctrl + o"))
                    .clicked()
                {
                    commands.trigger(triggers::Open(None));
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("transplant").shortcut_text("ctrl + t"))
                    .clicked()
                {
                    commands.trigger(triggers::Transplant);
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("save").shortcut_text("ctrl + s"))
                    .clicked()
                {
                    commands.trigger(triggers::SaveAs(false));
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("save as").shortcut_text("ctrl + shift + s"))
                    .clicked()
                {
                    commands.trigger(triggers::SaveAs(true));
                    ui.close_menu();
                }
            });
            egui::ComboBox::from_id_source("version").width(0.0)
                .show_index(ui, &mut appdata.version, VERSIONS.len(), |i| VERSIONS[i].1.to_string());
            let mut remove_at = None;
            // kinda wanna split appdata into components so this isn't necessary
            ui.menu_button("paks", |ui| {
                for i in 0..appdata.paks.len() {
                    ui.horizontal(|ui| {
                        let selected = appdata.pak == Some(i);
                        if ui.selectable_label(selected, &appdata.paks[i].0).clicked() {
                            appdata.pak = match selected {
                                true => None,
                                false => Some(i),
                            };
                            commands.trigger(triggers::LoadPaks);
                        }
                        egui::TextEdit::singleline(&mut appdata.paks[i].1)
                            .clip_text(false)
                            .hint_text("aes key if needed")
                            .desired_width(100.0)
                            .show(ui);
                        if ui.button("x").clicked() {
                            if selected {
                                appdata.pak = None;
                                paks.0.clear();
                            }
                            if let Some(pak) = appdata.pak.as_mut() {
                                if &i < pak {
                                    *pak -= 1;
                                }
                            }
                            remove_at = Some(i)
                        }
                    });
                }
                if ui
                    .add(egui::Button::new("add pak folder").shortcut_text("alt + o"))
                    .clicked()
                {
                    commands.trigger(triggers::AddPak);
                }
            });
            if let Some(i) = remove_at {
                appdata.paks.remove(i);
            }
            ui.menu_button("options", |ui| {
                ui.horizontal(|ui| {
                    ui.label("frame rate cap:"); 
                    if ui.add(egui::Checkbox::without_text(&mut appdata.cap)).changed() {
                        fps.limiter = match appdata.cap {
                            true => bevy_framepace::Limiter::from_framerate(appdata.rate),
                            false => bevy_framepace::Limiter::Off,
                        }
                    }
                    if ui.add_enabled(appdata.cap, egui::DragValue::new(&mut appdata.rate).range(5..=i32::MAX)).changed() {
                        fps.limiter = bevy_framepace::Limiter::from_framerate(appdata.rate)
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("load textures:");
                    ui.add(egui::Checkbox::without_text(&mut appdata.textures));
                });
                ui.horizontal(|ui| {
                    ui.label("show wireframe");
                    if ui.add(egui::Checkbox::without_text(&mut appdata.wireframe)).changed() {
                        wire.global = !wire.global
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("cache assets:");
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
                ui.label("post-save commands");
                ui.text_edit_multiline(&mut appdata.script);
            });
            ui.menu_button("help", |ui| {
                ui.menu_button("about",|ui| {
                    ui.horizontal_wrapped(|ui| {
                        let size = ui.fonts(|fonts| fonts.glyph_width(&egui::TextStyle::Body.resolve(ui.style()), ' '));
                        ui.spacing_mut().item_spacing.x = size;
                        ui.label("stove is an editor for cooked unreal map files running on my spaghetti code - feel free to help untangle it on");
                        ui.hyperlink_to("github","https://github.com/bananaturtlesandwich/stove");
                        ui.label(egui::special_emojis::GITHUB.to_string());
                    });
                });
                ui.menu_button("shortcuts", shortcuts);
            })
        });
        if map.0.is_none() {
            return;
        }
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
                            &actor.display,
                        )
                        .on_hover_text(&actor.class)
                        .clicked() {
                            ui.input(|state| if !state.modifiers.shift && !state.modifiers.ctrl {
                                for (entity, ..) in selected.iter() {
                                    match cubes.get_mut(entity) {
                                        Ok(mut mat) => {
                                            commands.entity(entity).remove::<actor::Selected>();
                                            *mat = consts.unselected.clone_weak();
                                        },
                                        Err(_) => {
                                            commands.entity(entity).remove::<actor::SelectedBundle>();
                                        },
                                    }
                                }
                            });
                            match highlighted {
                                true => match cubes.get_mut(entity) {
                                    Ok(mut mat) => {
                                        commands.entity(entity).remove::<actor::Selected>();
                                        *mat = consts.unselected.clone_weak();
                                    },
                                    Err(_) => {
                                        commands.entity(entity).remove::<actor::SelectedBundle>();
                                    },
                                },
                                // false if ui.input(|input| input.modifiers.shift) => todo!(),
                                false => match cubes.get_mut(entity) {
                                    Ok(mut mat) => {
                                        commands.entity(entity).insert(actor::Selected);
                                        *mat = consts.selected.clone_weak();
                                    }
                                    Err(_) => {
                                        commands
                                            .entity(entity)
                                            .insert(actor::SelectedBundle::default());
                                    }
                                },
                            };
                        }
                    }
                })
            );
        ui.add_space(10.0);
        if let (Ok((_, actor, mut transform)), Some((map, _, exports, imports))) = (selected.get_single_mut(), &mut map.0) {
            egui::ScrollArea::both()
                .id_source("properties")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    actor.show(map, ui, &mut transform, &exports, &imports);
                });
        }
    });
    let mut open = true;
    let mut transplanted = None;
    if let (Some((donor, others, selected)), Some((map, _, export_names, import_names))) =
        (&mut transplant.0, &mut map.0)
    {
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
                            let len = map.asset_data.exports.len();
                            let insert = unreal_asset::types::PackageIndex::new(len as i32 + 1);
                            actor.transplant(map, donor, export_names, import_names);
                            notif.send(Notif {
                                message: format!("transplanted {}", actor.name),
                                kind: Success,
                            });
                            // don't process mesh for transplanted actor for now
                            let (_, actor) = actor::Actor::new(map, insert).unwrap();
                            export_names[len] = actor.name.clone();
                            commands
                                .spawn((
                                    actor::Selected,
                                    MaterialMeshBundle {
                                        mesh: consts.cube.clone_weak(),
                                        material: consts.selected.clone_weak(),
                                        transform: actor.transform(map),
                                        ..default()
                                    },
                                    bevy::pbr::wireframe::NoWireframe,
                                    actor,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        consts.bounds.clone_weak(),
                                        SpatialBundle {
                                            visibility: Visibility::Hidden,
                                            ..default()
                                        },
                                        bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                                    ));
                                });
                        }
                    }
                });
                egui::ScrollArea::both().auto_shrink([false; 2]).show_rows(
                    ui,
                    ui.text_style_height(&egui::TextStyle::Body),
                    others.iter().len(),
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
            ("transplant", "ctrl + t"),
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
            ("deselect all", "escape"),
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
pub fn notifs(
    mut notif: EventReader<Notif>,
    mut notifs: ResMut<Notifs>,
    mut ctx: bevy_egui::EguiContexts,
) {
    for Notif { message, kind } in notif.read() {
        notifs
            .0
            .add(egui_notify::Toast::custom(message, kind.clone()));
    }
    notifs.0.show(ctx.ctx_mut());
}
