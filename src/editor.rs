use std::fmt::format;

use egui::{Separator, PaintCallbackInfo, Ui, ClippedPrimitive, TexturesDelta, Label, Sense, Rect, Style, Widget};
use egui_winit::EventResponse;
use log::info;
use winit::{event_loop::EventLoop, event::WindowEvent};

use crate::{RendererResources, renderer::RendererLoop, entities::components::MeshRenderer, scene::Scene};

type UpdateCallback = dyn Fn(
        &wgpu::Device,
        &wgpu::Queue,
        &mut wgpu::CommandEncoder,
        &RendererResources,
        &Vec<Box<dyn MeshRenderer>>
    ) + Send + Sync;

type PaintCallback =
    dyn for<'a, 'b> Fn(PaintCallbackInfo, &'a mut wgpu::RenderPass<'b>, &'b RendererResources, &'b Vec<Box<dyn MeshRenderer>>) + Send + Sync;

pub struct GamePreviewCallback {
    pub update: Box<UpdateCallback>,
    pub render: Box<PaintCallback>
}

pub struct Editor{
    pub ctx: egui::Context,
    pub pixels_per_point: f32,
    winit_state: egui_winit::State,
}

impl Editor {
    pub fn new(event_loop: &EventLoop<()>, window: &winit::window::Window) -> Self {
        let ctx =  egui::Context::default();

        let window_size = window.inner_size();

        let mut pixels_per_point = ctx.pixels_per_point();
        if window_size.width > 1920 && window_size.height > 1080 {
            pixels_per_point *= 2.0;
        }

        ctx.set_pixels_per_point(pixels_per_point);

        let mut winit_state =  egui_winit::State::new(event_loop);
        winit_state.set_pixels_per_point(pixels_per_point);

        Self {
            ctx,
            pixels_per_point,
            winit_state
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> EventResponse {
        self.winit_state.on_event(&self.ctx, event)
    }

    fn setup_game_preview_callback<'a>(&self, ui: &mut Ui) {
        let available_size = ui.available_size();
        let (rect, _response) = ui.allocate_at_least(available_size, egui::Sense::drag());

        let cb = GamePreviewCallback {
            update: Box::new(|_device, queue, _encoder, renderer_resources, meshes| RendererLoop::update(queue, renderer_resources, meshes)),
            render: Box::new(|_info, rpass, renderer_resources, meshes| RendererLoop::render(rpass, renderer_resources, meshes))
        };

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(cb),
        };

        ui.painter().add(callback);
    }


    pub fn draw(&mut self, window: &winit::window::Window, _renderer_resources: &RendererResources, scene: &Scene) -> (TexturesDelta, Vec<ClippedPrimitive>) {
        let raw_input = self.winit_state.take_egui_input(window);
        let full_output = self.ctx.run(raw_input, |ctx| {
            egui::SidePanel::left("Scene panel").show(ctx, |ui| {
                ui.heading("Scene");
                ui.add(Separator::default().horizontal());
                egui::Frame::menu(&Style::default())
                    .fill(egui::Color32::BLACK)
                    .show(ui, |ui| ui.add(EntityList::new(scene.component_storage.entities)))
            });
            egui::SidePanel::right("Right panel").show(ctx, |ui| {
                ui.heading("Properties");
                ui.add(Separator::default().horizontal());
                if ui.button("Click me").clicked() {
                    // take some action here
                    info!("Pressed hello world button");
                }
            });
            egui::TopBottomPanel::bottom("Content browser").show(ctx, |ui| {
                ui.heading("Content browser");
                ui.add(Separator::default().horizontal());
                if ui.button("Click me").clicked() {
                    // take some action here
                    info!("Content browser button press");
                }
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::Frame::canvas(ui.style())
                .show(ui, |ui| {
                    let available_size = ui.available_size();
                    ui.set_min_height(available_size.y * 0.3);
                    ui.set_min_width(available_size.x * 0.6);
                    self.setup_game_preview_callback(ui);
                });
            });
        });

        let clipped_primitives = self.ctx.tessellate(full_output.shapes);
        (full_output.textures_delta, clipped_primitives)
    }

}

struct EntityList {
    entities: u32
}

impl EntityList {
    fn new(entities: u32) -> Self {
        Self {
            entities
        }
    }

    fn scene_context_menu(ui: &mut Ui) {
        let _ = ui.button("Add empty");
    }

    fn entity_context_menu(ui: &mut Ui) {
        let _ = ui.button("Rename");
        let _ = ui.button("Remove");
    }
}

impl Widget for EntityList{
    fn ui(self, ui: &mut Ui) -> egui::Response {
        ui.vertical(|ui| {
            for entity in 0..self.entities {
                let ent_response = ui.add(Label::new(format!("Entity {}", entity)).sense(Sense::click()));
                ent_response.context_menu(EntityList::entity_context_menu);
            }
        });

        let response = ui.allocate_response(egui::vec2(ui.available_width(), ui.available_height()), Sense::click());
        response.context_menu(EntityList::scene_context_menu)
    }
}
