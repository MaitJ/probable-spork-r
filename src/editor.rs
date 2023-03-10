use egui::{ClippedPrimitive};
use egui_wgpu::{Renderer, renderer::ScreenDescriptor};
use log::info;
use winit::{window::Window, event_loop::EventLoop, event::WindowEvent};

//Something is fucked egui_winit
//Getting a linking error, which uses mingw64
pub struct Editor {
    winit_state: egui_winit::State,
    ctx: egui::Context,
    renderer: egui_wgpu::Renderer,
    screen_descriptor: ScreenDescriptor,
    clipped_primitives: Vec<ClippedPrimitive>
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
            clipped_primitives: vec![]
        }
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

    pub fn update_ui_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, 
            window: &winit::window::Window) {
        let raw_input = self.winit_state.take_egui_input(window);
        let full_output = self.ctx.run(raw_input, |ctx| {
            egui::Window::new("Debug window").show(&ctx, |ui| {
                ui.label("Hello world!");
                if ui.button("Click me").clicked() {
                    // take some action here
                }
            });
        });

        let clipped_primitives = self.ctx.tessellate(full_output.shapes); 
        for (id, image_delta) in full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, id, &image_delta);
        }
        self.renderer.update_buffers(device, queue, encoder, &clipped_primitives, &self.screen_descriptor);
        self.clipped_primitives = clipped_primitives;
    }

    //TODO - Consume the EventResponse
    pub fn update(&mut self, event: &WindowEvent) {
        self.winit_state.on_event(&self.ctx, event);
    }
}