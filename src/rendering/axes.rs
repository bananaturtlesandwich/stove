use super::{size_of, Vert};
use eframe::wgpu::{util::DeviceExt, *};

pub struct Axes {
    vertices: Buffer,
    indices: Buffer,
    pipeline: RenderPipeline,
    bindings: BindGroup,
    uniform: Buffer,
}

impl Axes {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("axes.wgsl").into()),
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
                    glam::Vec3::X,
                    glam::Vec3::NEG_X,
                    glam::Vec3::Y,
                    glam::Vec3::NEG_Y,
                    glam::Vec3::Z,
                    glam::Vec3::NEG_Z,
                ]),
                usage: BufferUsages::VERTEX,
            }),
            indices: device.create_buffer_init(&util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[0, 1, 2, 3, 4, 5]),
                usage: BufferUsages::INDEX,
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
                    buffers: &[Vert::desc()],
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
        }
    }
}

impl Axes {
    pub fn copy(&mut self, vp: &glam::Mat4, queue: &Queue) {
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(vp));
    }
    pub fn draw<'a>(&'a self, filter: glam::Vec3, pass: &mut RenderPass<'a>) {
        let mut draw = |range| {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bindings, &[]);
            pass.set_index_buffer(self.indices.slice(..), IndexFormat::Uint32);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.draw_indexed(range, 0, 0..1);
        };
        if filter.x == 1.0 {
            draw(0..2)
        }
        if filter.y == 1.0 {
            draw(2..4)
        }
        if filter.z == 1.0 {
            draw(4..6)
        }
    }
}
