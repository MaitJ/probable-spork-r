use cgmath::Vector3;
use log::info;
use probable_spork_ecs::component::Entity;
use script_gen_macro::ScriptComponentUpdater;

use crate::entities::components::MeshInstance;
use crate::{entities::components::Transform, script::Script};
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

//impl ScriptComponentUpdater for TestScript {
//    fn post_user_update(&mut self, world: &probable_spork_ecs::component::ComponentStorage) {
//        if let Some(mut c) = world.get_entity_component_mut::<Transform>(&self.entity) {
//            // TODO - Please replace this with a thing that checks if an updated is needed
//            *c = self.transform.clone()
//        }
//    }
//    fn pre_setup(&mut self, entity: Entity, world: &mut probable_spork_ecs::component::ComponentStorage) {
//        world.register_component::<Transform>(&self.entity, self.transform.clone());
//        world.register_component::<MeshInstance>(&self.entity, self.mesh.clone());
//    }
//    fn pre_user_update(&mut self, world: &probable_spork_ecs::component::ComponentStorage) {
//        if let Some(c) = world.get_entity_component::<Transform>(&self.entity) {
//            // todo - please replace this with a thing that checks if an updated is needed
//            self.transform = c.clone()
//        }
//        if let Some(c) = world.get_entity_component::<MeshInstance>(&self.entity) {
//            // todo - please replace this with a thing that checks if an updated is needed
//            self.mesh = c.clone()
//        }
//    }
//}

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
    fn script_setup(&mut self) {
        self.transform.x = 5.0;

    }

    fn script_update(&mut self) {
        self.transform.x += 0.01;
        let Transform {x, y, ..} = &self.transform;
        info!("position ({}, {})", x, y);
    }
}
