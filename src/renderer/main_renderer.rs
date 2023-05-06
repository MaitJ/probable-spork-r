use std::cell::Ref;

use log::warn;

use crate::{renderer::Renderer, WgpuStructs, RendererResources, texture::Texture};
use crate::entities::components::{MeshRenderer, MeshInstance};

use super::{TransformInstance, MeshManager};
use super::renderer::RendererLoop;

pub struct MainRenderer {
    depth_texture: Texture,
    mesh_manager: MeshManager
}

impl MainRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            depth_texture: Texture::create_depth_texture(device, config, "Depth texture"),
            mesh_manager: MeshManager::new()
        }
    }
}

impl Renderer for MainRenderer {
    fn get_mesh_manager(&self) -> &MeshManager {
        &self.mesh_manager
    }

    fn get_mesh_manager_mut(&mut self) -> &mut MeshManager {
        &mut self.mesh_manager
    }

    fn update_meshes(&mut self, mesh_instances: Vec<Ref<MeshInstance>>) {
        for mesh_instance in mesh_instances.iter() {
            match self.mesh_manager.get_meshes_mut().get_mut(mesh_instance.mesh_index) {
                Some(mesh) => {
                    if let Err(e) = mesh.update_instance_data(mesh_instance.mesh_instance_index, TransformInstance::from(&mesh_instance.local_transform)) {
                        warn!("Error updating instance: {}", e);
                    }
                },
                None => {
                    warn!("Couldnt find mesh instance (mesh_index: {}, mesh_instance_index: {})", mesh_instance.mesh_index, mesh_instance.mesh_instance_index);
                }
            }
        }
    }

    fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>, _scale_factor: Option<f32>, depth_texture: Option<Texture>) {
        match depth_texture {
            Some(depth_texture) => self.depth_texture = depth_texture,
            None => ()
        }
    }

    fn render<'a>(&'a mut self, wgpu_structs: &WgpuStructs, _window: &winit::window::Window, renderer_resources: &mut RendererResources) -> Result<(), wgpu::SurfaceError> {
        RendererLoop::update(&wgpu_structs.queue, renderer_resources, &self.mesh_manager.get_meshes());

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

            RendererLoop::render(&mut render_pass, renderer_resources, &self.mesh_manager.get_meshes());
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn add_mesh(&mut self, mesh: impl MeshRenderer + 'static) {
        self.mesh_manager.add_mesh(mesh);
    }
}
