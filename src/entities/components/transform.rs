use probable_spork_ecs::{component::{Component, ComponentStorage}};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Component for Transform {
    fn setup(&mut self) {
    }
    fn update(&mut self, _world: &ComponentStorage) {
    }
}
