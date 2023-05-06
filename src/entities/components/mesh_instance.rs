use log::{info, warn};
use probable_spork_ecs::{component::{Component, ComponentStorage}};

use crate::renderer::TransformInstance;

use super::{Transform, MeshRenderer};

#[derive(Clone, PartialEq, Default)]
pub struct MeshInstance {
    pub mesh_index: usize,
    pub mesh_instance_index: usize,
    pub local_transform: Transform
}

impl Component for MeshInstance {
    fn setup(&mut self) {
    }
    fn update(&mut self, _world: &ComponentStorage) {
    }
}
