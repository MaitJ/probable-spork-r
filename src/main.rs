use wgpu::{InstanceDescriptor, RequestAdapterOptions};
use winit::{event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}};
use winit::window::Window;

struct App {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>
}

impl App {

    //TODO - Create triangle and then remove unwraps
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(
            InstanceDescriptor {
                backends: wgpu::Backends::all(),
                dx12_shader_compiler: Default::default()
            }
        );

        let surface = unsafe {instance.create_surface(&window)}.unwrap();

        let adapter = instance.request_adapter(&RequestAdapterOptions{
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            label: None
        }, None).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![]
        };
        surface.configure(&device, &config);

        Self {
            window,
            device,
            queue,
            config,
            size,
            surface
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        println!("Render");
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view: &view, 
                    resolve_target: None, 
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0
                        }),
                        store: true
                    }
                })],
                depth_stencil_attachment: None
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        println!("Resize called");
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

async fn start() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut app = App::new(window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, window_id,} if window_id == app.window.id() => match event {
            WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput{
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => app.resize(*physical_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } =>
                app.resize(**new_inner_size),
            _ => {}
        },
        Event::MainEventsCleared => app.window.request_redraw(),
        Event::RedrawRequested(window_id) if window_id == app.window.id() => {
            match app.render() {
                Ok(_) => {},
                Err(wgpu::SurfaceError::Lost) => app.resize(app.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => println!("Err: {:?}", e)
            }
        }
        _ => {}
    });
}

fn main() {
    pollster::block_on(start());
}
