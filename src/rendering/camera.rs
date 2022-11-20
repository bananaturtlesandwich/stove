use miniquad::*;

// this isn't going to change so might as well just make it a constant
const PROJECTION: glam::Mat4 = glam::mat4(
    glam::vec4(1.0, 0.0, 0.0, 0.0),
    glam::vec4(0.0, 1.8, 0.0, 0.0),
    glam::vec4(0.0, 0.0, 1.0, 1.0),
    glam::vec4(0.0, 0.0, -1.0, 0.0),
);

pub struct Camera {
    position: glam::Vec3,
    can_move: bool,
    front: glam::Vec3,
    up: glam::Vec3,
    yaw: f32,
    pitch: f32,
    delta_time: f64,
    last_time: f64,
    last_mouse_pos: glam::Vec2,
    held_keys: Vec<KeyCode>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            delta_time: 0.0,
            last_time: date::now(),
            position: glam::Vec3::ZERO,
            can_move: false,
            up: glam::Vec3::Y,
            front: glam::vec3(0.0, 0.0, -1.0),
            pitch: 0.0,
            yaw: -90.0,
            last_mouse_pos: glam::vec2(0.0, 0.0),
            held_keys: Vec::new(),
        }
    }
}

impl Camera {
    pub fn projection(&self) -> glam::Mat4 {
        PROJECTION
    }
    pub fn update_times(&mut self) {
        let time = date::now();
        self.delta_time = time - self.last_time;
        self.last_time = time;
    }
    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_lh(self.position, self.position + self.front, self.up)
    }
    pub fn move_cam(&mut self) {
        if !self.can_move {
            return;
        }
        let velocity = (25.0 * self.delta_time) as f32;
        for keycode in self.held_keys.iter() {
            match keycode {
                KeyCode::W => self.position += self.front * velocity,
                KeyCode::A => self.position += self.front.cross(self.up).normalize() * velocity,
                KeyCode::S => self.position -= self.front * velocity,
                KeyCode::D => self.position -= self.front.cross(self.up).normalize() * velocity,
                KeyCode::E => self.position += glam::vec3(0.0, velocity, 0.0),
                KeyCode::Q => self.position -= glam::vec3(0.0, velocity, 0.0),
                _ => (),
            }
        }
    }
    pub fn set_focus(&mut self, pos: glam::Vec3) {
        self.position = pos - self.front * glam::Vec3::splat(4.0);
    }
    pub fn handle_key_down(&mut self, key: KeyCode) {
        if !self.held_keys.contains(&key) {
            self.held_keys.push(key)
        }
    }
    pub fn handle_key_up(&mut self, key: KeyCode) {
        if let Some(pos) = self.held_keys.iter().position(|k| k == &key) {
            self.held_keys.remove(pos);
        }
    }
    pub fn handle_mouse_motion(&mut self, x: f32, y: f32) {
        if self.can_move {
            let delta = glam::vec2(x - self.last_mouse_pos.x, y - self.last_mouse_pos.y);
            let scale = (10.0 * self.delta_time) as f32;
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
        self.last_mouse_pos = glam::vec2(x, y);
    }
    pub fn handle_mouse_down(&mut self, button: MouseButton) {
        self.can_move = button == MouseButton::Right;
    }
    pub fn handle_mouse_up(&mut self, button: MouseButton) {
        if button == MouseButton::Right {
            self.can_move = false;
        }
    }
}
