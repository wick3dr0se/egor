use std::collections::HashMap;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::PhysicalKey,
};

pub use winit::keyboard::KeyCode;

#[derive(Default)]
pub struct Input {
    keyboard: HashMap<KeyCode, (ElementState, ElementState)>,
    mouse_buttons: HashMap<MouseButton, (ElementState, ElementState)>,
    mouse_position: (f32, f32),
    mouse_delta: (f32, f32),
}

impl Input {
    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keyboard.get(&key).map_or(false, |(curr, prev)| {
            *curr == ElementState::Pressed && *prev != ElementState::Pressed
        })
    }

    pub fn key_held(&self, key: KeyCode) -> bool {
        self.keyboard
            .get(&key)
            .map_or(false, |(curr, _)| *curr == ElementState::Pressed)
    }

    pub fn key_released(&self, key: KeyCode) -> bool {
        self.keyboard
            .get(&key)
            .map_or(false, |(curr, _)| *curr == ElementState::Released)
    }

    pub fn keys_pressed(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|&key| self.key_pressed(key))
    }

    pub fn keys_held(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|&key| self.key_held(key))
    }

    pub fn keys_released(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|&key| self.key_released(key))
    }

    pub fn all_keys_pressed(&self, keys: &[KeyCode]) -> bool {
        keys.iter().all(|&key| self.key_pressed(key))
    }

    pub fn all_keys_held(&self, keys: &[KeyCode]) -> bool {
        keys.iter().all(|&key| self.key_held(key))
    }

    pub fn all_keys_released(&self, keys: &[KeyCode]) -> bool {
        keys.iter().all(|&key| self.key_released(key))
    }

    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .get(&button)
            .map_or(false, |(curr, prev)| {
                *curr == ElementState::Pressed && *prev != ElementState::Pressed
            })
    }

    pub fn mouse_held(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .get(&button)
            .map_or(false, |(curr, _)| *curr == ElementState::Pressed)
    }

    pub fn mouse_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .get(&button)
            .map_or(false, |(curr, _)| *curr == ElementState::Released)
    }

    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    pub(crate) fn keyboard(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            let prev = self
                .keyboard
                .get(&key_code)
                .map_or(ElementState::Released, |(curr, _)| *curr);
            self.keyboard.insert(key_code, (event.state, prev));
        }
    }

    pub(crate) fn mouse(&mut self, button: MouseButton, state: ElementState) {
        let prev = self
            .mouse_buttons
            .get(&button)
            .map_or(ElementState::Released, |(curr, _)| *curr);
        self.mouse_buttons.insert(button, (state, prev));
    }

    pub(crate) fn cursor(&mut self, position: PhysicalPosition<f64>) {
        let prev_pos = self.mouse_position;
        let pos: (f32, f32) = position.into();
        self.mouse_delta = (pos.0 - prev_pos.0, pos.1 - prev_pos.1);
        self.mouse_position = pos;
    }

    pub(crate) fn end_frame(&mut self) {
        for (curr, prev) in self.keyboard.values_mut() {
            *prev = *curr;
        }
        for (curr, prev) in self.mouse_buttons.values_mut() {
            *prev = *curr;
        }

        self.keyboard
            .retain(|_, (curr, _)| *curr != ElementState::Released);
        self.mouse_buttons
            .retain(|_, (curr, _)| *curr != ElementState::Released);
        self.mouse_delta = (0.0, 0.0);
    }
}
