#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
    let winit::dpi::PhysicalSize { width, height } = window.inner_size();
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width,
        height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);
    let mut platform =
        egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor: window.scale_factor(),
            ..Default::default()
        });
    let ui = egui_wgpu_backend::RenderPass::new(&device, format, 1);
    let app = stove::Stove::new(platform.context(), &device, format);
    events.run(move |event, _, flow| {
        use winit::{
            event::{Event, WindowEvent},
            event_loop::ControlFlow,
        };
        platform.handle_event(&event);
        match event {
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
            } => *flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
