#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use egui_wgpu::wgpu;

#[pollster::main]
async fn main() {
    let events = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("stove")
        .with_window_icon(Some(
            winit::window::Icon::from_rgba(include_bytes!("../assets/pot.rgba").to_vec(), 64, 64)
                .unwrap(),
        ))
        .build(&events)
        .unwrap();
    let instance = wgpu::Instance::default();
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::POLYGON_MODE_LINE,
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .unwrap();
    let caps = surface.get_capabilities(&adapter);
    let format = caps.formats[0];
    let mut size = window.inner_size();
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);
    let (mut tex, mut depthview, mut sampler) = make_depth(&device, &config);
    let mut screen = egui_wgpu::renderer::ScreenDescriptor {
        size_in_pixels: [size.width, size.height],
        pixels_per_point: window.scale_factor() as f32,
    };
    let mut platform = egui_winit::State::new(&window);
    let mut ctx = egui::Context::default();
    let mut ui =
        egui_wgpu::Renderer::new(&device, format, Some(wgpu::TextureFormat::Depth32Float), 1);
    let mut app = stove::Stove::new(&mut ctx, &device, format);
    events.run(move |event, _, flow| {
        use winit::{
            event::{Event, WindowEvent},
            event_loop::ControlFlow,
        };
        if let Event::WindowEvent { event, .. } = &event {
            let _ = platform.on_event(&ctx, event);
        }
        match event {
            Event::RedrawRequested(_) => {
                let frame = surface.get_current_texture().unwrap();
                let mut encoder = device.create_command_encoder(&Default::default());
                let view = frame.texture.create_view(&Default::default());
                let jobs = app.show_ui().then(|| {
                    let output = ctx.run(platform.take_egui_input(&window), |ctx| {
                        app.ui(ctx, &device, format, &size)
                    });
                    let jobs = ctx.tessellate(output.shapes);
                    platform.handle_platform_output(&window, &ctx, output.platform_output);
                    for (tex, delta) in output.textures_delta.set {
                        ui.update_texture(&device, &queue, tex, &delta)
                    }
                    for tex in output.textures_delta.free {
                        ui.free_texture(&tex)
                    }
                    ui.update_buffers(&device, &queue, &mut encoder, &jobs, &screen);
                    jobs
                });
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depthview,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });
                app.scene(&queue, &mut pass, &size);
                if let Some(jobs) = jobs {
                    ui.render(&mut pass, &jobs, &screen)
                }
                drop(pass);
                queue.submit(Some(encoder.finish()));
                frame.present();
                window.request_redraw();
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                event: WindowEvent::Resized(new),
                ..
            } => {
                if new.width == 0 || new.height == 0 {
                    return;
                }
                size = new;
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                (tex, depthview, sampler) = make_depth(&device, &config);
                screen = egui_wgpu::renderer::ScreenDescriptor {
                    size_in_pixels: [size.width, size.height],
                    pixels_per_point: window.scale_factor() as f32,
                };
                #[cfg(target_os = "macos")]
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                ctx.memory_mut(|storage| app.store(&mut storage.data));
                app.close();
                *flow = ControlFlow::Exit
            }
            _ => (),
        }
    });
}

fn make_depth(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });
    (tex, view, sampler)
}
