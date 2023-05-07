use log::{info, warn};
use probable_spork_ecs::{component::{Component, ComponentStorage, Entity}};

use crate::{renderer::TransformInstance, scene::Scene};

use super::{Transform, MeshRenderer};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct MeshInstance {
    pub mesh_index: usize,
    pub mesh_instance_index: usize,
    pub local_transform: Transform
}

impl MeshInstance {
    pub fn apply_transforms_to_mesh_instance(scene: &Scene, entity_id: usize, mut mesh_instance: MeshInstance) -> MeshInstance {
        match scene.component_storage.get_entity_component::<Transform>(&Entity(entity_id as u32)) {
            Some(transform) => {
                let transformed_t = Transform {
                    position: mesh_instance.local_transform.position + transform.position,
                    rotation: mesh_instance.local_transform.rotation * transform.rotation
                };
                mesh_instance.local_transform = transformed_t;

                mesh_instance
            },
            None => mesh_instance
        }
    }
}

impl Component for MeshInstance {
    fn setup(&mut self, _world: &ComponentStorage) {
    }
    fn update(&mut self, _world: &ComponentStorage) {
    }
}
