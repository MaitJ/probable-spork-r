use egui_winit::EventResponse;
use wgpu::RenderPass;
use winit::{event::WindowEvent, window::Window};

use crate::{WgpuStructs, RendererResources, texture::Texture, renderer::Mesh};

pub trait Renderer {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: Option<f32>, depth_texture: Option<Texture>);
    fn handle_event(&mut self, event: &WindowEvent) -> EventResponse;
    fn render<'a>(&'a mut self, wgpu_structs: &WgpuStructs, window: &Window, renderer_resources: &RendererResources) -> Result<(), wgpu::SurfaceError>;
}

pub struct RendererLoop;

impl RendererLoop {
    pub fn update(queue: &wgpu::Queue, renderer_resources: &RendererResources) {
        let RendererResources { camera_uniform, renderables } = renderer_resources;
        renderables.iter().for_each(|mesh| mesh.update_camera(queue, &[*camera_uniform]));
    }

    pub fn render<'a>(render_pass: &mut RenderPass<'a>, renderer_resources: &'a RendererResources) {
        renderer_resources.renderables.iter().for_each(|mesh| mesh.render(render_pass));
    }
}