mod texture;
mod entities;
mod mesh;
mod vertex;
mod shader;
mod errors;
mod renderer;
mod test_tree;
mod world;
mod assets;

use std::rc::Rc;
use std::sync::Arc;
use log::{info, warn};
use mesh::TexturedMesh;
use entities::{Camera, CameraUniform, Entity};
use entities::CameraController;
use renderer::Renderer;
use shader::{Shader, ShaderBuilder};
use texture::Texture;
use wgpu::{InstanceDescriptor, RequestAdapterOptions};
use winit::{event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}, dpi::LogicalSize};
use winit::window::Window;
use test_tree::{VERTICES, INDICES};
use world::Scene;

use crate::assets::TestScript;
use crate::renderer::Editor;

pub struct WgpuStructs {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration
}

pub struct RendererResources {
   camera_uniform: CameraUniform,
   renderables: Vec<Rc<TexturedMesh>>
}

struct App {
    window: Window,
    wgpu_structs: WgpuStructs,
    size: winit::dpi::PhysicalSize<u32>,
    pixels_per_point: f32,
    renderer_resources: RendererResources,
    shaders: Vec<Arc<Shader>>,
    scene: Scene,
    camera_controller: CameraController,
    camera: Camera
}

impl App {
    async fn new(window: Window) -> App {
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

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");
        //let renderer = MainRenderer::new(depth_texture);

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
            camera_uniform,
            renderables: vec![]
        };


        Self {
            window,
            wgpu_structs,
            size,
            pixels_per_point,
            renderer_resources,
            shaders: vec![],
            scene: Scene::new(),
            camera_controller,
            camera
        }
    }

    fn get_shader_by_label(&self, label: &str) -> Option<Arc<Shader>> {
        self.shaders.iter()
            .find(|shader| shader.label == label)
            .cloned()
    }

    fn create_mesh(&mut self) -> Option<TexturedMesh> {
        let WgpuStructs { device, queue, .. } = &self.wgpu_structs;

        let texture_bytes = include_bytes!("textures/happy-tree.png");
        let diffuse_texture = Texture::from_bytes(texture_bytes, device, queue, "Tree texture").unwrap();

        info!("Tree texture format: {:?}", diffuse_texture.texture.format());
        let default_shader = self.get_shader_by_label("basic_shader.wgsl");

        match default_shader {
            Some(shader) => {
                let textured_mesh = TexturedMesh::from(device, VERTICES, INDICES, shader, diffuse_texture);

                match textured_mesh {
                    Ok(mesh) => Some(mesh),
                    Err(e) => {
                        warn!("Wasn't able to create mesh: {}", e);
                        None
                    }
                }
            },
            None => {
                warn!("Couldn't find shader ({}) for {}", "basic_shader.wgsl", "tree");
                None
            }
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
        self.camera_controller.process_events(event)
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

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: Option<f32>) -> Option<Texture> {
        let depth_texture = self.resize_window(new_size);
        if let Some(scale_factor) = scale_factor {
            info!("Resized with scale_factor: {}", scale_factor);
        }
        return depth_texture;
    }

    fn update(&mut self) {
        let RendererResources { camera_uniform, .. } = &mut self.renderer_resources;

        self.camera_controller.update_camera(&mut self.camera);
        camera_uniform.update_view_proj(&self.camera);
    }

}

async fn start() {
    env_logger::init();
    info!("Engine start");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1280, 720))
        .build(&event_loop).unwrap();

    {
        let mut app = App::new(window).await;

        if let Err(err) = app.init_shaders() {
            warn!("Failed to initialize shaders: {}", err);
        }

        let mesh = app.create_mesh();

        {
            let entity = app.scene.add_empty_entity()
                .add_script(Box::new(TestScript::default()));

            match mesh {
                Some(mesh) => entity.add_component(Rc::new(mesh)),
                None => info!("Couldn't add textured_mesh to entity")
            }
            let textured_mesh = entity.get_component::<TexturedMesh>();
            match textured_mesh {
                Some(mesh) => info!("Found textured_mesh on entity, index_count: {}", mesh.index_count),
                None => info!("Couldn't get textured_mesh on entity")
            }
        }

        let mut renderer = Editor::new(&event_loop, &app.wgpu_structs, &app.window, app.wgpu_structs.config.format).await;

        app.scene.call_user_script_setups();
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { ref event, window_id,} if window_id == app.window.id() => if !app.input(event) {
                
                let event_response = renderer.handle_event(event);
                
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
                        WindowEvent::Resized(physical_size) => {
                            let depth_texture = app.resize(*physical_size, None);
                            renderer.resize(*physical_size, None, depth_texture);
                        },
                        WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor } => {
                            let depth_texture = app.resize(**new_inner_size, Some(*scale_factor as f32));
                            renderer.resize(**new_inner_size, Some(*scale_factor as f32), depth_texture);
                        },
                        _ => {}
                    }
                }
            },
            Event::MainEventsCleared => app.window.request_redraw(),
            Event::RedrawRequested(window_id) if window_id == app.window.id() => {
                app.scene.call_user_script_updates();
                app.update();

                app.renderer_resources.renderables = app.scene.get_renderables();
                match renderer.render(&app.wgpu_structs, &app.window, &app.renderer_resources) {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => {
                        let depth_texture = app.resize(app.size, Some(app.pixels_per_point));
                        renderer.resize(app.size, Some(app.pixels_per_point), depth_texture);
                    },
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => println!("Err: {:?}", e)
                }
            }
            _ => {}
        });
    }
}

fn main() {
    pollster::block_on(start());
}
