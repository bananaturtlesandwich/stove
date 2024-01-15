use super::*;

pub fn follow(
    mut actions: EventReader<Action>,
    mut commands: Commands,
    mut map: NonSendMut<Map>,
    selected: Query<(Entity, &actor::Actor), With<actor::Selected>>,
) {
    let Some((map, _)) = &mut map.0 else { return };
    for action in actions.read() {
        match action {
            Action::Duplicate => todo!(),
            Action::Delete => {
                for (entity, actor) in selected.iter() {
                    actor.delete(map);
                    commands.entity(entity).despawn_recursive()
                }
            }
            Action::Transplant => todo!(),
        }
    }
}
