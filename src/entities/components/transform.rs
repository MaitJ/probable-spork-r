use cgmath::{Vector3, Quaternion};
use probable_spork_ecs::{component::{Component, ComponentStorage}};

use crate::renderer::TransformInstance;

#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0)
        }
    }
}

impl From<&Transform> for TransformInstance {
    fn from(value: &Transform) -> Self {
        Self {
            position: value.position,
            rotation: value.rotation
        }
    }
}

impl Component for Transform {
    fn setup(&mut self, _world: &ComponentStorage) {
    }
    fn update(&mut self, _world: &ComponentStorage) {
    }
}
