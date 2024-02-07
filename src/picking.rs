use super::*;

pub fn pick(
    mut commands: Commands,
    mut drag: ResMut<Drag>,
    mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera: Query<&bevy_mod_raycast::deferred::RaycastSource<()>>,
    selected: Query<(Entity, &Transform), With<actor::Selected>>,
    mut ctx: bevy_egui::EguiContexts,
    mut action: EventWriter<Action>,
) {
    // EguiContexts isn't a ReadOnlySystemParam so can't make into a conditional
    if ctx.ctx_mut().is_pointer_over_area() {
        return;
    }
    if mouse.any_just_released([MouseButton::Left, MouseButton::Middle, MouseButton::Right]) {
        *drag = Drag::None
    }
    if let Some((entity, data)) = camera.single().get_nearest_intersection() {
        if selected.contains(entity) {
            if keys.any_just_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
                action.send(Action::Duplicate)
            }
            match &mouse {
                mouse if mouse.just_pressed(MouseButton::Left) => {
                    *drag = Drag::Translate(data.position())
                }
                mouse if mouse.just_pressed(MouseButton::Middle) => {
                    *drag = Drag::Scale(window.single().cursor_position().unwrap_or_default())
                }
                mouse if mouse.just_pressed(MouseButton::Right) => {
                    *drag = Drag::Rotate(
                        window.single().cursor_position().unwrap_or_default(),
                        Vec2::ZERO,
                    )
                }
                _ => (),
            }
        } else if mouse.just_pressed(MouseButton::Left) {
            commands
                .entity(entity)
                .insert(actor::SelectedBundle::default());
        }
    }
    if mouse.just_pressed(MouseButton::Left)
        && !keys.any_pressed([
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
        ])
        && matches!(drag.as_ref(), Drag::None)
    {
        for (entity, _) in selected.iter() {
            commands.entity(entity).remove::<actor::SelectedBundle>();
        }
    }
}

pub fn drag(
    mut drag: ResMut<Drag>,
    lock: Res<Lock>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut map: NonSendMut<Map>,
    camera: Query<(
        &bevy_mod_raycast::deferred::RaycastSource<()>,
        &smooth_bevy_cameras::LookTransform,
    )>,
    mut selected: Query<(&actor::Actor, &mut Transform), With<actor::Selected>>,
) {
    let Some((map, _)) = &mut map.0 else { return };
    let window = window.single();
    let camera = camera.single();
    match drag.as_mut() {
        Drag::None => (),
        Drag::Translate(pos) => {
            let Some(data) =
                camera
                    .0
                    .intersect_primitive(bevy_mod_raycast::primitives::Primitive3d::Plane {
                        point: *pos,
                        normal: match lock.as_ref() {
                            Lock::XYZ => camera.1.look_direction().unwrap_or_default(),
                            Lock::XY | Lock::X => Vec3::Z,
                            Lock::YZ | Lock::Y => Vec3::X,
                            Lock::ZX | Lock::Z => Vec3::Y,
                        },
                    })
            else {
                return;
            };
            let mut offset = data.position() - *pos;
            for (actor, mut transform) in selected.iter_mut() {
                match lock.as_ref() {
                    Lock::X => offset.y = 0.0,
                    Lock::Y => offset.z = 0.0,
                    Lock::Z => offset.x = 0.0,
                    _ => (),
                }
                actor.add_location(map, offset);
                transform.translation += offset;
            }
            *drag = Drag::Translate(data.position());
        }
        Drag::Rotate(start, prev) => {
            let current =
                (window.cursor_position().unwrap_or_default() - *start).normalize_or_zero();
            if *prev == Vec2::ZERO {
                *prev = current;
                return;
            }
            let angle = current.angle_between(*prev);
            *prev = current;
            let rotation = Quat::from_axis_angle(
                match lock.as_ref() {
                    Lock::XYZ => -camera.1.look_direction().unwrap_or_default(),
                    Lock::X | Lock::YZ => Vec3::X,
                    Lock::Y | Lock::ZX => Vec3::Y,
                    Lock::Z | Lock::XY => Vec3::Z,
                },
                angle,
            );
            for (actor, mut transform) in selected.iter_mut() {
                actor.combine_rotation(map, rotation);
                transform.rotation = rotation * transform.rotation;
            }
        }
        Drag::Scale(start) => {
            let current = window.cursor_position().unwrap_or_default();
            let centre = Vec2::new(window.width() / 2.0, window.height() / 2.0);
            let factor = (current - centre).length() / (*start - centre).length();
            *start = current;
            let scalar = match lock.as_ref() {
                Lock::XYZ => Vec3::splat(factor),
                Lock::XY => Vec3::new(factor, factor, 1.0),
                Lock::YZ => Vec3::new(1.0, factor, factor),
                Lock::ZX => Vec3::new(factor, 1.0, factor),
                Lock::X => Vec3::new(factor, 1.0, 1.0),
                Lock::Y => Vec3::new(1.0, factor, 1.0),
                Lock::Z => Vec3::new(1.0, 1.0, factor),
            };
            for (actor, mut transform) in selected.iter_mut() {
                actor.mul_scale(map, scalar);
                transform.scale *= scalar;
            }
        }
    }
}
