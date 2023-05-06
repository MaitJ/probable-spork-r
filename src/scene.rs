use std::cell::{RefCell, Ref};

use log::{info, warn};
use probable_spork_ecs::{component::{ComponentStorage, Entity, Component}};

use crate::{script::Script, entities::components::MeshInstance};

pub struct Scene {
    pub component_storage: ComponentStorage,
}


impl Scene {
    pub fn new() -> Self {
        Self {
            component_storage: ComponentStorage::new(),
        }
    }

    pub fn setup_components(&self) {
        self.component_storage.setup_components();
    }

    pub fn update_components(&self) {
        self.component_storage.update_components();
    }

    pub fn create_entity(&mut self) -> Entity {
        self.component_storage.create_entity()
    }

    pub fn add_component_to_entity<T: Component + 'static>(&mut self, entity: &Entity, component: T) {
        self.component_storage.register_component(&entity, component);
    }

    pub fn get_mesh_instances(&self) -> Vec<Ref<MeshInstance>> {
        let mesh_instances_opt = self.component_storage.get_component_vec::<MeshInstance>();
        match mesh_instances_opt {
            Some(mesh_instances) => {
                return mesh_instances.iter().map(|m| m.borrow()).collect();
            },
            None => warn!("Couldn't find any mesh instances")
        }
        vec![]
    }


    pub fn add_script_to_entity<T: Script + 'static>(&mut self, entity: &Entity, script: T) {
        let mut boxed_script: Box<dyn Script> = Box::new(script);
        boxed_script.pre_setup(entity.clone(), &mut self.component_storage);
        boxed_script.post_user_update(&self.component_storage);
        self.add_component_to_entity(entity, boxed_script);
    }
}
