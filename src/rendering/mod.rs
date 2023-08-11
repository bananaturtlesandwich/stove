mod axes;
mod camera;
mod cube;
mod mesh;
pub use {axes::*, camera::*, cube::*, mesh::*};

fn size_of<T>() -> u64 {
    std::mem::size_of::<T>() as u64
}

use eframe::wgpu;
#[repr(C)]
#[derive(wrld::Desc, bytemuck::Pod, Clone, Copy, bytemuck::Zeroable)]
struct Vert {
    #[f32x3(0)]
    pos: [f32; 3],
}

impl Vert {
    const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { pos: [x, y, z] }
    }
}
