use egui_winit::EventResponse;
use wgpu::RenderPass;
use winit::{event::WindowEvent, window::Window};

use crate::{mesh::Mesh, WgpuStructs, RendererResources, texture::Texture};

//Renderer should generate the depth_texture for render_pass

pub trait Renderer {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: Option<f32>, depth_texture: Option<Texture>);
    fn handle_event(&mut self, event: &WindowEvent) -> EventResponse;
    fn render<'a>(&'a mut self, wgpu_structs: &WgpuStructs, window: &Window, renderer_resources: &RendererResources) -> Result<(), wgpu::SurfaceError>;
}