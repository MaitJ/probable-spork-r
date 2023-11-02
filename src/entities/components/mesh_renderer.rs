use std::{error::Error, fmt::Display};

use crate::{entities::CameraUniform, renderer::TransformInstance};

pub trait MeshRenderer {
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
    fn update_camera(&self, queue: &wgpu::Queue, camera_uniform_slice: &[CameraUniform]);
    fn update_instance_data(
        &mut self,
        instance_index: usize,
        transform: TransformInstance,
    ) -> Result<(), MeshRendererError>;
    fn write_instance_data(&self, queue: &wgpu::Queue);
    fn create_instance(&mut self) -> usize;
}

#[derive(Debug)]
pub enum MeshRendererError {
    InstanceNotFound,
}

impl Error for MeshRendererError {}
impl Display for MeshRendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshRendererError::InstanceNotFound => write!(f, "Meshes instance wasn't found"),
        }
    }
}
