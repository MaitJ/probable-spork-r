
use cgmath::{Vector3, Quaternion, Rotation3, Deg};
use log::info;
use probable_spork_ecs::component::Entity;
use script_gen_macro::ScriptComponentUpdater;

use crate::entities::components::MeshInstance;
use crate::{entities::components::Transform, script::Script};
use crate::script::ScriptComponentUpdater;


// Get rid of this crates entities
// Move Script trait to appropriate place
#[derive(ScriptComponentUpdater, Default)]
pub struct TestScript {
    entity: Entity,
    #[SyncComponent]
    transform: Transform,
    #[SyncComponent]
    mesh: MeshInstance
}

impl Script for TestScript {
    fn script_setup(&mut self) {
        self.mesh.local_transform.rotation = Quaternion::from_axis_angle(Vector3::new(1.0, 0.0, 0.0), Deg(45.0));
        self.transform.position.x += 2.5;
        self.transform.rotation = Quaternion::from_axis_angle(Vector3::new(1.0, 0.0, 0.0), Deg(45.0));
    }

    fn script_update(&mut self) {
    }
}
