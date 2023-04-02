use egui::epaint::Primitive;
use egui::{ClippedPrimitive, Ui, Separator, PaintCallbackInfo};
use egui_wgpu::renderer::ScreenDescriptor;
use egui_winit::EventResponse;
use log::info;
use wgpu::{Color, RenderPass};
use winit::{window::Window, event_loop::EventLoop, event::WindowEvent};
use crate::entities::CameraUniform;

use crate::{mesh::Mesh, WgpuStructs, renderer::Renderer, texture::Texture, RendererResources};
pub struct Editor {
    depth_texture: Texture,
    winit_state: egui_winit::State,
    ctx: egui::Context,
    renderer: egui_wgpu::Renderer,
    screen_descriptor: ScreenDescriptor,
    clipped_primitives: Vec<ClippedPrimitive>,
    renderables: Vec<Box<dyn Mesh + Send + Sync>>,
    pub is_enabled: bool
}

type UpdateCallback = dyn Fn(
        &wgpu::Device,
        &wgpu::Queue,
        &mut wgpu::CommandEncoder,
        &Vec<Box<dyn Mesh + Send + Sync>>,
        &CameraUniform
    ) + Send + Sync;

type PaintCallback =
    dyn for<'a, 'b> Fn(PaintCallbackInfo, &'a mut wgpu::RenderPass<'b>, &'b Vec<Box<dyn Mesh + Send + Sync>>) + Send + Sync;

struct GamePreviewCallback {
    update: Box<UpdateCallback>,
    render: Box<PaintCallback>
}

impl Editor {
    pub async fn new(event_loop: &EventLoop<()>, wgpu_structs: &WgpuStructs, window: &Window, texture_format: wgpu::TextureFormat) -> Self {
        info!("Creating editor");

        let WgpuStructs { device, config, .. } = wgpu_structs;

        let mut winit_state = egui_winit::State::new(event_loop);
        let ctx = egui::Context::default();
        let mut renderer = egui_wgpu::Renderer::new(device, texture_format, Some(crate::Texture::DEPTH_FORMAT), 1);
        let meshes: Vec<Box<dyn Mesh + Send + Sync>> = vec![];

        let camera_uniform = CameraUniform::new();

        renderer.paint_callback_resources.insert(meshes);
        renderer.paint_callback_resources.insert(camera_uniform);

        info!("Pixels per point: {}", ctx.pixels_per_point());

        let scaled_pixels_per_point = ctx.pixels_per_point() * 2.0;
        ctx.set_pixels_per_point(scaled_pixels_per_point);

        winit_state.set_pixels_per_point(scaled_pixels_per_point);

        let screen_descriptor = ScreenDescriptor {
            pixels_per_point: scaled_pixels_per_point,
            size_in_pixels: window.inner_size().into()
        };

        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth texture");

        Self {
            depth_texture,
            winit_state,
            ctx,
            renderer,
            screen_descriptor,
            clipped_primitives: vec![],
            is_enabled: true,
            renderables: vec![]
        }
    }

    fn setup_callback<'a>(&self, ui: &mut Ui) {
        let available_size = ui.available_size();
        let (rect, _response) = ui.allocate_at_least(available_size, egui::Sense::drag());
        info!("setup_callback");

        let cb = GamePreviewCallback {
            update: Box::new(|_device, queue, _encoder, meshes, camera_uniform| {
                meshes.iter().for_each(|mesh| mesh.update_camera(queue, &[*camera_uniform]));
            }),
            render: Box::new(|_info, rpass, meshes| {
                meshes.iter().for_each(|mesh| mesh.render(rpass));
            })
        };

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
                ui.heading("Scene");
                ui.add(Separator::default().horizontal());

                self.renderables
                    .iter()
                    .enumerate()
                    .for_each(|(i, _mesh)| {
                        ui.label(format!("Mesh {}", i));
                    })
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
                    self.setup_callback(ui);
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
    
    fn call_game_preview_update(&self, device: &wgpu::Device, queue: &wgpu::Queue, 
        encoder: &mut wgpu::CommandEncoder, clipped_primitive: &Vec<ClippedPrimitive>, renderer_resources: &RendererResources) {
        for egui::epaint::ClippedPrimitive {
            clip_rect: _,
            primitive
        } in clipped_primitive {
            match primitive {
                Primitive::Callback(callback) => {
                    let cbfn = if let Some(c) = callback.callback.downcast_ref::<GamePreviewCallback>() {
                        c
                    } else {
                        // We already warned in the `prepare` callback
                        continue;
                    };

                    (cbfn.update)(
                        device,
                        queue,
                        encoder,
                        &self.renderables,
                        &renderer_resources.camera_uniform
                    );
                },
                _ => ()
            }
        }
    }

    fn call_game_preview_render<'a>(&'a self, render_pass: &mut RenderPass<'a>, clipped_primitive: &Vec<ClippedPrimitive>) {
        for egui::epaint::ClippedPrimitive {
            clip_rect,
            primitive
        } in clipped_primitive {
            match primitive {
                Primitive::Callback(callback) => {
                    let cbfn = if let Some(c) = callback.callback.downcast_ref::<GamePreviewCallback>() {
                        c
                    } else {
                        continue;
                    };

                    let pixels_per_point = self.ctx.pixels_per_point();

                    {

                        let min = (callback.rect.min.to_vec2() * pixels_per_point).round();
                        let max = (callback.rect.max.to_vec2() * pixels_per_point).round();

                        render_pass.set_viewport(
                            min.x,
                            min.y,
                            max.x - min.x,
                            max.y - min.y,
                            0.0,
                            1.0,
                        );
                    }

                    (cbfn.render)(
                        PaintCallbackInfo {
                            viewport: callback.rect,
                            clip_rect: *clip_rect,
                            pixels_per_point,
                            screen_size_px: self.screen_descriptor.size_in_pixels,
                        },
                        render_pass,
                        &self.renderables
                    );
                },
                _ => ()
            }
        }
    }

    fn update(&mut self, wgpu_structs: &WgpuStructs, renderer_resources: &crate::RendererResources) {
        let RendererResources { camera_uniform } = &renderer_resources;
        self.renderables.iter().for_each(|mesh| mesh.update_camera(&wgpu_structs.queue, &[*camera_uniform]));
    }
}

