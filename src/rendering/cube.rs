use super::size_of;
use egui_wgpu::wgpu::{util::DeviceExt, *};

pub struct Cube {
    vertices: Buffer,
    indices: Buffer,
    inst: Buffer,
    pipeline: RenderPipeline,
    bindings: BindGroup,
    uniform: Buffer,
    num: u32,
}

#[repr(C)]
#[derive(bytemuck::Pod, Clone, Copy, bytemuck::Zeroable)]
struct Inst {
    mat: [[f32; 4]; 4],
    selected: f32,
}

impl Cube {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("cube.wgsl").into()),
        });
        let bindings = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(size_of::<glam::Mat4>()),
                },
                count: None,
            }],
        });
        let uniform = device.create_buffer(&BufferDescriptor {
            label: None,
            size: size_of::<glam::Mat4>(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertices:  device.create_buffer_init(&util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice::<f32, u8>(&[
                    // front verts
                    -0.5, -0.5, -0.5,
                    -0.5, 0.5, -0.5,
                    0.5, -0.5, -0.5,
                    0.5, 0.5, -0.5,
                    // back verts
                    -0.5, -0.5, 0.5,
                    -0.5, 0.5, 0.5,
                    0.5, -0.5, 0.5,
                    0.5, 0.5, 0.5,
                ]),
                usage: BufferUsages::VERTEX,
            }),
            indices: device.create_buffer_init(&util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[
                    0, 1, 0, 2, 1, 3, 2, 3, 4, 5, 4, 6, 5, 7, 6, 7, 4, 0, 5, 1, 6, 2, 7, 3,
                ]),
                usage: BufferUsages::INDEX,
            }),
            inst: device.create_buffer(&BufferDescriptor {
                label: None,
                size: 512 * 1024 * (size_of::<glam::Mat4>() + size_of::<f32>()),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            pipeline: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bindings],
                    push_constant_ranges: &[],
                })),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vert",
                    buffers: &[super::VERT,
                        VertexBufferLayout {
                            array_stride: size_of::<f32>() * 4 * 4 + size_of::<u32>(),
                            step_mode: VertexStepMode::Instance,
                            attributes: &vertex_attr_array![1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32],
                        },
                    ],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::LineList,
                    cull_mode: Some(Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: "frag",
                    targets: &[Some(format.into())],
                }),
                multiview: None,
            }),
            bindings: device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &bindings,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                }],
            }),
            uniform,
            num: 0,
        }
    }

    pub fn copy(&mut self, inst: &[(glam::Mat4, f32)], vp: &glam::Mat4, queue: &Queue) {
        let inst: Vec<_> = inst
            .iter()
            .map(|(mat, selected)| Inst {
                mat: mat.to_cols_array_2d(),
                selected: *selected,
            })
            .collect();
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(vp));
        queue.write_buffer(&self.inst, 0, bytemuck::cast_slice(&inst));
        self.num = inst.len() as u32;
    }

    pub fn draw<'a>(&'a self, pass: &mut RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bindings, &[]);
        pass.set_index_buffer(self.indices.slice(..), IndexFormat::Uint32);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        // again don't know a better way to do mat4 :p
        let chunk = self.inst.size() / 4;
        for i in 1..5 {
            pass.set_vertex_buffer(i as u32, self.inst.slice((i - 1) * chunk..i * chunk));
        }
        pass.set_vertex_buffer(5, self.inst.slice(..));
        pass.draw_indexed(0..24, 0, 0..self.num);
    }
}
