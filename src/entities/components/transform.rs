use probable_spork_ecs::component::Component;

#[derive(Default, Debug, Clone)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

#[derive(Debug, Clone)]
pub struct Mesh {
    label: &'static str
}

impl Component for Mesh {}
impl Component for Transform {}
