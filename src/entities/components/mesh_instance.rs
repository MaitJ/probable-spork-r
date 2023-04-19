use probable_spork_ecs::component::Component;

use super::Transform;

#[derive(Clone)]
pub struct MeshInstance {
    pub mesh_index: usize,
    pub transform: Transform
}

impl Component for MeshInstance {}
