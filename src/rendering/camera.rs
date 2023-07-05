use miniquad::*;

pub struct Camera {
    pub position: glam::Vec3,
    can_move: bool,
    pub front: glam::Vec3,
    pub up: glam::Vec3,
    yaw: f32,
    pitch: f32,
    delta_time: f64,
    last_time: f64,
    pub speed: u16,
    focus: Option<glam::Vec3>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            delta_time: 0.0,
            last_time: date::now(),
            position: glam::Vec3::ZERO,
            can_move: false,
            up: glam::Vec3::Y,
            front: glam::Vec3::NEG_Z,
            pitch: 0.0,
            yaw: -90.0,
            speed: 25,
            focus: None,
        }
    }
}

impl Camera {
    pub fn update_times(&mut self) {
        let time = date::now();
        self.delta_time = time - self.last_time;
        self.last_time = time;
    }
    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_lh(self.position, self.position + self.front, self.up)
    }
    pub fn left(&self) -> glam::Vec3 {
        self.front.cross(self.up).normalize()
    }
    pub fn move_cam(&mut self, held: &[KeyCode]) {
        match self.focus {
            Some(target) => {
                self.position += self.delta_time as f32 * 10.0 * (target - self.position);
                if self.position.distance(target) < 1.0 {
                    self.focus = None;
                }
            }
            None => {
                let velocity = self.speed as f32 * self.delta_time as f32;
                for keycode in held.iter() {
                    match keycode {
                        KeyCode::W => self.position += self.front * velocity,
                        KeyCode::A => self.position += self.left() * velocity,
                        KeyCode::S => self.position -= self.front * velocity,
                        KeyCode::D => self.position -= self.left() * velocity,
                        KeyCode::E => self.position += glam::vec3(0.0, velocity, 0.0),
                        KeyCode::Q => self.position -= glam::vec3(0.0, velocity, 0.0),
                        _ => (),
                    }
                }
            }
        }
    }
    pub fn set_focus(&mut self, pos: glam::Vec3, sca: glam::Vec3) {
        self.focus = Some(pos - self.front * sca.length() * 4.0)
    }
    pub fn handle_mouse_motion(&mut self, delta: glam::Vec2) {
        if self.can_move {
            let scale = 10.0 * self.delta_time as f32;
            self.yaw -= delta.x * scale;
            self.pitch -= delta.y * scale;
            self.pitch = self.pitch.clamp(-89.0, 89.0);
            let front_pitch = self.pitch.to_radians().sin_cos();
            let front_yaw = self.yaw.to_radians().sin_cos();
            self.front = glam::vec3(
                front_pitch.1 * front_yaw.1,
                front_pitch.0,
                front_pitch.1 * front_yaw.0,
            )
            .normalize();
        }
    }
    pub fn enable_move(&mut self) {
        self.can_move = true;
    }
    pub fn disable_move(&mut self) {
        self.can_move = false;
    }
}
