mod texture;
mod camera;
mod camera_controller;
mod test_tree;
mod mesh;
mod vertex;
mod shader;
mod errors;
mod renderer;

use std::{rc::Rc, sync::Arc};

//use renderer::Editor;
use egui_wgpu::WgpuConfiguration;
use log::{info, warn};
use mesh::{TexturedMesh, Mesh};
use camera::{Camera, CameraUniform};
use camera_controller::CameraController;
use renderer::Renderer;
use shader::{Shader, ShaderBuilder};
use test_tree::{VERTICES, INDICES};
use texture::Texture;
use wgpu::{InstanceDescriptor, RequestAdapterOptions, RenderPass};
use winit::{event_loop::{EventLoop, ControlFlow, EventLoopWindowTarget}, window::WindowBuilder, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}, dpi::LogicalSize};
use winit::window::Window;

use crate::renderer::{MainRenderer, Editor};

pub struct WgpuStructs {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration
}

pub struct RendererResources {
   camera_controller: CameraController,
   camera_uniform: CameraUniform,
   camera: Camera
}

struct App {
    window: Window,
    wgpu_structs: WgpuStructs,
    size: winit::dpi::PhysicalSize<u32>,
    pixels_per_point: f32,
    renderer_resources: RendererResources,
    shaders: Vec<Arc<Shader>>,
    meshes: Arc<Vec<Box<dyn Mesh + Send + Sync>>>,
    renderer: Box<dyn Renderer>
}

impl App {
    async fn new(window: Window, event_loop: &EventLoop<()>) -> App {
        let size = window.inner_size();
        let pixels_per_point = window.scale_factor() as f32;

        let backends = wgpu::Backends::all();
        let device_descriptor = wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            label: None
        };
        let power_preference = wgpu::PowerPreference::HighPerformance;

        let instance = wgpu::Instance::new(
            InstanceDescriptor {
                backends,
                dx12_shader_compiler: Default::default()
            }
        );

        let surface = unsafe {instance.create_surface(&window)}.unwrap();

        let adapter = instance.request_adapter(&RequestAdapterOptions{
            power_preference,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&device_descriptor, None).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let present_mode = surface_caps.present_modes[0];
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        info!("Surface format: {:?}", surface_format);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![]
        };
        surface.configure(&device, &config);

        //let editor = Editor::new(event_loop, &device, &window, surface_format).await;

        let camera = Self::init_camera(&config);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_controller = CameraController::new(0.2);

        let wgpu_structs = WgpuStructs {
            device,
            config,
            surface,
            queue
        };

        let renderer_resources = RendererResources {
            camera_controller,
            camera,
            camera_uniform
        };

        //let renderer = MainRenderer::new(depth_texture);
        let renderer = Editor::new(event_loop, &wgpu_structs, &window, surface_format).await;

        Self {
            window,
            wgpu_structs,
            size,
            pixels_per_point,
            renderer_resources,
            meshes: Arc::new(vec![]),
            shaders: vec![],
            renderer: Box::new(renderer)
        }
    }

    fn get_shader_by_label(&self, label: &str) -> Option<Arc<Shader>> {
        self.shaders.iter()
            .find(|shader| shader.label == label)
            .cloned()
    }

    fn add_mesh(&mut self) {
        let WgpuStructs { device, queue, .. } = &self.wgpu_structs;

        let texture_bytes = include_bytes!("textures/happy-tree.png");
        let diffuse_texture = Texture::from_bytes(texture_bytes, device, queue, "Tree texture").unwrap();

        info!("Tree texture format: {:?}", diffuse_texture.texture.format());
        let default_shader = self.get_shader_by_label("basic_shader.wgsl");

        match default_shader {
            Some(shader) => {
                let textured_mesh = TexturedMesh::from(device, VERTICES, INDICES, shader, diffuse_texture);

                match textured_mesh {
                    Ok(mesh) => self.renderer.add_mesh(Box::new(mesh)),
                    Err(e) => warn!("Wasn't able to create mesh: {}", e)
                }
            },
            None => warn!("Couldn't find shader ({}) for {}", "basic_shader.wgsl", "tree")
        }

    }

    fn init_shaders(&mut self) -> Result<(), anyhow::Error> {
        let WgpuStructs { device, config, .. } = &self.wgpu_structs;

        let basic_shader = ShaderBuilder::new()
            .load_shader(device, "basic_shader.wgsl")?
            .add_texture(device, "texture")
            .add_uniform::<CameraUniform>(device, "camera_uniform", CameraUniform::new())
            //TODO - Replace with logger
            .build(device, config).expect("Failed to build shader");

        self.shaders.push(Arc::new(basic_shader));
        Ok(())
    }

    fn init_camera(config: &wgpu::SurfaceConfiguration) -> Camera {
        Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.renderer_resources.camera_controller.process_events(event)
    }

    fn resize_window(&mut self, new_size: winit::dpi::PhysicalSize<u32>) -> Option<Texture> {
        let WgpuStructs { config, device, surface, .. } = &mut self.wgpu_structs;

        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            config.width = new_size.width;
            config.height = new_size.height;
            surface.configure(device, config);
            let depth_texture = Texture::create_depth_texture(device, config, "Depth texture");
            return Some(depth_texture)
        }
        None
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: Option<f32>) {
        let depth_texture = self.resize_window(new_size);
        if let Some(scale_factor) = scale_factor {
            info!("Resized with scale_factor: {}", scale_factor);
        }
        self.renderer.resize(new_size, scale_factor, depth_texture);
    }

}

async fn start() {
    env_logger::init();
    info!("Engine start");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1280, 720))
        .build(&event_loop).unwrap();
    let mut app = App::new(window, &event_loop).await;
    if let Err(err) = app.init_shaders() {
        warn!("Failed to initialize shaders: {}", err);
    }

    app.add_mesh();
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, window_id,} if window_id == app.window.id() => if !app.input(event) {
            
            let event_response = app.renderer.handle_event(event);
            
            if !event_response.consumed {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput{
                                state: ElementState::Pressed,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                            ..
                    } => match keycode {
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => app.resize(*physical_size, None),
                    WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor } => app.resize(**new_inner_size, Some(*scale_factor as f32)),
                    _ => {}
                }
            }
        },
        Event::MainEventsCleared => app.window.request_redraw(),
        Event::RedrawRequested(window_id) if window_id == app.window.id() => {
            app.renderer.update(&app.wgpu_structs, &mut app.renderer_resources);
            match app.renderer.render(&app.wgpu_structs, &app.window) {
                Ok(_) => {},
                Err(wgpu::SurfaceError::Lost) => app.resize(app.size, Some(app.pixels_per_point)),
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
