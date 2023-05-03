use log::{log, info};
use probable_spork_ecs::component::Entity;
use winit::event::WindowEvent;

use crate::{entities::{CameraUniform, CameraController, Camera, components::Transform}, RendererResources, script::Script, scene::Scene, assets::TestScript};

pub struct Engine {
    camera_controller: CameraController,
    camera: Camera,
    pub scene: Scene
}

impl Engine {
    pub fn new(config: &wgpu::SurfaceConfiguration) -> Self {
        let camera = Self::init_camera(&config);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_controller = CameraController::new(0.2);

        Self {
            scene: Scene::new(),
            camera_controller,
            camera,

        }
    }

    pub fn init_camera(config: &wgpu::SurfaceConfiguration) -> Camera {
        Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0
        }
    }

    pub fn setup(&mut self) {
        let test_script = TestScript::default();
        self.scene.initiate_script(test_script);

        self.scene.setup_components();
        self.scene.update_components();

        if let Some(t) = self.scene.component_storage.get_entity_component::<Transform>(&Entity(0)) {
            info!("Actually found transform ({}, {}, {})", t.x, t.y, t.z);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self, renderer_resources: &mut RendererResources) {
        self.scene.update_components();

        let RendererResources { camera_uniform, .. } = renderer_resources;

        self.camera_controller.update_camera(&mut self.camera);
        camera_uniform.update_view_proj(&self.camera);
    }
}
