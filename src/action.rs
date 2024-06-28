use super::*;

pub fn follow(
    mut actions: EventReader<Action>,
    mut notif: EventWriter<Notif>,
    mut commands: Commands,
    mut map: NonSendMut<Map>,
    mut focus: ResMut<Focus>,
    mut buffer: ResMut<Buffer>,
    registry: Res<Registry>,
    consts: Res<Constants>,
    mut selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
    children: Query<&Children>,
    mut cubes: Query<&mut Handle<wire::Wire>>,
    mut camera: Query<&mut smooth_bevy_cameras::LookTransform, With<Camera3d>>,
) {
    let Some((map, _)) = &mut map.0 else { return };
    for action in actions.read() {
        match action {
            Action::Duplicate => {
                if selected.is_empty() {
                    notif.send(Notif {
                        message: "no actors to duplicate".into(),
                        kind: Warning,
                    });
                    continue;
                }
                for (entity, actor, ..) in selected.iter() {
                    match children.get(entity) {
                        Ok(children) => {
                            if let Some(mut mat) = children
                                .first()
                                .and_then(|child| cubes.get_mut(*child).ok())
                            {
                                commands.entity(entity).remove::<actor::Selected>();
                                *mat = consts.unselected.clone_weak();
                            }
                        }
                        Err(_) => {
                            commands.entity(entity).remove::<actor::SelectedBundle>();
                        }
                    }
                    let insert = unreal_asset::types::PackageIndex::new(
                        map.asset_data.exports.len() as i32 + 1,
                    );
                    actor.duplicate(map);
                    let new = actor::Actor::new(map, insert).unwrap();
                    notif.send(Notif {
                        message: format!("{} duplicated", actor.name),
                        kind: Warning,
                    });
                    match &actor.draw_type {
                        actor::DrawType::Mesh(path) => {
                            let (mesh, material) = &registry.0[path];
                            commands.spawn((
                                actor::SelectedBundle::default(),
                                MaterialMeshBundle {
                                    mesh: mesh.clone_weak(),
                                    material: material
                                        .first()
                                        .map(Handle::clone_weak)
                                        .unwrap_or(consts.grid.clone_weak()),
                                    transform: actor.transform(map),
                                    ..default()
                                },
                                bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                                new,
                            ));
                        }
                        actor::DrawType::Cube => {
                            commands
                                .spawn((
                                    SpatialBundle {
                                        transform: actor.transform(map),
                                        visibility: Visibility::Hidden,
                                        ..default()
                                    },
                                    consts.bounds.clone_weak(),
                                    bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                                    new, // child because it's LineList which picking can't do
                                ))
                                .with_children(|parent| {
                                    parent.spawn(MaterialMeshBundle {
                                        mesh: consts.cube.clone_weak(),
                                        material: consts.unselected.clone_weak(),
                                        visibility: Visibility::Visible,
                                        ..default()
                                    });
                                });
                        }
                    }
                }
            }
            Action::Delete => {
                if selected.is_empty() {
                    notif.send(Notif {
                        message: "no actors to delete".into(),
                        kind: Warning,
                    });
                    continue;
                }
                for (entity, actor, ..) in selected.iter() {
                    actor.delete(map);
                    notif.send(Notif {
                        message: format!("{} deleted", actor.name),
                        kind: Warning,
                    });
                    commands.entity(entity).despawn_recursive()
                }
            }
            Action::Focus => {
                if selected.is_empty() {
                    notif.send(Notif {
                        message: "no actors to focus".into(),
                        kind: Warning,
                    });
                    continue;
                }
                let (mut pos, mut sca) = selected
                    .iter()
                    .fold((Vec3::ZERO, Vec3::ZERO), |(pos, sca), (_, _, trans)| {
                        (pos + trans.translation, sca + trans.scale)
                    });
                let len = selected.iter().len() as f32;
                pos /= len;
                sca /= len;
                camera.single_mut().target = pos;
                focus.0 = Some(
                    pos - camera.single().look_direction().unwrap_or_default() * sca.length() * 5.0,
                )
            }
            Action::Copy => {
                if selected.is_empty() {
                    notif.send(Notif {
                        message: "no actors to copy location from".into(),
                        kind: Warning,
                    });
                    continue;
                }
                buffer.0 = selected
                    .iter()
                    .fold(Vec3::ZERO, |pos, (_, _, trans)| pos + trans.translation)
                    / selected.iter().len() as f32;
                notif.send(Notif {
                    message: "location copied".into(),
                    kind: Success,
                });
            }
            Action::Paste => {
                if selected.is_empty() {
                    notif.send(Notif {
                        message: "no actors to paste location to".into(),
                        kind: Warning,
                    });
                    continue;
                }
                let offset = buffer.0
                    - selected
                        .iter()
                        .fold(Vec3::ZERO, |pos, (_, _, trans)| pos + trans.translation)
                        / selected.iter().len() as f32;
                for (_, actor, mut trans) in selected.iter_mut() {
                    actor.add_location(map, offset);
                    trans.translation += offset;
                }
                notif.send(Notif {
                    message: "location pasted".into(),
                    kind: Success,
                });
            }
        }
    }
    let Some(target) = focus.0 else { return };
    let trans = &mut camera.single_mut().eye;
    *trans += target - *trans;
    if trans.distance(target) < 1.0 {
        focus.0 = None
    }
}
