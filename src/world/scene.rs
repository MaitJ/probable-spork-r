use std::rc::Rc;

use crate::{entities::{Entity, Script}, mesh::TexturedMesh};

pub struct Scene {
    entity_count: u32,
    entities: Vec<Entity>
}

impl Scene {
    pub fn new() -> Self {
        Scene { entity_count: 0, entities: vec![] }
    }

    pub fn get_entity(&self, id: u32) -> Option<&Entity> {
        self.entities.get(id as usize)
    }

    pub fn get_entity_mut(&mut self, id: u32) -> Option<&mut Entity> {
        self.entities.get_mut(id as usize)
    }

    pub fn add_empty_entity(&mut self) -> &mut Entity {
        let label = format!("EmptyEntity{}", self.entity_count);
        self.entities.push(Entity::new(self.entity_count, label));
        self.entity_count += 1;

        self.entities.get_mut((self.entity_count - 1) as usize).unwrap()
    }

    pub fn get_renderables(&self) -> Vec<Rc<TexturedMesh>> {
        self.entities
            .iter()
            .map(|entity| entity.get_component::<TexturedMesh>())
            .filter_map(|e| e)
            .collect()
    }

    pub fn call_user_script_setups(&mut self) {
        self.entities
            .iter_mut()
            .for_each(|entity| entity.call_script_setup());
    }

    pub fn call_user_script_updates(&mut self) {
        self.entities
            .iter_mut()
            .for_each(|entity| entity.call_script_update());
    }
}