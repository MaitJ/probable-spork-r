use log::info;
use probable_spork_ecs::world::World;

use crate::{script::Script, entities::components::Transform};

pub struct Scene {
    pub world: World,
    user_scripts: Vec<Box<dyn Script>>
}

impl Scene {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            user_scripts: vec![]
        }
    }

    pub fn initiate_script<T: Script + 'static>(&mut self, script: T) {
        let mut boxed_script = Box::new(script);
        let entity = self.world.create_entity();
        boxed_script.pre_setup(entity, &mut self.world);
        self.user_scripts.push(boxed_script);
    }

    pub fn script_setup(&mut self) {
        self.user_scripts
            .iter_mut()
            .for_each(|script| {
                script.setup();
            });

        info!("{:?}", self.world.component_storage.get_component_vec::<Transform>());
    }

    pub fn script_update(&mut self) {
        self.user_scripts
            .iter_mut()
            .for_each(|script| {
                script.pre_user_update(&self.world);
                script.update();
                script.post_user_update(&mut self.world)
            })
    }
}
