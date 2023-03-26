use std::{rc::Rc, sync::Arc};

use wgpu::util::DeviceExt;
use crate::entities::{Camera, CameraUniform};
use crate::{vertex::Vertex, shader::{Shader, BIND_GROUP_POSTFIX}, texture::Texture};

pub trait Mesh {
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
    fn update_camera(&self, queue: &wgpu::Queue, camera_uniform_slice: &[CameraUniform]);
}

pub struct TexturedMesh {
    pub shader: Arc<Shader>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub texture: Texture,
    pub texture_bind_group: wgpu::BindGroup,
    pub camera_bind_group: wgpu::BindGroup
}

impl Mesh for TexturedMesh {
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.shader.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }

    fn update_camera(&self, queue: &wgpu::Queue, camera_uniform_slice: &[CameraUniform]) {
        let camera_buffer = self.shader.get_uniform_buffer("camera_uniform");

        match camera_buffer {
            Some(buffer) => queue.write_buffer(buffer, 0, bytemuck::cast_slice(camera_uniform_slice)),
            None => println!("Couldn't find camera buffer")
        }
    }
}

impl TexturedMesh {
    pub fn from(device: &wgpu::Device, vertices: &[Vertex], indices: &[u16], shader: Arc<Shader>, 
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

        let texture_uniform = shader.get_uniform("texture")?;

        let texture_bind_group = Self::create_texture_bind_group(device, "test_texture", &texture_uniform.layout, &texture);
        let camera_uniform = shader.get_uniform("camera_uniform")?;

        //TODO - Recreate camera buffer if it doesn't exist
        let camera_buffer = camera_uniform.buffer.as_ref().expect("camera_uniform doesn't have buffer");
        let camera_bind_group = Self::create_camera_bind_group(device, "camera", &camera_uniform.layout, &camera_buffer);

        Ok(Self {
            shader,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            texture,
            texture_bind_group,
            camera_bind_group
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