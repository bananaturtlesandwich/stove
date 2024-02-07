use super::*;

pub fn shortcuts(
    mut lock: ResMut<Lock>,
    mut dialog: EventWriter<Dialog>,
    mut action: EventWriter<Action>,
    keys: Res<Input<KeyCode>>,
) {
    let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if keys.just_released(KeyCode::O) && ctrl {
        dialog.send(Dialog::Open(None))
    }
    if keys.just_released(KeyCode::S) && ctrl {
        dialog.send(Dialog::SaveAs(
            keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        ))
    }
    if keys.just_released(KeyCode::Delete) {
        action.send(Action::Delete)
    }
    if keys.just_released(KeyCode::T) && ctrl {
        dialog.send(Dialog::Transplant)
    }
    if keys.just_released(KeyCode::F) {
        action.send(Action::Focus)
    }
    if keys.just_pressed(KeyCode::X) {
        *lock = match shift {
            true => Lock::YZ,
            false => Lock::X,
        }
    } else if keys.just_pressed(KeyCode::Y) {
        *lock = match shift {
            true => Lock::ZX,
            false => Lock::Y,
        }
    } else if keys.just_pressed(KeyCode::Z) {
        *lock = match shift {
            true => Lock::XY,
            false => Lock::Z,
        }
    }
    if keys.any_just_released([KeyCode::X, KeyCode::Y, KeyCode::Z]) {
        *lock = Lock::XYZ
    }
}

// an edited version of the original default input map
pub fn camera(
    mut events: EventWriter<smooth_bevy_cameras::controllers::unreal::ControlEvent>,
    mut wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut motion: EventReader<bevy::input::mouse::MouseMotion>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    drag: Res<Drag>,
    mut controllers: Query<&mut smooth_bevy_cameras::controllers::unreal::UnrealCameraController>,
    mut ctx: bevy_egui::EguiContexts,
) {
    if ctx.ctx_mut().is_pointer_over_area() {
        return;
    }
    let mut controller = controllers.single_mut();
    let smooth_bevy_cameras::controllers::unreal::UnrealCameraController {
        rotate_sensitivity: mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        wheel_translate_sensitivity,
        mut keyboard_mvmt_sensitivity,
        keyboard_mvmt_wheel_sensitivity,
        ..
    } = *controller;

    let right_pressed = mouse.pressed(MouseButton::Right);
    let middle_pressed =
        mouse.pressed(MouseButton::Middle) && !matches!(drag.as_ref(), Drag::Scale(_));

    let mut cursor_delta = Vec2::ZERO;
    for event in motion.read() {
        cursor_delta += event.delta;
    }

    let mut wheel_delta = 0.0;
    for event in wheel.read() {
        wheel_delta += event.x + event.y;
    }

    let mut panning_dir = Vec2::ZERO;
    let mut translation_dir = Vec2::ZERO; // y is forward/backward axis, x is rotation around Z

    for key in keyboard.get_pressed() {
        match key {
            KeyCode::W => translation_dir.y += 1.0,
            KeyCode::A => panning_dir.x -= 1.0,
            KeyCode::S => translation_dir.y -= 1.0,
            KeyCode::D => panning_dir.x += 1.0,
            KeyCode::E => panning_dir.y += 1.0,
            KeyCode::Q => panning_dir.y -= 1.0,
            _ => {}
        }
    }

    let mut panning = Vec2::ZERO;
    let mut locomotion = Vec2::ZERO;

    if right_pressed {
        panning += keyboard_mvmt_sensitivity * panning_dir;

        if translation_dir.y != 0.0 {
            locomotion.y += keyboard_mvmt_sensitivity * translation_dir.y;
        }

        keyboard_mvmt_sensitivity += keyboard_mvmt_wheel_sensitivity * wheel_delta;
        controller.keyboard_mvmt_sensitivity = keyboard_mvmt_sensitivity.max(0.01);
    }

    if wheel_delta != 0.0 {
        locomotion.y += wheel_translate_sensitivity * wheel_delta;
    }

    if middle_pressed {
        // for some reason y needs inversion
        panning += mouse_translate_sensitivity * bevy::math::vec2(cursor_delta.x, -cursor_delta.y);
    }

    use smooth_bevy_cameras::controllers::unreal::ControlEvent;

    if right_pressed {
        events.send(ControlEvent::Rotate(
            mouse_rotate_sensitivity * cursor_delta,
        ));
    }

    if panning.length_squared() > 0.0 {
        events.send(ControlEvent::TranslateEye(panning));
    }

    if locomotion.length_squared() > 0.0 {
        events.send(ControlEvent::Locomotion(locomotion));
    }
}
