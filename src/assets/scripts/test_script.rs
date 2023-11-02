use cgmath::{Deg, Quaternion, Rotation3, Vector3};
use log::info;
use probable_spork_ecs::component::Entity;
use script_gen_macro::ScriptComponentUpdater;

use crate::entities::components::MeshInstance;
use crate::script::ScriptComponentUpdater;
use crate::{entities::components::Transform, script::Script};

// Get rid of this crates entities
// Move Script trait to appropriate place
#[derive(ScriptComponentUpdater, Default)]
pub struct TestScript {
    entity: Entity,
    #[SyncComponent]
    transform: Transform,
    #[SyncComponent]
    mesh: MeshInstance,
}

impl Script for TestScript {
    fn script_setup(&mut self) {
        self.mesh.local_transform.rotation =
            Quaternion::from_axis_angle(Vector3::new(1.0, 0.0, 0.0), Deg(-45.0));
        self.transform.position.x += 4.5;
        self.transform.rotation =
            Quaternion::from_axis_angle(Vector3::new(1.0, 0.0, 0.0), Deg(-45.0));
    }

    fn script_update(&mut self) {}
}
