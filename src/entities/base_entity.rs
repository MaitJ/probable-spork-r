use log::info;

use crate::renderer::{TexturedMesh, Mesh};
use std::{any::Any, rc::Rc};


pub trait Script {
    fn setup(&mut self);
    fn update(&mut self);
}

pub trait Component {
    fn setup(&mut self);
    fn update(&mut self);
    fn as_any(self: Rc<Self>) -> Rc<dyn Any>;
}


pub struct Entity {
    id: u32,
    label: String,
    script: Option<Box<dyn Script>>,
    children: Vec<Entity>,
    components: Vec<Rc<dyn Component>>
}

impl Entity {
    pub fn new(id: u32, label: String) -> Self {
        Self {
            id,
            label,
            script: None,
            children: vec![],
            components: vec![]
        }
    }

    pub fn call_script_setup(&mut self) {
        if let Some(script) = &mut self.script {
            script.setup();
        }

        self.children
            .iter_mut()
            .for_each(|child_entity| child_entity.call_script_setup());
    }
    
    pub fn call_script_update(&mut self) {
        if let Some(script) = &mut self.script {
            script.update();
        }

        self.children
            .iter_mut()
            .for_each(|child_entity| child_entity.call_script_update());
    }

    pub fn get_component<T: Component + 'static>(&self) -> Option<Rc<T>> {
        for component in self.components.iter() {
            if let Ok(c) = component.clone().as_any().downcast::<T>() {
                return Some(c);
            }
        }
        None
    }

    pub fn add_component(&mut self, component: Rc<dyn Component>) {
        self.components.push(component);
    }

    pub fn add_script(&mut self, script: Box<dyn Script>) -> &mut Self {
        self.script = Some(script);
        self
    }
}