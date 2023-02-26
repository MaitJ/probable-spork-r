use winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

use crate::camera::Camera;

pub struct CameraController {
    speed: f32,
    //forward-backward
    is_y_pressed: (bool, bool),
    //left-right
    is_x_pressed: (bool, bool)
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_x_pressed: (false, false),
            is_y_pressed: (false, false)
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { input: KeyboardInput {
                state,
                virtual_keycode: Some(keycode),
                ..
            }, ..} => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W => {
                        self.is_y_pressed.0 = is_pressed;
                        true
                    },
                    VirtualKeyCode::S => {
                        self.is_y_pressed.1 = is_pressed;
                        true
                    },
                    VirtualKeyCode::A => {
                        self.is_x_pressed.0 = is_pressed;
                        true
                    },
                    VirtualKeyCode::D => {
                        self.is_x_pressed.1 = is_pressed;
                        true
                    },
                    _ => false
                }
            },
            _ => false
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_normalized = forward.normalize();
        let forward_magnitude = forward.magnitude();

        if self.is_y_pressed.0 && forward_magnitude > self.speed {
            camera.eye += forward_normalized * self.speed;
        }
        if self.is_y_pressed.1 {
            camera.eye -= forward_normalized * self.speed;
        }

        let right = forward_normalized.cross(camera.up);

        let forward = camera.target - camera.eye;
        let forward_magnitude = forward.magnitude();

        if self.is_x_pressed.1 {
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_magnitude;
        }

        if self.is_x_pressed.0 {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_magnitude;
        }
    }
}