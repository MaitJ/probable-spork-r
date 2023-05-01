use cgmath::Vector3;
use log::info;
use probable_spork_ecs::component::Entity;
use script_gen_macro::ScriptComponentUpdater;

use crate::entities::components::MeshInstance;
use crate::{entities::components::Transform, script::Script, renderer::TexturedMesh};
use crate::script::ScriptComponentUpdater;


// Get rid of this crates entities
// Move Script trait to appropriate place
#[derive(ScriptComponentUpdater)]
pub struct TestScript {
    entity: Entity,
    #[SyncComponent]
    transform: Transform,
    #[SyncComponent]
    mesh: MeshInstance
}

impl TestScript {
    pub fn default() -> Self {
        Self {
            entity: Entity(0),
            transform: Transform::default(),
            mesh: MeshInstance {
                mesh_index: 0,
                mesh_instance_index: 0
            }
        }
    }
}

impl Script for TestScript {
    fn setup(&mut self) {
    }

    fn update(&mut self) {
        let Transform {x, y, ..} = &self.transform;
        info!("position ({}, {})", x, y);
    }
}
