use crate::{renderer::Renderer, WgpuStructs, RendererResources, mesh::Mesh, texture::Texture};

pub struct MainRenderer {
    depth_texture: Texture,
}

impl MainRenderer {
    pub fn new(depth_texture: Texture) -> Self {
        Self { depth_texture }
    }

    fn update(&mut self, wgpu_structs: &WgpuStructs, renderer_resources: &RendererResources) {
        let queue = &wgpu_structs.queue;
        let RendererResources { camera_uniform, .. } = renderer_resources;
        renderer_resources.renderables.iter().for_each(|mesh| mesh.update_camera(queue, &[*camera_uniform]));
    }
}

impl Renderer for MainRenderer {
    fn handle_event(&mut self, _event: &winit::event::WindowEvent) -> egui_winit::EventResponse {
        egui_winit::EventResponse { consumed: false, repaint: false }
    }

    fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>, _scale_factor: Option<f32>, depth_texture: Option<Texture>) {
        match depth_texture {
            Some(depth_texture) => self.depth_texture = depth_texture,
            None => ()
        }
    }

    fn render<'a>(&'a mut self, wgpu_structs: &WgpuStructs, _window: &winit::window::Window, renderer_resources: &RendererResources) -> Result<(), wgpu::SurfaceError> {
        self.update(wgpu_structs, renderer_resources);

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

            renderer_resources.renderables.iter().for_each(|mesh| mesh.render(&mut render_pass));
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}