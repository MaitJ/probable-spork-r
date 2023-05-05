use log::info;
use probable_spork_ecs::{component::{ComponentStorage}};

use crate::{script::Script};

pub struct Scene {
    pub component_storage: ComponentStorage,
    user_scripts: Vec<Box<dyn Script>>
}


impl Scene {
    pub fn new() -> Self {
        Self {
            component_storage: ComponentStorage::new(),
            user_scripts: vec![]
        }
    }

    pub fn setup_components(&self) {
        self.component_storage.setup_components();
    }

    pub fn update_components(&self) {
        self.component_storage.update_components();
    }

    pub fn initiate_script<T: Script + 'static>(&mut self, script: T) {
        let mut boxed_script: Box<dyn Script> = Box::new(script);
        let entity = self.component_storage.create_entity();
        info!("entity: {}", entity.0);
        boxed_script.pre_setup(entity.clone(), &mut self.component_storage);
        boxed_script.post_user_update(&self.component_storage);
        self.component_storage.register_component(&entity, boxed_script);

        //self.user_scripts.push(boxed_script);
    }
}
