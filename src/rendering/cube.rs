use eframe::wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

pub struct Cube {
    vertices: Buffer,
    indices: Buffer,
    inst: Buffer,
    selected: Buffer,
    pipeline: RenderPipeline,
    bindings: BindGroup,
    uniform: Buffer,
    num: u32,
}

fn size_of<T>() -> u64 {
    std::mem::size_of::<T>() as u64
}

#[repr(C)]
#[derive(bytemuck::Pod, Clone, Copy, bytemuck::Zeroable)]
struct Inst {
    inst: [[f32; 4]; 4],
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
            vertices: device.create_buffer_init(&util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[
                    // front verts
                    glam::vec3(-0.5, -0.5, -0.5),
                    glam::vec3(-0.5, 0.5, -0.5),
                    glam::vec3(0.5, -0.5, -0.5),
                    glam::vec3(0.5, 0.5, -0.5),
                    // back verts
                    glam::vec3(-0.5, -0.5, 0.5),
                    glam::vec3(-0.5, 0.5, 0.5),
                    glam::vec3(0.5, -0.5, 0.5),
                    glam::vec3(0.5, 0.5, 0.5),
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
                size: 512 * 1024 * size_of::<glam::Mat4>(),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            selected: device.create_buffer(&BufferDescriptor {
                label: None,
                size: 512 * 1024 * size_of::<f32>(),
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
                    buffers: &[
                        VertexBufferLayout {
                            array_stride: size_of::<glam::Vec3>(),
                            step_mode: VertexStepMode::Vertex,
                            attributes: &[VertexAttribute {
                                format: VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            }],
                        },
                        VertexBufferLayout {
                            array_stride: size_of::<glam::Mat4>(),
                            step_mode: VertexStepMode::Instance,
                            attributes: &[
                                // is this the best way to do matrices?
                                VertexAttribute {
                                    format: VertexFormat::Float32x4,
                                    offset: 0,
                                    shader_location: 1,
                                },
                                VertexAttribute {
                                    format: VertexFormat::Float32x4,
                                    offset: size_of::<glam::Vec4>(),
                                    shader_location: 2,
                                },
                                VertexAttribute {
                                    format: VertexFormat::Float32x4,
                                    offset: size_of::<glam::Vec4>() * 2,
                                    shader_location: 3,
                                },
                                VertexAttribute {
                                    format: VertexFormat::Float32x4,
                                    offset: size_of::<glam::Vec4>() * 3,
                                    shader_location: 4,
                                },
                            ],
                        },
                        VertexBufferLayout {
                            array_stride: size_of::<f32>(),
                            step_mode: VertexStepMode::Instance,
                            attributes: &[VertexAttribute {
                                format: VertexFormat::Float32,
                                offset: 0,
                                shader_location: 5,
                            }],
                        },
                    ],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::LineList,
                    cull_mode: Some(Face::Back),
                    ..Default::default()
                },
                depth_stencil: None,
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

    pub fn copy(
        &mut self,
        (inst, selected): &(Vec<glam::Mat4>, Vec<f32>),
        // just a slice so it's easier to cast
        vp: &[glam::Mat4],
        queue: &Queue,
    ) {
        let inst: Vec<_> = inst.into_iter().map(|mat| mat.to_cols_array_2d()).collect();
        queue.write_buffer(&self.uniform, 0, bytemuck::cast_slice(vp));
        queue.write_buffer(&self.inst, 0, bytemuck::cast_slice(&inst));
        queue.write_buffer(&self.selected, 0, bytemuck::cast_slice(&selected));
        self.num = selected.len() as u32;
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
