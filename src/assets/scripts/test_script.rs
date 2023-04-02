use log::info;

use crate::entities::Script;

#[derive(Default)]
pub struct TestScript {
    test_var: i32
}

impl Script for TestScript {
    fn setup(&mut self) {
        self.test_var = 22;
    }

    fn update(&mut self) {
    }
}