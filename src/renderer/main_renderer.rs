use crate::{renderer::Renderer, WgpuStructs, RendererResources, mesh::Mesh, texture::Texture};

pub struct MainRenderer {
    depth_texture: Texture,
    meshes: Vec<Box<dyn Mesh>>,
}

impl MainRenderer {
    pub fn new(depth_texture: Texture) -> Self {
        Self { depth_texture, meshes: vec![] }
    }
}

impl Renderer for MainRenderer {
    fn add_mesh(&mut self, mesh: Box<dyn Mesh + Send + Sync>) {
        self.meshes.push(mesh);
    }

    fn handle_event(&mut self, event: &winit::event::WindowEvent) -> egui_winit::EventResponse {
        egui_winit::EventResponse { consumed: false, repaint: false }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: Option<f32>, depth_texture: Option<Texture>) {
        match depth_texture {
            Some(depth_texture) => self.depth_texture = depth_texture,
            None => ()
        }
    }

    fn update(&mut self, wgpu_structs: &WgpuStructs, renderer_resources: &mut RendererResources) {
        let queue = &wgpu_structs.queue;
        let RendererResources { camera_controller, camera_uniform, camera } = renderer_resources;

        camera_controller.update_camera(camera);
        camera_uniform.update_view_proj(camera);

        self.meshes.iter().for_each(|mesh| mesh.update_camera(queue, &[*camera_uniform]));
    }

    fn render<'a>(&'a mut self, wgpu_structs: &WgpuStructs, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        let WgpuStructs { surface, device, queue, .. } = wgpu_structs;
        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}