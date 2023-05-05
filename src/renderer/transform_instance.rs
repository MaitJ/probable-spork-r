use cgmath::{Vector3, Quaternion};
use wgpu::VertexAttribute;

pub struct TransformInstance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformInstanceRaw {
    model: [[f32; 4]; 4]
}

impl From<&TransformInstance> for TransformInstanceRaw {
    fn from(value: &TransformInstance) -> Self {
        Self { 
            model: (cgmath::Matrix4::from(value.rotation) * cgmath::Matrix4::from_translation(value.position)).into() 
        }
    }
}

impl TransformInstance {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout { 
            array_stride: std::mem::size_of::<TransformInstanceRaw>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Instance, 
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 5
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 6
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 7
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 8
                }
            ]
        }
    }
}