use super::*;

pub fn duplicate(
    _: Trigger<triggers::Duplicate>,
    mut notif: EventWriter<Notif>,
    mut commands: Commands,
    mut map: NonSendMut<Map>,
    registry: Res<Registry>,
    consts: Res<Constants>,
    selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
    mut cubes: Query<&mut Handle<wire::Wire>>,
) {
    let Some((map, _)) = &mut map.0 else { return };
    if selected.is_empty() {
        notif.send(Notif {
            message: "no actors to duplicate".into(),
            kind: Warning,
        });
        return;
    }
    for (entity, actor, ..) in selected.iter() {
        match cubes.get_mut(entity) {
            Ok(mut mat) => {
                commands.entity(entity).remove::<actor::Selected>();
                *mat = consts.unselected.clone_weak();
            }
            Err(_) => {
                commands.entity(entity).remove::<actor::SelectedBundle>();
            }
        }
        let insert =
            unreal_asset::types::PackageIndex::new(map.asset_data.exports.len() as i32 + 1);
        actor.duplicate(map);
        let (path, new) = actor::Actor::new(map, insert).unwrap();
        notif.send(Notif {
            message: format!("{} duplicated", actor.name),
            kind: Warning,
        });
        match path {
            Some(ref path) => {
                let (mesh, material) = &registry.meshes[path];
                commands.spawn((
                    actor::SelectedBundle::default(),
                    MaterialMeshBundle {
                        mesh: mesh.clone_weak(),
                        material: material
                            .as_ref()
                            .map(|mat| registry.mats[mat].clone_weak())
                            .unwrap_or(consts.grid.clone_weak()),
                        transform: actor.transform(map),
                        ..default()
                    },
                    bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                    new,
                ));
            }
            None => {
                commands
                    .spawn((
                        MaterialMeshBundle {
                            mesh: consts.cube.clone_weak(),
                            material: consts.unselected.clone_weak(),
                            transform: actor.transform(map),
                            ..default()
                        },
                        bevy::pbr::wireframe::NoWireframe,
                        new,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            SpatialBundle {
                                visibility: Visibility::Hidden,
                                ..default()
                            },
                            consts.bounds.clone_weak(),
                            bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                        ));
                    });
            }
        }
    }
}

pub fn delete(
    _: Trigger<triggers::Delete>,
    mut notif: EventWriter<Notif>,
    mut commands: Commands,
    mut map: NonSendMut<Map>,
    selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
) {
    let Some((map, _)) = &mut map.0 else { return };
    if selected.is_empty() {
        notif.send(Notif {
            message: "no actors to delete".into(),
            kind: Warning,
        });
        return;
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

pub fn focus(
    _: Trigger<triggers::Focus>,
    mut notif: EventWriter<Notif>,
    mut focus: ResMut<Focus>,
    selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
    mut camera: Query<&mut smooth_bevy_cameras::LookTransform, With<Camera3d>>,
) {
    if selected.is_empty() {
        notif.send(Notif {
            message: "no actors to focus".into(),
            kind: Warning,
        });
        return;
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
    focus.0 = Some(pos - camera.single().look_direction().unwrap_or_default() * sca.length() * 5.0)
}

pub fn approach(
    mut focus: ResMut<Focus>,
    mut camera: Query<&mut smooth_bevy_cameras::LookTransform, With<Camera3d>>,
) {
    let Some(target) = focus.0 else { return };
    let trans = &mut camera.single_mut().eye;
    *trans += target - *trans;
    if trans.distance(target) < 1.0 {
        focus.0 = None
    }
}

pub fn copy(
    _: Trigger<triggers::Copy>,
    mut notif: EventWriter<Notif>,
    mut buffer: ResMut<Buffer>,
    selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
) {
    if selected.is_empty() {
        notif.send(Notif {
            message: "no actors to copy location from".into(),
            kind: Warning,
        });
        return;
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

pub fn paste(
    _: Trigger<triggers::Paste>,
    mut notif: EventWriter<Notif>,
    mut map: NonSendMut<Map>,
    buffer: Res<Buffer>,
    mut selected: Query<(Entity, &actor::Actor, &mut Transform), With<actor::Selected>>,
) {
    let Some((map, _)) = &mut map.0 else { return };
    if selected.is_empty() {
        notif.send(Notif {
            message: "no actors to paste location to".into(),
            kind: Warning,
        });
        return;
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

pub fn deselect(
    _: Trigger<triggers::Deselect>,
    consts: Res<Constants>,
    selected: Query<Entity, With<actor::Selected>>,
    mut cubes: Query<&mut Handle<wire::Wire>>,
    mut commands: Commands,
) {
    for entity in selected.iter() {
        match cubes.get_mut(entity) {
            Ok(mut mat) => {
                commands.entity(entity).remove::<actor::Selected>();
                *mat = consts.unselected.clone_weak();
            }
            Err(_) => {
                commands.entity(entity).remove::<actor::SelectedBundle>();
            }
        }
    }
}

pub fn fullscreen(_: Trigger<triggers::Fullscreen>, mut windows: Query<&mut Window>) {
    use bevy::window::WindowMode;
    let mut window = windows.single_mut();
    window.mode = match window.mode {
        WindowMode::Windowed => WindowMode::BorderlessFullscreen,
        _ => WindowMode::Windowed,
    };
}

pub fn hide(_: Trigger<triggers::Hide>, mut hidden: ResMut<Hidden>) {
    hidden.0 = !hidden.0
}
