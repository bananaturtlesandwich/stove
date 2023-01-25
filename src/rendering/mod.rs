mod camera;
mod cube;
mod mesh;
pub use {camera::*, cube::*, mesh::*};

// this isn't going to change so might as well just make it a constant
pub const PROJECTION: glam::Mat4 = glam::mat4(
    glam::vec4(1.0, 0.0, 0.0, 0.0),
    glam::vec4(0.0, 1.8, 0.0, 0.0),
    glam::vec4(0.0, 0.0, 1.0, 1.0),
    glam::vec4(0.0, 0.0, -1.0, 0.0),
);
