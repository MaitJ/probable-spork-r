
pub trait Script {
    fn setup(&mut self);
    fn update(&mut self);
}

pub trait Component {
    fn setup(&mut self);
    fn update(&mut self);
}

pub struct Entity {
    id: u32,
    label: String,
    script: Option<Box<dyn Script>>,
    children: Vec<Entity>
}

impl Entity {
    pub fn new(id: u32, label: String) -> Self {
        Self {
            id,
            label,
            script: None,
            children: vec![]
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

    pub fn add_script(&mut self, script: Box<dyn Script>) {
        self.script = Some(script);
    }
}