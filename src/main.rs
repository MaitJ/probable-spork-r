mod texture;
mod camera;
mod camera_controller;
mod test_tree;
mod mesh;
mod vertex;
mod shader;
mod errors;
mod editor;

use std::{rc::Rc, sync::Arc};

use editor::Editor;
use egui_wgpu::WgpuConfiguration;
use log::{info, warn};
use mesh::{TexturedMesh, Mesh};
use camera::{Camera, CameraUniform};
use camera_controller::CameraController;
use shader::{Shader, ShaderBuilder};
use test_tree::{VERTICES, INDICES};
use texture::Texture;
use wgpu::{InstanceDescriptor, RequestAdapterOptions};
use winit::{event_loop::{EventLoop, ControlFlow, EventLoopWindowTarget}, window::WindowBuilder, event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}, dpi::LogicalSize};
use winit::window::Window;

struct App {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pixels_per_point: f32,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    depth_texture: Texture,
    shaders: Vec<Rc<Shader>>,
    meshes: Vec<Box<dyn Mesh>>,
    editor: Editor
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

        let editor = Editor::new(&device, &window, surface_format).await;


        let camera = Self::init_camera(&config);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_controller = CameraController::new(0.2);

        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth texture");


        Self {
            window,
            device,
            queue,
            config,
            size,
            pixels_per_point,
            surface,
            camera,
            camera_controller,
            camera_uniform,
            depth_texture,
            editor,
            meshes: vec![],
            shaders: vec![],
        }
    }

    fn add_mesh(&mut self) {
        let texture_bytes = include_bytes!("textures/happy-tree.png");
        let diffuse_texture = Texture::from_bytes(texture_bytes, &self.device, &self.queue, "Tree texture").unwrap();

        info!("Tree texture format: {:?}", diffuse_texture.texture.format());
        let textured_mesh = TexturedMesh::from(&self.device, VERTICES, INDICES, Rc::clone(&self.shaders[0]), diffuse_texture);

        match textured_mesh {
            Ok(mesh) => self.meshes.push(Box::new(mesh)),
            Err(e) => warn!("Wasn't able to create mesh: {}", e)
        }
    }

    fn init_shaders(&mut self) {
        let basic_shader = ShaderBuilder::new()
            .load_shader(&self.device, "basic_shader.wgsl")
            .add_texture(&self.device, "texture")
            .add_uniform::<CameraUniform>(&self.device, "camera_uniform", CameraUniform::new())
            //TODO - Replace with logger
            .build(&self.device, &self.config).expect("[ERROR] Failed to build shader");

        self.shaders.push(Rc::new(basic_shader));
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

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true
                    }),
                    stencil_ops: None
                })
            });

            self.meshes.iter().for_each(|mesh| mesh.render(&mut render_pass));
        }

        self.editor.update_ui_textures(&self.device, &self.queue, &mut encoder);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Editor render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view: &view, 
                    resolve_target: None, 
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true
                    }),
                    stencil_ops: None
                })
            });
            self.editor.render_ui(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: Option<f32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "Depth texture");
        }

        self.editor.resize(new_size);
        if let Some(scale_factor) = scale_factor {
            self.editor.rescale(scale_factor);
            self.pixels_per_point = scale_factor;
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.meshes.iter().for_each(|mesh| mesh.update_camera(&self.queue, &[self.camera_uniform]));
    }
}

async fn start() {
    env_logger::init();
    info!("Engine start");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1280, 720))
        .build(&event_loop).unwrap();
    let mut app = App::new(window).await;
    app.init_shaders();
    app.add_mesh();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, window_id,} if window_id == app.window.id() => if !app.input(event) {
            //app.editor.update(event);
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
        },
        Event::MainEventsCleared => app.window.request_redraw(),
        Event::RedrawRequested(window_id) if window_id == app.window.id() => {
            app.update();
            match app.render() {
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
