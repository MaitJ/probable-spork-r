use probable_spork_ecs::{component::{Entity, Component, ComponentStorage}, world::{GameWorld}};

pub trait ScriptComponentUpdater {
    fn pre_setup(&mut self, entity: Entity, world: &ComponentStorage);
    fn pre_user_update(&mut self, world: &ComponentStorage);
    fn post_user_update(&mut self, world: &ComponentStorage);
}

pub trait Script: ScriptComponentUpdater {
    fn setup(&mut self);
    fn update(&mut self);
}


impl Component for dyn Script {
    //TODO - Maybe I should call pre_setup here aswell
    fn setup(&self) {
        self.setup();
    }
    fn update(&mut self, world: &ComponentStorage) {
        self.pre_user_update(world);
        Script::update(self);
        self.post_user_update(world);
    }
}
