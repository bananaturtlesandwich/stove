#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[pollster::main]
async fn main() {
    let start = std::time::Instant::now();
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
    let size = window.inner_size();
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
    let mut platform =
        egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            ..Default::default()
        });
    let mut ui = egui_wgpu_backend::RenderPass::new(&device, format, 1);
    let mut app = stove::Stove::new(&mut platform.context(), &device, format);
    events.run(move |event, _, flow| {
        use winit::{
            event::{Event, WindowEvent},
            event_loop::ControlFlow,
        };
        platform.handle_event(&event);
        match event {
            Event::RedrawRequested(_) => {
                let size = window.inner_size();
                let show_ui = app.show_ui();
                if show_ui {
                    platform.update_time(start.elapsed().as_secs_f64());
                    platform.begin_frame();
                    app.ui(&platform.context(), &device, format, &size);
                }
                let frame = surface.get_current_texture().unwrap();
                let mut encoder = device.create_command_encoder(&Default::default());
                let view = frame.texture.create_view(&Default::default());
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
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
                    ..Default::default()
                });
                app.scene(&queue, &mut pass, &size);
                let mut delta = Default::default();
                if show_ui {
                    let output = platform.end_frame(Some(&window));
                    let jobs = platform.context().tessellate(output.shapes);
                    let screen = egui_wgpu_backend::ScreenDescriptor {
                        physical_width: size.width,
                        physical_height: size.height,
                        scale_factor: window.scale_factor() as f32,
                    };
                    delta = output.textures_delta;
                    ui.add_textures(&device, &queue, &delta).unwrap();
                    ui.update_buffers(&device, &queue, &jobs, &screen);
                    ui.execute_with_renderpass(&mut pass, &jobs, &screen)
                        .unwrap();
                }
                drop(pass);
                queue.submit(Some(encoder.finish()));
                frame.present();
                ui.remove_textures(delta).unwrap();
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if size.width == 0 || size.height == 0 {
                    return;
                }
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                #[cfg(target_os = "macos")]
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                platform
                    .context()
                    .memory_mut(|storage| app.store(&mut storage.data));
                app.on_close_event();
                *flow = ControlFlow::Exit
            }
            _ => (),
        }
    });
}
