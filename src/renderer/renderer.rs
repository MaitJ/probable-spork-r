use std::cell::{Ref, RefCell};

use wgpu::RenderPass;
use winit::window::Window;

use crate::entities::components::{MeshInstance, MeshRenderer};
use crate::{texture::Texture, RendererResources, WgpuStructs};

use super::MeshManager;

pub trait Renderer {
    fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
        scale_factor: Option<f32>,
        depth_texture: Option<Texture>,
    );
    fn render<'a>(
        &'a mut self,
        wgpu_structs: &WgpuStructs,
        window: &Window,
        renderer_resources: &'a mut RendererResources,
    ) -> Result<(), wgpu::SurfaceError>;
    fn add_mesh(&mut self, mesh: impl MeshRenderer + 'static);
    fn update_meshes(&mut self, mesh_instances: Vec<MeshInstance>);
    fn get_mesh_manager(&self) -> &MeshManager;
    fn get_mesh_manager_mut(&mut self) -> &mut MeshManager;
}

pub struct RendererLoop;

impl RendererLoop {
    pub fn update(
        queue: &wgpu::Queue,
        renderer_resources: &RendererResources,
        meshes: &Vec<Box<dyn MeshRenderer>>,
    ) {
        let RendererResources { camera_uniform, .. } = renderer_resources;
        meshes.iter().for_each(|mesh| {
            mesh.write_instance_data(queue);
            mesh.update_camera(queue, &[*camera_uniform]);
        });
    }

    pub fn render<'a>(
        render_pass: &mut RenderPass<'a>,
        _renderer_resources: &'a RendererResources,
        meshes: &'a Vec<Box<dyn MeshRenderer>>,
    ) {
        meshes.iter().for_each(|mesh| mesh.render(render_pass));
    }
}
