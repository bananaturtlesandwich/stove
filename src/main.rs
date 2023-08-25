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
                features: wgpu::Features::POLYGON_MODE_LINE
                    | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .unwrap();
    let caps = surface.get_capabilities(&adapter);
    let format = caps
        .formats
        .iter()
        .copied()
        .find(|f| !f.is_srgb())
        .unwrap_or(caps.formats[0]);
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
    let samples = match adapter.get_texture_format_features(format).flags {
        flags if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X16) => 16,
        flags if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X8) => 8,
        flags if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4) => 4,
        flags if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X2) => 2,
        _ => 1,
    };
    let (mut depth, mut tex) = make_tex(&device, &config, samples, format);
    let mut screen = egui_wgpu::renderer::ScreenDescriptor {
        size_in_pixels: [size.width, size.height],
        pixels_per_point: window.scale_factor() as f32,
    };
    let mut platform = egui_winit::State::new(&window);
    let mut ctx = egui::Context::default();
    let mut ui = egui_wgpu::Renderer::new(
        &device,
        format,
        Some(wgpu::TextureFormat::Depth32Float),
        samples,
    );
    let mut app = stove::Stove::new(&mut ctx, &device, format, samples);
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
                let Ok(frame) = surface.get_current_texture() else {
                    return;
                };
                let mut encoder = device.create_command_encoder(&Default::default());
                let jobs = app.show_ui().then(|| {
                    let output = ctx.run(platform.take_egui_input(&window), |ctx| {
                        app.ui(ctx, &device, format, samples, &size)
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
                let resolve = frame.texture.create_view(&Default::default());
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &tex,
                        resolve_target: Some(&resolve),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.15,
                                g: 0.15,
                                b: 0.15,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth,
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
                (depth, tex) = make_tex(&device, &config, samples, format);
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

fn make_tex(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    samples: u32,
    format: wgpu::TextureFormat,
) -> (wgpu::TextureView, wgpu::TextureView) {
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };
    (
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default()),
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default()),
    )
}
