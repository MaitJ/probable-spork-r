use probable_spork_ecs::{component::{Component, ComponentStorage}};

#[derive(Clone, PartialEq)]
pub struct MeshInstance {
    pub mesh_index: usize,
    pub mesh_instance_index: usize
}

impl Component for MeshInstance {
    fn setup(&mut self) {
    }
    fn update(&mut self, _world: &ComponentStorage) {
    }
}
