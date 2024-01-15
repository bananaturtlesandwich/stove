use super::*;

pub fn follow(
    mut actions: EventReader<Action>,
    mut notif: EventWriter<Notif>,
    mut commands: Commands,
    mut map: NonSendMut<Map>,
    selected: Query<(Entity, &actor::Actor), With<actor::Selected>>,
    registry: Res<Registry>,
    consts: Res<Constants>,
) {
    let Some((map, _)) = &mut map.0 else { return };
    for action in actions.read() {
        match action {
            Action::Duplicate => {
                if selected.is_empty() {
                    notif.send(Notif {
                        message: "no actors to duplicate".into(),
                        kind: Warning,
                    })
                }
                for (entity, actor) in selected.iter() {
                    commands.entity(entity).remove::<actor::SelectedBundle>();
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
                                    actor::SelectedBundle::default(),
                                    PbrBundle {
                                        mesh: consts.bounds.clone_weak(),
                                        transform: actor.transform(map),
                                        visibility: Visibility::Hidden,
                                        ..default()
                                    },
                                    bevy_mod_raycast::deferred::RaycastMesh::<()>::default(),
                                    new, // child because it's LineList which picking can't do
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
                }
            }
            Action::Delete => {
                if selected.is_empty() {
                    notif.send(Notif {
                        message: "no actors to delete".into(),
                        kind: Warning,
                    })
                }
                for (entity, actor) in selected.iter() {
                    actor.delete(map);
                    notif.send(Notif {
                        message: format!("{} deleted", actor.name),
                        kind: Warning,
                    });
                    commands.entity(entity).despawn_recursive()
                }
            }
        }
    }
}
