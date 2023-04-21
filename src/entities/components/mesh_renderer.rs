use crate::entities::CameraUniform;

pub trait MeshRenderer {
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
    fn update_camera(&self, queue: &wgpu::Queue, camera_uniform_slice: &[CameraUniform]);
    fn update_instance_data(&self, queue: &wgpu::Queue);
}
