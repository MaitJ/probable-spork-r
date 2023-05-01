use probable_spork_ecs::{component::{Component, ComponentStorage}, world::GameWorld};

#[derive(Default, Debug, Clone)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Component for Transform {
    fn setup(&self) {
    }
    fn update(&mut self, world: &ComponentStorage) {
    }
}
