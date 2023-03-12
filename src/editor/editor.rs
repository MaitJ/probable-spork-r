use std::sync::Arc;

use egui::{ClippedPrimitive, Ui};
use egui_wgpu::{Renderer, renderer::ScreenDescriptor};
use egui_winit::EventResponse;
use log::info;
use winit::{window::Window, event_loop::EventLoop, event::WindowEvent};

use crate::{mesh::Mesh, camera::CameraUniform};

//Something is fucked egui_winit
//Getting a linking error, which uses mingw64
pub struct Editor {
    winit_state: egui_winit::State,
    ctx: egui::Context,
    renderer: egui_wgpu::Renderer,
    screen_descriptor: ScreenDescriptor,
    clipped_primitives: Vec<ClippedPrimitive>,
    pub is_enabled: bool
}

impl Editor {
    pub async fn new(event_loop: &EventLoop<()>, device: &wgpu::Device, window: &Window, texture_format: wgpu::TextureFormat) -> Self {
        info!("Creating editor");
        let winit_state = egui_winit::State::new(event_loop);
        let ctx = egui::Context::default();
        let renderer = Renderer::new(device, texture_format, Some(crate::Texture::DEPTH_FORMAT), 1);

        let screen_descriptor = ScreenDescriptor {
            pixels_per_point: window.scale_factor() as f32,
            size_in_pixels: window.inner_size().into()
        };

        Self {
            winit_state,
            ctx,
            renderer,
            screen_descriptor,
            clipped_primitives: vec![],
            is_enabled: true
        }
    }

    pub fn add_render_resources<'a>(&mut self, meshes: Arc<Vec<Box<dyn Mesh + Send + Sync>>>) {
        self.renderer.paint_callback_resources.insert(meshes);
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.screen_descriptor.size_in_pixels = size.into();
    }

    pub fn rescale(&mut self, pixels_per_point: f32) {
        self.screen_descriptor.pixels_per_point = pixels_per_point;
    }

    pub fn render_ui<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.renderer.render(render_pass, &self.clipped_primitives, &self.screen_descriptor);
    }

    fn draw_game<'a>(&self, ui: &mut Ui, camera_uniform_slice: CameraUniform) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |device, queue, encoder, paint_callback_resources| {
                let meshes:  &Vec<Box<dyn Mesh + Send + Sync>> = paint_callback_resources.get().unwrap();
                meshes.iter().for_each(|mesh| mesh.update_camera(queue, &[camera_uniform_slice]));
                vec![]
            })
            .paint(move |_info, rpass, paint_callback_resources| {
                let meshes:  &Vec<Box<dyn Mesh + Send + Sync>> = paint_callback_resources.get().unwrap();
                meshes.iter().for_each(|mesh| mesh.render(rpass));
            });

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(cb),
        };

        ui.painter().add(callback);
    }

    pub fn update_ui_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, 
            window: &winit::window::Window) {
        let raw_input = self.winit_state.take_egui_input(window);
        let full_output = self.ctx.run(raw_input, |ctx| {
            egui::SidePanel::left("Scene panel").show(ctx, |ui| {
                if ui.button("Click me").clicked() {
                    // take some action here
                    info!("Pressed hello world button");
                }
            });
            egui::TopBottomPanel::bottom("Content browser").show(ctx, |ui| {
                if ui.button("Click me").clicked() {
                    // take some action here
                    info!("Pressed hello world button");
                }
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                });
            });
        });

        let clipped_primitives = self.ctx.tessellate(full_output.shapes); 
        for (id, image_delta) in full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, id, &image_delta);
        }
        self.renderer.update_buffers(device, queue, encoder, &clipped_primitives, &self.screen_descriptor);
        self.clipped_primitives = clipped_primitives;
    }

    pub fn update(&mut self, event: &WindowEvent) -> EventResponse {
        self.winit_state.on_event(&self.ctx, event)
    }
}