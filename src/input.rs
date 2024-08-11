use super::*;

pub fn shortcuts(
    mut commands: Commands,
    mut lock: ResMut<Lock>,
    keys: Res<ButtonInput<KeyCode>>,
    mut ctx: bevy_egui::EguiContexts,
) {
    let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    if keys.just_released(KeyCode::KeyO) && ctrl {
        commands.trigger(triggers::Open(None));
    }
    if keys.just_released(KeyCode::KeyO) && keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
    {
        commands.trigger(triggers::AddPak);
    }
    if keys.just_released(KeyCode::KeyS) && ctrl {
        commands.trigger(triggers::SaveAs(shift));
    }
    if keys.just_released(KeyCode::KeyT) && ctrl {
        commands.trigger(triggers::Transplant);
    }
    if keys.just_pressed(KeyCode::KeyX) {
        *lock = match shift {
            true => Lock::YZ,
            false => Lock::X,
        }
    } else if keys.just_pressed(KeyCode::KeyY) {
        *lock = match shift {
            true => Lock::ZX,
            false => Lock::Y,
        }
    } else if keys.just_pressed(KeyCode::KeyZ) {
        *lock = match shift {
            true => Lock::XY,
            false => Lock::Z,
        }
    }
    if keys.any_just_released([KeyCode::KeyX, KeyCode::KeyY, KeyCode::KeyZ]) {
        *lock = Lock::XYZ
    }
    if ctx.ctx_mut().wants_keyboard_input() {
        return;
    }
    if keys.just_released(KeyCode::Delete) {
        commands.trigger(triggers::Delete);
    }
    if keys.just_released(KeyCode::KeyF) {
        commands.trigger(triggers::Focus);
    }
    if keys.just_released(KeyCode::KeyC) && ctrl {
        commands.trigger(triggers::Copy);
    }
    if keys.just_released(KeyCode::KeyV) && ctrl {
        commands.trigger(triggers::Paste);
    }
    if keys.just_released(KeyCode::Escape) {
        commands.trigger(triggers::Deselect);
    }
    if keys.just_released(KeyCode::Enter) && alt {
        commands.trigger(triggers::Fullscreen)
    }
    if keys.just_released(KeyCode::KeyH) {
        commands.trigger(triggers::Hide)
    }
}

// an edited version of the original default input map
pub fn camera(
    mut events: EventWriter<smooth_bevy_cameras::controllers::unreal::ControlEvent>,
    mut wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut motion: EventReader<bevy::input::mouse::MouseMotion>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
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

    let right_pressed =
        mouse.pressed(MouseButton::Right) && !matches!(drag.as_ref(), Drag::Rotate(..));
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
            KeyCode::KeyW => translation_dir.y += 1.0,
            KeyCode::KeyA => panning_dir.x -= 1.0,
            KeyCode::KeyS => translation_dir.y -= 1.0,
            KeyCode::KeyD => panning_dir.x += 1.0,
            KeyCode::KeyE => panning_dir.y += 1.0,
            KeyCode::KeyQ => panning_dir.y -= 1.0,
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
