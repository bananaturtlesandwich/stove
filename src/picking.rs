use super::*;

pub fn pick(
    mut commands: Commands,
    mut drag: ResMut<Drag>,
    consts: Res<Constants>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera: Query<&bevy_mod_raycast::deferred::RaycastSource<()>>,
    selected: Query<(Entity, &Transform), With<actor::Selected>>,
    parents: Query<&Parent>,
    mut cubes: Query<&mut Handle<wire::Wire>>,
    mut ctx: bevy_egui::EguiContexts,
) {
    // EguiContexts isn't a ReadOnlySystemParam so can't make into a conditional
    if ctx.ctx_mut().is_pointer_over_area() {
        return;
    }
    if mouse.any_just_released([MouseButton::Left, MouseButton::Middle, MouseButton::Right]) {
        *drag = Drag::None
    }
    if let Some((entity, data)) = camera.single().get_nearest_intersection() {
        if selected.contains(entity)
            || parents
                .get(entity)
                .is_ok_and(|parent| selected.contains(parent.get()))
        {
            match &mouse {
                mouse if mouse.just_pressed(MouseButton::Left) => {
                    if keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
                        commands.trigger(triggers::Duplicate);
                    }
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
            match parents.get(entity) {
                Ok(parent) => {
                    if let Ok(mut mat) = cubes.get_mut(parent.get()) {
                        commands.entity(parent.get()).insert(actor::Selected);
                        *mat = consts.selected.clone_weak();
                    }
                }
                Err(_) => {
                    commands
                        .entity(entity)
                        .insert(actor::SelectedBundle::default());
                }
            }
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
            let Some(ray) = camera.0.ray else { return };
            let Some(dist) = ray.intersect_plane(
                *pos,
                InfinitePlane3d::new(match lock.as_ref() {
                    Lock::XYZ => camera.1.look_direction().unwrap_or_default(),
                    Lock::XY | Lock::X => Vec3::Z,
                    Lock::YZ | Lock::Y => Vec3::X,
                    Lock::ZX | Lock::Z => Vec3::Y,
                }),
            ) else {
                return;
            };
            let hit = ray.origin + ray.direction * dist;
            let mut offset = hit - *pos;
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
            *drag = Drag::Translate(hit);
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
