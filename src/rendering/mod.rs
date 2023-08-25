mod axes;
mod camera;
mod cube;
mod mesh;
use egui_wgpu::wgpu::*;
pub use {axes::*, camera::*, cube::*, mesh::*};

const fn size_of<T>() -> u64 {
    std::mem::size_of::<T>() as u64
}

const VERT: VertexBufferLayout = VertexBufferLayout {
    array_stride: size_of::<f32>() * 3,
    step_mode: VertexStepMode::Vertex,
    attributes: &vertex_attr_array![0 => Float32x3],
};
