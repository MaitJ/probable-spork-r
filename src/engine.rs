use log::{warn, info};
use winit::event::WindowEvent;

use crate::{entities::{CameraUniform, CameraController, Camera, components::{MeshInstance, MeshRenderer, Transform}}, RendererResources, scene::Scene, assets::TestScript, renderer::Renderer};

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

    pub fn setup(&mut self, renderer: &mut impl Renderer) {
        let test_script = TestScript::default();
        let mut entity = self.scene.create_entity();
        self.scene.add_script_to_entity(&entity, test_script);


        let mesh_index = 0;
        let mesh_instance_index = renderer.get_mesh_manager_mut().create_mesh_instance(mesh_index);

        match mesh_instance_index {
            Some(mesh_instance_index) => {
                let mesh_instance = MeshInstance {
                    mesh_index,
                    mesh_instance_index,
                    local_transform: Transform::default()
                };

                self.scene.update_entity_component(&entity, mesh_instance);
            },
            None => warn!("Couldnt find mesh/mesh_instance")
        }


        entity = self.scene.create_entity();

        let mesh_instance_index = renderer.get_mesh_manager_mut().create_mesh_instance(mesh_index);

        if let Some(mesh_instance_index) = mesh_instance_index {
            let mut transform = Transform::default();
            transform.position.x = -4.0;

            let mesh_instance = MeshInstance {
                mesh_index,
                mesh_instance_index,
                local_transform: transform
            };
            self.scene.add_component_to_entity(&entity, mesh_instance);
        }

        self.scene.setup_components();
        self.scene.update_components();
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