impl Renderer for Editor {
    fn add_mesh(&mut self, mesh: Box<dyn Mesh + Send + Sync>) {
        self.renderables.push(mesh);
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>, scale_factor: Option<f32>, depth_texture: Option<Texture>) {
        match depth_texture {
            Some(depth_texture) => self.depth_texture = depth_texture,
            None => ()
        }

        self.screen_descriptor.size_in_pixels = size.into();

        if let Some(pixels_per_point) = scale_factor {
            self.screen_descriptor.pixels_per_point = pixels_per_point;
        }
    }


    fn render<'a>(&'a mut self, wgpu_structs: &WgpuStructs, window: &Window, renderer_resources: &RendererResources) -> Result<(), wgpu::SurfaceError> {
        self.update(wgpu_structs, renderer_resources);

        let WgpuStructs { surface, device, queue, .. } = wgpu_structs;
        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        self.update_ui_textures(device, queue, &mut encoder, window);
        self.call_game_preview_update(device, queue, &mut encoder, &self.clipped_primitives, renderer_resources);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Editor render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view: &view, 
                    resolve_target: None, 
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0
                        }),
                        store: true
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true
                    }),
                    stencil_ops: None
                })
            });
            self.renderer.render(&mut render_pass, &self.clipped_primitives, &self.screen_descriptor);
            self.call_game_preview_render(&mut render_pass, &self.clipped_primitives);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }


    fn handle_event(&mut self, event: &WindowEvent) -> EventResponse {
        self.winit_state.on_event(&self.ctx, event)
    }
}