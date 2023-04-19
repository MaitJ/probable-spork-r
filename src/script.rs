use probable_spork_ecs::{component::Entity, world::World};

pub trait ScriptComponentUpdater {
    fn pre_setup(&mut self, entity: Entity, world: &mut World);
    fn pre_user_update(&mut self, world: &World);
    fn post_user_update(&mut self, world: &mut World);
}

pub trait Script: ScriptComponentUpdater {
    fn setup(&mut self);
    fn update(&mut self);
}
