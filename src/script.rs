use log::info;
use probable_spork_ecs::{component::{Entity, Component, ComponentStorage}, world::{GameWorld}};

pub trait ScriptComponentUpdater {
    fn pre_setup(&mut self, entity: Entity, world: &mut ComponentStorage);
    fn pre_user_update(&mut self, world: &ComponentStorage);
    fn post_user_update(&mut self, world: &ComponentStorage);
}

pub trait Script: ScriptComponentUpdater {
    fn script_setup(&mut self);
    fn script_update(&mut self);
}


impl Component for Box<dyn Script> {
    //TODO - Maybe I should call pre_setup here aswell
    fn setup(&mut self) {
        info!("called script component update");
        self.script_setup();
    }
    fn update(&mut self, world: &ComponentStorage) {
        self.pre_user_update(world);
        self.script_update();
        self.post_user_update(world);
    }
}
