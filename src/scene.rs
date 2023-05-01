use log::info;
use probable_spork_ecs::{world::GameWorld, component::ComponentStorage};

use crate::{script::Script, entities::components::Transform};

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
        let mut boxed_script = Box::new(script);
        let entity = self.component_storage.create_entity();
        boxed_script.pre_setup(entity, &self.component_storage);
        self.user_scripts.push(boxed_script);
    }

    pub fn script_setup(&mut self) {
        self.user_scripts
            .iter_mut()
            .for_each(|script| {
                script.setup();
            });
    }

    pub fn script_update(&mut self) {
        self.user_scripts
            .iter_mut()
            .for_each(|script| {
                script.pre_user_update(&self.component_storage);
                script.update();
                script.post_user_update(&self.component_storage)
            })
    }
}
