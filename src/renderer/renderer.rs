use egui_winit::EventResponse;
use log::info;
use wgpu::RenderPass;
use winit::{event::WindowEvent, window::Window};

use crate::{WgpuStructs, RendererResources, texture::Texture};
use crate::entities::components::MeshRenderer;

pub trait Renderer {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: Option<f32>, depth_texture: Option<Texture>);
    fn render<'a>(&'a mut self, wgpu_structs: &WgpuStructs, window: &Window, renderer_resources: &'a mut RendererResources) -> Result<(), wgpu::SurfaceError>;
    fn add_mesh(&mut self, mesh: impl MeshRenderer + 'static);
    //TODO - Maybe add a instantiate mesh that takes a mesh id and returns the instance id
}

pub struct RendererLoop;

impl RendererLoop {
    pub fn update(queue: &wgpu::Queue, renderer_resources: &RendererResources, meshes: &Vec<Box<dyn MeshRenderer>>) {
        let RendererResources { camera_uniform } = renderer_resources;
        meshes.iter().for_each(|mesh| {
            mesh.update_instance_data(queue);
            mesh.update_camera(queue, &[*camera_uniform]);
        });
    }

    pub fn render<'a>(render_pass: &mut RenderPass<'a>, _renderer_resources: &'a RendererResources, meshes: &'a Vec<Box<dyn MeshRenderer>>) {
        meshes.iter().for_each(|mesh| mesh.render(render_pass));
    }
}
