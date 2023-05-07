use std::cell::{RefCell, Ref};

use egui::epaint::Primitive;
use egui::{ClippedPrimitive, PaintCallbackInfo, TexturesDelta};
use egui_wgpu::renderer::ScreenDescriptor;
use log::{info, warn};
use wgpu::{Color, RenderPass};
use winit::window::Window;
use crate::editor::GamePreviewCallback;

use crate::entities::components::{MeshRenderer, MeshInstance};
use crate::{WgpuStructs, renderer::Renderer, texture::Texture, RendererResources};

use super::{TransformInstance, MeshManager};

pub struct EditorRenderer {
    depth_texture: Texture,
    renderer: egui_wgpu::Renderer,
    screen_descriptor: ScreenDescriptor,
    clipped_primitives: Vec<ClippedPrimitive>,
    textures_delta: TexturesDelta,
    pub is_enabled: bool,
    mesh_manager: MeshManager
}


impl EditorRenderer {
    pub async fn new(wgpu_structs: &WgpuStructs, window: &Window, 
        texture_format: wgpu::TextureFormat, pixels_per_point: f32) -> Self {
        info!("Creating editor");

        let WgpuStructs { device, config, .. } = wgpu_structs;

        let renderer = egui_wgpu::Renderer::new(device, texture_format, Some(crate::Texture::DEPTH_FORMAT), 1);

        let screen_descriptor = ScreenDescriptor {
            pixels_per_point,
            size_in_pixels: window.inner_size().into()
        };

        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth texture");

        Self {
            depth_texture,
            renderer,
            screen_descriptor,
            clipped_primitives: vec![],
            is_enabled: true,
            textures_delta: TexturesDelta::default(),
            mesh_manager: MeshManager::new()
        }
    }
    
    pub fn update_ui(&mut self, textures_delta: TexturesDelta, clipped_primitives: Vec<ClippedPrimitive>) {
        self.clipped_primitives = clipped_primitives;
        self.textures_delta = textures_delta;
    }

    pub fn update_ui_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, 
            _window: &winit::window::Window) {


        for (id, image_delta) in self.textures_delta.set.iter() {
            self.renderer.update_texture(device, queue, *id, &image_delta);
        }
        self.renderer.update_buffers(device, queue, encoder, &self.clipped_primitives, &self.screen_descriptor);
    }
    
    fn call_game_preview_update(&self, device: &wgpu::Device, queue: &wgpu::Queue, 
        encoder: &mut wgpu::CommandEncoder, clipped_primitive: &Vec<ClippedPrimitive>, renderer_resources: &RendererResources) {
        for egui::epaint::ClippedPrimitive {
            clip_rect: _,
            primitive
        } in clipped_primitive {
            match primitive {
                Primitive::Callback(callback) => {
                    let cbfn = if let Some(c) = callback.callback.downcast_ref::<GamePreviewCallback>() {
                        c
                    } else {
                        // We already warned in the `prepare` callback
                        continue;
                    };

                    (cbfn.update)(
                        device,
                        queue,
                        encoder,
                        renderer_resources,
                        &self.mesh_manager.get_meshes()
                    );
                },
                _ => ()
            }
        }
    }


    fn call_game_preview_render<'a>(&'a self, render_pass: &mut RenderPass<'a>, clipped_primitive: &Vec<ClippedPrimitive>, 
        renderer_resources: &'a RendererResources) {
        for egui::epaint::ClippedPrimitive {
            clip_rect,
            primitive
        } in clipped_primitive {
            match primitive {
                Primitive::Callback(callback) => {
                    let cbfn = if let Some(c) = callback.callback.downcast_ref::<GamePreviewCallback>() {
                        c
                    } else {
                        continue;
                    };

                    let pixels_per_point = self.screen_descriptor.pixels_per_point;

                    {

                        let min = (callback.rect.min.to_vec2() * pixels_per_point).round();
                        let max = (callback.rect.max.to_vec2() * pixels_per_point).round();

                        render_pass.set_viewport(
                            min.x,
                            min.y,
                            max.x - min.x,
                            max.y - min.y,
                            0.0,
                            1.0,
                        );
                    }

                    (cbfn.render)(
                        PaintCallbackInfo {
                            viewport: callback.rect,
                            clip_rect: *clip_rect,

                            pixels_per_point,
                            screen_size_px: self.screen_descriptor.size_in_pixels,
                        },
                        render_pass,
                        renderer_resources,
                        &self.mesh_manager.get_meshes()
                    );
                },
                _ => ()
            }
        }
    }
}

impl Renderer for EditorRenderer {
    fn get_mesh_manager(&self) -> &MeshManager {
        &self.mesh_manager
    }

    fn get_mesh_manager_mut(&mut self) -> &mut MeshManager {
        &mut self.mesh_manager
    }

    fn update_meshes(&mut self, mesh_instances: Vec<MeshInstance>) {
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

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>, scale_factor: Option<f32>, depth_texture: Option<Texture>) {
        match depth_texture {
            Some(depth_texture) => self.depth_texture = depth_texture,
            None => ()
        }

        self.screen_descriptor.size_in_pixels = size.into();

        if let Some(pixels_per_point) = scale_factor {
            self.screen_descriptor.pixels_per_point = pixels_per_point;
        }
    }

    fn render<'a>(&'a mut self, wgpu_structs: &WgpuStructs, window: &Window, renderer_resources: &'a mut RendererResources) -> Result<(), wgpu::SurfaceError> {

        let WgpuStructs { surface, device, queue, .. } = wgpu_structs;
        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        //renderer_resources.renderables.iter().for_each(|renderable| renderable.update_instance_data(queue));

        self.update_ui_textures(device, queue, &mut encoder, window);
        self.call_game_preview_update(device, queue, &mut encoder, &self.clipped_primitives, renderer_resources);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Editor render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view: &view, 
                    resolve_target: None, 
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color {
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
            self.renderer.render(&mut render_pass, &self.clipped_primitives, &self.screen_descriptor);
            self.call_game_preview_render(&mut render_pass, &self.clipped_primitives, renderer_resources);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn add_mesh(&mut self, mesh: impl MeshRenderer + 'static) {
        self.mesh_manager.add_mesh(mesh);
    }

}
