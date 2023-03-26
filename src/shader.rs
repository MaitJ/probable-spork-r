use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::fs;
use std::path::Path;

use crate::errors::{GeneralError, ErrorIdentificator};
use crate::texture::Texture;
use crate::vertex::Vertex;
use bytemuck::{Pod, Zeroable};
use log::{debug, info};
use wgpu::util::DeviceExt;

//Should change because right now I have to start it
//from src/
const SHADER_FOLDER: &str = "/shaders/";
const SOURCE_FOLDER: &str = "/src/";
const BIND_GROUP_LAYOUT_POSTFIX: &str = "_bind_group_layout";
pub const BIND_GROUP_POSTFIX: &str = "_bind_group";

#[derive(Debug)]
pub enum ShaderBuilderError {
    ShaderNotLoaded
}

impl Display for ShaderBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShaderNotLoaded => write!(f, "Shader hasn't been loaded")
        }
    }
}

impl Error for ShaderBuilderError {}

pub struct ShaderUniform {
    pub label: &'static str,
    pub layout: wgpu::BindGroupLayout,
    pub buffer: Option<wgpu::Buffer>,
}

pub struct ShaderBuilder {
    label: &'static str,
    uniforms: Vec<ShaderUniform>,
    shader: Option<wgpu::ShaderModule>
}

pub struct Shader {
    pub label: &'static str,
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniforms: Vec<ShaderUniform>
}

impl Shader {
    pub fn get_uniform(&self, label: &'static str) -> Result<&ShaderUniform, GeneralError> {
        match self.uniforms.iter()
            .find(|uniform| uniform.label == label) {
            Some(uniform) => Ok(uniform),
            None => Err(GeneralError::NotFound(ErrorIdentificator {
                fn_call: "Shader::get_uniform",
                arg: label
            }))
        }
    }

    pub fn get_uniform_buffer(&self, label: &str) -> Option<&wgpu::Buffer> {
        let uniform = self.uniforms.iter()
            .find(|uniform| uniform.label == label);

        let buffer = if let Some(u) = uniform {
            u.buffer.as_ref()
        } else { None };

        return buffer;
    }
}

impl ShaderBuilder {
    pub fn new() -> Self {
        Self {
            label: "",
            uniforms: vec![],
            shader: None
        }
    }

    pub fn load_shader(mut self, device: &wgpu::Device, file_name: &'static str) -> Result<Self, anyhow::Error> {
        let project_root = env!("CARGO_MANIFEST_DIR");

        let mut file_path = String::from(project_root);
        file_path.push_str(SOURCE_FOLDER);
        file_path.push_str(SHADER_FOLDER);
        file_path.push_str(file_name);
        info!("Loading shader: {}", file_path);

        let contents = fs::read_to_string(Path::new(&file_path))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(file_name),
            source: wgpu::ShaderSource::Wgsl(Cow::from(contents))
        });

        self.shader = Some(shader);
        self.label = file_name;
        Ok(self)
    }

    pub fn add_uniform<T>(mut self, device: &wgpu::Device, label: &'static str, data: T)
        -> Self
        where T: Pod + Zeroable + Copy + Clone + Debug {
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(&[data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let buffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ],
            label: Some(&(label.to_string() + BIND_GROUP_LAYOUT_POSTFIX))
        });

        self.uniforms.push(ShaderUniform {
            label,
            layout: buffer_bind_group_layout,
            buffer: Some(buffer)
        });
        info!("Added uniform ({}) to shader, group: {}", label, self.uniforms.len() - 1);

        self
    }

    pub fn add_texture(mut self, device: &wgpu::Device, label: &'static str) -> Self {
        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture { 
                            sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                            view_dimension: wgpu::TextureViewDimension::D2, 
                            multisampled: false
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None
                    }
                ],
                label: Some(&(label.to_string() + BIND_GROUP_LAYOUT_POSTFIX))
            }
        );

        self.uniforms.push(ShaderUniform {
            label,
            layout: texture_bind_group_layout,
            buffer: None
        });
        info!("Added texture ({}) to shader, group: {}", label, self.uniforms.len() - 1);

        self
    }

    pub fn build(self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Result<Shader, ShaderBuilderError> {
        let layouts_ref = self.uniforms.iter().map(|layout| &layout.layout).collect::<Vec<_>>();

        info!("Building shader: {}, uniforms: {}", self.label, self.uniforms.len());

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: &layouts_ref[..],
            push_constant_ranges:&[]
        });

        match self.shader {
            Some(shader) => {
                let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::desc()]
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState { 
                            format: config.format, 
                            blend: Some(wgpu::BlendState::REPLACE), 
                            write_mask: wgpu::ColorWrites::ALL
                        })]
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: Texture::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default()
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false
                    },
                    multiview: None
                });

                Ok(Shader {
                    label: self.label,
                    render_pipeline,
                    uniforms: self.uniforms
                })
            },
            None => Err(ShaderBuilderError::ShaderNotLoaded)
        }
    }
}