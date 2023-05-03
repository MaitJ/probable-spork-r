use probable_spork_ecs::{component::{Component, ComponentStorage}, world::GameWorld};

#[derive(Clone)]
pub struct MeshInstance {
    pub mesh_index: usize,
    pub mesh_instance_index: usize
}

impl Component for MeshInstance {
    fn setup(&mut self) {
    }
    fn update(&mut self, world: &ComponentStorage) {
    }
}
