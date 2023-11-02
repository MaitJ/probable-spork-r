use probable_spork_ecs::component::{Component, ComponentStorage, Entity};

pub trait ScriptComponentUpdater {
    fn pre_setup(&mut self, entity: Entity, world: &mut ComponentStorage);
    fn pre_user_update(&mut self, world: &ComponentStorage);
    fn post_user_update(&mut self, world: &ComponentStorage);
}

pub trait Script: ScriptComponentUpdater {
    fn script_setup(&mut self);
    fn script_update(&mut self);
}

impl PartialEq for Box<dyn Script> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
    fn ne(&self, _other: &Self) -> bool {
        false
    }
}

impl Component for Box<dyn Script> {
    fn setup(&mut self, world: &ComponentStorage) {
        self.script_setup();
        self.post_user_update(world);
    }
    fn update(&mut self, world: &ComponentStorage) {
        self.pre_user_update(world);
        self.script_update();
        self.post_user_update(world);
    }
}
