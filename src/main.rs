mod texture;
mod entities;
mod vertex;
mod shader;
mod errors;
mod renderer;
mod test_tree;
mod assets;
mod engine;
mod editor;
mod script;
mod scene;

use std::sync::Arc;
use log::{info, warn};
use probable_spork_ecs::component::Component;
use renderer::TexturedMesh;
use entities::{CameraUniform, components::MeshRenderer};
use renderer::{Renderer};
use shader::{Shader, ShaderBuilder};
use texture::Texture;
use wgpu::{InstanceDescriptor, RequestAdapterOptions};
use winit::{event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}, dpi::LogicalSize};
use winit::window::Window;
use test_tree::{VERTICES, INDICES};

use crate::{editor::Editor};
use crate::engine::Engine;
use crate::renderer::EditorRenderer;

pub struct WgpuStructs {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration
}

pub struct RendererResources {
    camera_uniform: CameraUniform,
}

struct App {
    window: Window,
    wgpu_structs: WgpuStructs,
    size: winit::dpi::PhysicalSize<u32>,
    pixels_per_point: f32,
    shaders: Vec<Arc<Shader>>
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



        let wgpu_structs = WgpuStructs {
            device,
            config,
            surface,
            queue
        };

        Self {
            window,
            wgpu_structs,
            size,
            pixels_per_point,
            shaders: vec![],
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
                let textured_mesh = TexturedMesh::from(String::from("happy-tree"), device, VERTICES, INDICES, shader, diffuse_texture);

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
}

async fn start() {
    env_logger::init();
    info!("Engine start");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1280, 720))
        .with_title("Probable-spork")
        .build(&event_loop).unwrap();

    {
        let mut app = App::new(window).await;
        let mut engine = Engine::new(&app.wgpu_structs.config);
        let mut editor = Editor::new(&event_loop, &app.window);


        if let Err(err) = app.init_shaders() {
            warn!("Failed to initialize shaders: {}", err);
        }

        let mesh = app.create_mesh();

        //{
        //    let entity = engine.scene.add_empty_entity()
        //        .add_script(Box::new(TestScript::default()));


        //    match mesh {
        //        Some(mesh) => entity.add_component(Component::TexturedMesh(mesh)),
        //        None => info!("Couldn't add textured_mesh component to entity")
        //    }
        //    let textured_mesh = entity.get_renderable();
        //    match textured_mesh {
        //        Some(mesh) => info!("Found textured_mesh on entity, index_count: {}", mesh.index_count),
        //        None => info!("Couldn't get textured_mesh on entity")
        //    }
        //}

        let pixels_per_point = editor.pixels_per_point;
        let mut renderer = EditorRenderer::new(&app.wgpu_structs, &app.window, app.wgpu_structs.config.format, pixels_per_point).await;
        //let mut renderer = MainRenderer::new(&app.wgpu_structs.device, &app.wgpu_structs.config);
        if let Some(mesh) = mesh {
            renderer.add_mesh(mesh);
        }

        engine.setup(&mut renderer);
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { ref event, window_id,} if window_id == app.window.id() => if !engine.input(event) {


                let event_response = editor.handle_event(event);
                
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
                            let depth_texture = app.resize_window(*physical_size);
                            renderer.resize(*physical_size, None, depth_texture);
                        },
                        WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor } => {
                            let depth_texture = app.resize_window(**new_inner_size);
                            renderer.resize(**new_inner_size, Some(*scale_factor as f32), depth_texture);
                        },
                        _ => {}
                    }
                }
            },
            Event::MainEventsCleared => app.window.request_redraw(),
            Event::RedrawRequested(window_id) if window_id == app.window.id() => {

                let mut renderer_resources = RendererResources {
                    camera_uniform: CameraUniform::new(),
                };

                engine.update(&mut renderer_resources);
                renderer.update_meshes(engine.scene.get_mesh_instances());

                let editor_output = editor.draw(&app.window, &renderer_resources);
                renderer.update_ui(editor_output.0, editor_output.1);

                match renderer.render(&app.wgpu_structs, &app.window, &mut renderer_resources) {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => {
                        let depth_texture = app.resize_window(app.size);
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
