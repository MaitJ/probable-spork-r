use crate::{renderer::Renderer, WgpuStructs, RendererResources, renderer::Mesh, texture::Texture};

use super::renderer::RendererLoop;

pub struct MainRenderer {
    depth_texture: Texture,
}

impl MainRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        Self { depth_texture: Texture::create_depth_texture(device, config, "Depth texture") }
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

        RendererLoop::update(&wgpu_structs.queue, renderer_resources);

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

            RendererLoop::render(&mut render_pass, renderer_resources);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}