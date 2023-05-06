use std::sync::Arc;

use cgmath::{Vector3, Quaternion, Rotation3};

use wgpu::util::DeviceExt;
use crate::entities::CameraUniform;
use crate::{vertex::Vertex, shader::{Shader, BIND_GROUP_POSTFIX}, texture::Texture};
use crate::entities::components::{MeshRenderer, MeshRendererError};

use super::TransformInstance;
use super::transform_instance::TransformInstanceRaw;

// TODO - Major store one instance of each mesh and use instances for entites
pub struct TexturedMesh {
    pub label: String,
    pub shader: Arc<Shader>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub texture: Texture,
    pub texture_bind_group: wgpu::BindGroup,
    pub camera_bind_group: wgpu::BindGroup,
    pub instance_buffer: wgpu::Buffer,
    pub instances: Vec<TransformInstance>
}

impl MeshRenderer for TexturedMesh {
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.shader.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

        render_pass.draw_indexed(0..self.index_count, 0, 0..self.instances.len() as u32);
    }

    fn update_camera(&self, queue: &wgpu::Queue, camera_uniform_slice: &[CameraUniform]) {
        let camera_buffer = self.shader.get_uniform_buffer("camera_uniform");

        match camera_buffer {
            Some(buffer) => queue.write_buffer(buffer, 0, bytemuck::cast_slice(camera_uniform_slice)),
            None => println!("Couldn't find camera buffer")
        }
    }

    fn update_instance_data(&mut self, instance_index: usize, transform: TransformInstance) -> Result<(), MeshRendererError> {
        match self.instances.get_mut(instance_index) {
            Some(instance) => {
                *instance = transform.clone();
                Ok(())
            },
            None => Err(MeshRendererError::InstanceNotFound)
        }
    }

    fn write_instance_data(&self, queue: &wgpu::Queue) {
        let instance_data: Vec<TransformInstanceRaw> = self.instances.iter().map(TransformInstanceRaw::from).collect();
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instance_data));
    }
}

impl TexturedMesh {
    pub fn from(label: String, device: &wgpu::Device, vertices: &[Vertex], indices: &[u16], shader: Arc<Shader>,
        texture: Texture) -> Result<TexturedMesh, anyhow::Error> {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX
            }
        );

        let instance = TransformInstance {
            position: Vector3::new(-0.5, 0.0, 0.0),
            rotation: Quaternion::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), cgmath::Deg(0.0))
        };

        let instance_0 = TransformInstance {
            position: Vector3::new(0.5, 0.0, 0.0),
            rotation: Quaternion::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), cgmath::Deg(90.0))
        };

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance buffer"),
            size: (20 * std::mem::size_of::<[[f32; 4]; 4]>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let texture_uniform = shader.get_uniform("texture")?;

        let texture_bind_group = Self::create_texture_bind_group(device, "test_texture", &texture_uniform.layout, &texture);
        let camera_uniform = shader.get_uniform("camera_uniform")?;

        //TODO - Recreate camera buffer if it doesn't exist
        let camera_buffer = camera_uniform.buffer.as_ref().expect("camera_uniform doesn't have buffer");
        let camera_bind_group = Self::create_camera_bind_group(device, "camera", &camera_uniform.layout, &camera_buffer);

        Ok(Self {
            label,
            shader,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            texture,
            texture_bind_group,
            camera_bind_group,
            instance_buffer,
            instances: vec![instance, instance_0]
        })
    }


    fn create_camera_bind_group(device: &wgpu::Device, label: &str, layout: &wgpu::BindGroupLayout, camera_buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding()
                }
            ],
            label: Some(&(label.to_string() + BIND_GROUP_POSTFIX))
        })
    }

    fn create_texture_bind_group(device: &wgpu::Device, label: &str, layout: &wgpu::BindGroupLayout, texture: &Texture) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler)
                }
            ],
            label: Some(&(label.to_string() + BIND_GROUP_POSTFIX))
        })
    }
}
