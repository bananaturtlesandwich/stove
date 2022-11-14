use miniquad::*;

pub struct Camera {
    position: glam::Vec3,
    view: bool,
    front: glam::Vec3,
    up: glam::Vec3,
    yaw: f32,
    pitch: f32,
    delta_time: f64,
    last_time: f64,
    last_pos: glam::Vec2,
    held: Vec<KeyCode>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            delta_time: 0.0,
            last_time: date::now(),
            position: glam::Vec3::ZERO,
            view: false,
            up: glam::Vec3::Y,
            front: glam::vec3(0.0, 0.0, -1.0),
            pitch: 0.0,
            yaw: -90.0,
            last_pos: glam::vec2(0.0, 0.0),
            held: Vec::new(),
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
        glam::Mat4::look_at_rh(self.position, self.position + self.front, self.up)
    }
    pub fn move_cam(&mut self) {
        if !self.view {
            return;
        }
        let velocity = (5.0 * self.delta_time) as f32;
        for keycode in self.held.iter() {
            match keycode {
                KeyCode::W => self.position += self.front * velocity,
                KeyCode::A => self.position -= self.front.cross(self.up).normalize() * velocity,
                KeyCode::S => self.position -= self.front * velocity,
                KeyCode::D => self.position += self.front.cross(self.up).normalize() * velocity,
                KeyCode::E => self.position += glam::vec3(0.0, velocity, 0.0),
                KeyCode::Q => self.position -= glam::vec3(0.0, velocity, 0.0),
                _ => (),
            }
        }
    }
    pub fn handle_key_down(&mut self, key: KeyCode) {
        if !self.held.contains(&key) {
            self.held.push(key)
        }
    }
    pub fn handle_key_up(&mut self, key: KeyCode) {
        if let Some(pos) = self.held.iter().position(|k| k == &key) {
            self.held.remove(pos);
        }
    }
    pub fn handle_mouse_motion(&mut self, x: f32, y: f32) {
        if self.view {
            let delta = glam::vec2(x - self.last_pos.x, y - self.last_pos.y);
            let scale = (10.0 * self.delta_time) as f32;
            self.yaw += (delta.x * scale).clamp(-89.0, 89.0);
            self.pitch -= delta.y * scale;
            let front_pitch = self.pitch.to_radians().sin_cos();
            let front_yaw = self.yaw.to_radians().sin_cos();
            self.front = glam::vec3(
                front_pitch.1 * front_yaw.1,
                front_pitch.0,
                front_pitch.1 * front_yaw.0,
            )
            .normalize();
        }
        self.last_pos = glam::vec2(x, y);
    }
    pub fn handle_mouse_down(&mut self, button: MouseButton) {
        self.view = button == MouseButton::Right;
    }
    pub fn handle_mouse_up(&mut self, button: MouseButton) {
        if button == MouseButton::Right {
            self.view = false;
        }
    }
}
