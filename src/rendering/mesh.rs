use egui_wgpu::wgpu::{self, util::DeviceExt, *};

use super::{size_of, Vert};

pub struct Mesh {
    vertices: Buffer,
    indices: Buffer,
    inst: Buffer,
    solid: RenderPipeline,
    wire: RenderPipeline,
    bindings: BindGroup,
    uniform: Buffer,
    len: u32,
    num: u32,
}

#[repr(C)]
#[derive(wrld::DescInstance, bytemuck::Pod, Clone, Copy, bytemuck::Zeroable)]
struct Inst {
    #[f32x4(1)]
    instx: [f32; 4],
    #[f32x4(2)]
    insty: [f32; 4],
    #[f32x4(3)]
    instz: [f32; 4],
    #[f32x4(4)]
    instw: [f32; 4],
}

impl Mesh {
    pub fn new(
        vertices: &[glam::Vec3],
        indices: &[u32],
        device: &Device,
        format: TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("mesh.wgsl").into()),
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
        let len = indices.len() as u32;
        Self {
            vertices: device.create_buffer_init(&util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX,
            }),
            indices: device.create_buffer_init(&util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(indices),
                usage: BufferUsages::INDEX,
            }),
            inst: device.create_buffer(&BufferDescriptor {
                label: None,
                size: 512 * 1024 * size_of::<glam::Mat4>(),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            solid: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bindings],
                    push_constant_ranges: &[],
                })),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vert",
                    buffers: &[Vert::desc(), Inst::desc()],
                },
                primitive: PrimitiveState {
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
                    entry_point: "solid",
                    targets: &[Some(format.into())],
                }),
                multiview: None,
            }),
            wire: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bindings],
                    push_constant_ranges: &[],
                })),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vert",
                    buffers: &[Vert::desc(), Inst::desc()],
                },
                primitive: PrimitiveState {
                    polygon_mode: PolygonMode::Line,
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
                    entry_point: "wire",
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
            len,
            num: 0,
        }
    }

    pub fn reset(&mut self) {
        self.num = 0
    }

    pub fn copy(&mut self, mat: glam::Mat4, vp: &glam::Mat4, queue: &Queue) {
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(vp));
        queue.write_buffer(
            &self.inst,
            self.num as u64 * size_of::<glam::Mat4>(),
            bytemuck::bytes_of(&Inst {
                instx: mat.x_axis.to_array(),
                insty: mat.y_axis.to_array(),
                instz: mat.z_axis.to_array(),
                instw: mat.w_axis.to_array(),
            }),
        );
        self.num += 1;
    }

    pub fn draw<'a>(&'a self, pass: &mut RenderPass<'a>) {
        let mut draw = |pipeline| {
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &self.bindings, &[]);
            pass.set_index_buffer(self.indices.slice(..), IndexFormat::Uint32);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_vertex_buffer(1, self.inst.slice(..));
            pass.draw_indexed(0..self.len, 0, 0..self.num);
        };
        draw(&self.solid);
        draw(&self.wire);
    }
}
// pub struct Mesh {
//     pipeline: Pipeline,
//     bindings: Vec<Bindings>,
// }

// impl Mesh {
//     pub fn new(
//         ctx: &mut Context,
//         vertices: Vec<glam::Vec3>,
//         indices: Vec<u32>,
//         uvs: Vec<Vec<(f32, f32)>>,
//         mats: Vec<(u32, u32, Vec<u8>)>,
//         mat_data: Vec<(u32, u32)>,
//     ) -> Self {
//         let mut mat_indices = Vec::with_capacity(mat_data.capacity());
//         let mut mat_data = mat_data.into_iter().peekable();
//         while let Some((mat, first)) = mat_data.next() {
//             mat_indices.push((
//                 mat as usize,
//                 first as usize,
//                 mat_data
//                     .peek()
//                     .map_or(indices.len(), |(_, first)| *first as usize),
//             ))
//         }
//         let shader = Shader::new(
//             ctx,
//             include_str!("mesh.vert"),
//             include_str!("mesh.frag"),
//             ShaderMeta {
//                 uniforms: UniformBlockLayout {
//                     uniforms: vec![UniformDesc::new("transform", UniformType::Mat4)],
//                 },
//                 images: vec!["tex".to_string()],
//             },
//         )
//         .unwrap();
//         Self {
//             pipeline: Pipeline::with_params(
//                 ctx,
//                 &[BufferLayout::default(), BufferLayout::default()],
//                 &[
//                     VertexAttribute::with_buffer("pos", VertexFormat::Float3, 0),
//                     VertexAttribute::with_buffer("texcoord", VertexFormat::Float2, 1),
//                 ],
//                 shader,
//                 PipelineParams {
//                     depth_test: Comparison::LessOrEqual,
//                     depth_write: true,
//                     primitive_type: PrimitiveType::Triangles,
//                     ..Default::default()
//                 },
//             ),
//             bindings: mat_indices
//                 .into_iter()
//                 .map(|(i, start, end)| Bindings {
//                     vertex_buffers: vec![
//                         Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices),
//                         Buffer::immutable(ctx, BufferType::VertexBuffer, &uvs[i]),
//                     ],
//                     index_buffer: Buffer::immutable(
//                         ctx,
//                         BufferType::IndexBuffer,
//                         &indices[start..end],
//                     ),
//                     images: vec![Texture::new(
//                         ctx,
//                         TextureAccess::Static,
//                         Some(mats[i].2.as_slice()),
//                         TextureParams {
//                             format: TextureFormat::RGBA8,
//                             wrap: TextureWrap::Repeat,
//                             filter: FilterMode::Linear,
//                             width: mats[i].0,
//                             height: mats[i].1,
//                         },
//                     )],
//                 })
//                 .collect(),
//         }
//     }

//     pub fn draw(&self, ctx: &mut Context, vp: glam::Mat4) {
//         for binding in self.bindings.iter() {
//             ctx.apply_pipeline(&self.pipeline);
//             ctx.apply_bindings(binding);
//             ctx.apply_uniforms(&vp);
//             ctx.draw(
//                 0,
//                 (binding.index_buffer.size() / std::mem::size_of::<usize>()) as i32,
//                 1,
//             )
//         }
//     }
// }
