use log::warn;

use crate::entities::components::MeshRenderer;

pub struct MeshManager {
    meshes: Vec<Box<dyn MeshRenderer>>
}

impl MeshManager {
    pub fn new() -> Self {
        Self {
            meshes: vec![]
        }
    }

    pub fn get_mesh(&self, mesh_index: usize) -> Option<&Box<dyn MeshRenderer>> {
        self.meshes.get(mesh_index)
    }
    pub fn create_mesh_instance(&mut self, mesh_index: usize) -> Option<usize> {
        match self.meshes.get_mut(mesh_index) {
            Some(mesh) => {
                Some(mesh.create_instance())
            },
            None => {
                warn!("Couldn't find mesh at index {}", mesh_index);
                None
            }
        }
    }
    pub fn add_mesh(&mut self, mesh: impl MeshRenderer + 'static) {
        self.meshes.push(Box::new(mesh));
    }
    pub fn get_meshes(&self) -> &Vec<Box<dyn MeshRenderer>> {
         &self.meshes
    }
    pub fn get_meshes_mut(&mut self) -> &mut Vec<Box<dyn MeshRenderer>> {
         &mut self.meshes
    }
}
