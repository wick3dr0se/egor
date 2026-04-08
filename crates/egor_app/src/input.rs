pub use winit::{event::MouseButton, event::Touch, event::TouchPhase, keyboard::KeyCode};

use std::collections::HashMap;

use winit::{
    dpi::PhysicalPosition,
    event::{DeviceId, ElementState, KeyEvent},
    keyboard::PhysicalKey,
};

pub struct Input {
    keyboard: HashMap<KeyCode, (ElementState, ElementState)>, // (current, previous) state
    mouse_buttons: HashMap<MouseButton, (ElementState, ElementState)>,
    mouse_position: (f32, f32),
    mouse_delta: (f32, f32),
    mouse_wheel_delta: f32,
    touches: HashMap<u64, Touch>,
    simulate_touch_with_mouse: bool,
    simulate_mouse_with_touch: bool,
    /// Tracks which touch id is acting as the primary mouse (for touch→mouse simulation)
    primary_touch_id: Option<u64>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            keyboard: HashMap::default(),
            mouse_buttons: HashMap::default(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            mouse_wheel_delta: 0.0,
            touches: HashMap::default(),
            simulate_touch_with_mouse: false,
            simulate_mouse_with_touch: false,
            primary_touch_id: None,
        }
    }
}

impl Input {
    /// Enable or disable simulating touch events from mouse input.
    /// When enabled, left mouse button presses/moves/releases generate touch events with id 0.
    /// Useful for testing touch logic on desktop.
    pub fn set_simulate_touch_with_mouse(&mut self, enabled: bool) {
        self.simulate_touch_with_mouse = enabled;
    }

    /// Enable or disable simulating mouse events from touch input.
    /// When enabled, the first active touch generates mouse position, delta, and left-button events.
    /// Useful on mobile to make existing mouse-based code work with touch.
    pub fn set_simulate_mouse_with_touch(&mut self, enabled: bool) {
        self.simulate_mouse_with_touch = enabled;
    }

    /// Returns whether touch-from-mouse simulation is enabled
    pub fn simulate_touch_with_mouse(&self) -> bool {
        self.simulate_touch_with_mouse
    }

    /// Returns whether mouse-from-touch simulation is enabled
    pub fn simulate_mouse_with_touch(&self) -> bool {
        self.simulate_mouse_with_touch
    }

    /// Update keyboard state from a `winit` KeyEvent
    pub(crate) fn update_key(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            let prev = self
                .keyboard
                .get(&key_code)
                .map_or(ElementState::Released, |(curr, _)| *curr);
            self.keyboard.insert(key_code, (event.state, prev));
        }
    }

    /// Update mouse button state
    pub(crate) fn update_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let prev = self
            .mouse_buttons
            .get(&button)
            .map_or(ElementState::Released, |(curr, _)| *curr);
        self.mouse_buttons.insert(button, (state, prev));
    }

    /// Update cursor position & compute delta
    pub(crate) fn update_cursor(&mut self, position: PhysicalPosition<f64>) {
        let prev_pos = self.mouse_position;
        let pos: (f32, f32) = position.into();
        self.mouse_delta = (pos.0 - prev_pos.0, pos.1 - prev_pos.1);
        self.mouse_position = pos;
    }

    /// Update mouse wheel delta
    pub(crate) fn update_scroll(&mut self, delta: f32) {
        self.mouse_wheel_delta += delta;
    }

    /// Update touch state from a winit Touch event
    pub(crate) fn update_touch(&mut self, touch: Touch) {
        let id = touch.id;
        let phase = touch.phase;
        let location = touch.location;

        self.touches.insert(id, touch);

        if self.simulate_mouse_with_touch {
            match phase {
                TouchPhase::Started => {
                    if self.primary_touch_id.is_none() {
                        self.primary_touch_id = Some(id);
                        self.update_cursor(location);
                        self.update_mouse_button(MouseButton::Left, ElementState::Pressed);
                    }
                }
                TouchPhase::Moved => {
                    if self.primary_touch_id == Some(id) {
                        self.update_cursor(location);
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if self.primary_touch_id == Some(id) {
                        self.update_cursor(location);
                        self.update_mouse_button(MouseButton::Left, ElementState::Released);
                        self.primary_touch_id = None;
                    }
                }
            }
        }
    }

    /// Simulate a touch event from mouse input (called internally when simulation is enabled)
    pub(crate) fn simulate_touch_from_mouse(&mut self, button: MouseButton, state: ElementState) {
        if !self.simulate_touch_with_mouse || button != MouseButton::Left {
            return;
        }
        let pos = self.mouse_position;
        let phase = match state {
            ElementState::Pressed => TouchPhase::Started,
            ElementState::Released => TouchPhase::Ended,
        };
        // Use id 0 for mouse-simulated touch
        self.touches.insert(
            0,
            Touch {
                device_id: DeviceId::dummy(),
                id: 0,
                phase,
                location: PhysicalPosition::new(pos.0 as f64, pos.1 as f64),
                force: None,
            },
        );
    }

    /// Simulate a touch move from mouse cursor movement (called internally when simulation is enabled)
    pub(crate) fn simulate_touch_move_from_mouse(&mut self) {
        if !self.simulate_touch_with_mouse {
            return;
        }
        // Only generate a move if the simulated touch is already active
        if let Some(touch) = self.touches.get(&0)
            && matches!(touch.phase, TouchPhase::Started | TouchPhase::Moved)
        {
            let pos = self.mouse_position;
            self.touches.insert(
                0,
                Touch {
                    device_id: DeviceId::dummy(),
                    id: 0,
                    phase: TouchPhase::Moved,
                    location: PhysicalPosition::new(pos.0 as f64, pos.1 as f64),
                    force: None,
                },
            );
        }
    }

    /// Update previous states & clean up released keys/buttons
    pub(crate) fn end_frame(&mut self) {
        for (curr, prev) in self.keyboard.values_mut() {
            *prev = *curr;
        }
        for (curr, prev) in self.mouse_buttons.values_mut() {
            *prev = *curr;
        }

        // Drop released keys/buttons to avoid buildup
        self.keyboard
            .retain(|_, (curr, _)| *curr != ElementState::Released);
        self.mouse_buttons
            .retain(|_, (curr, _)| *curr != ElementState::Released);

        self.mouse_delta = (0.0, 0.0);
        self.mouse_wheel_delta = 0.0;

        // Remove ended/cancelled touches
        self.touches
            .retain(|_, touch| !matches!(touch.phase, TouchPhase::Ended | TouchPhase::Cancelled));
    }

    /// True if the key went from not pressed last frame to pressed this frame
    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keyboard.get(&key).is_some_and(|(curr, prev)| {
            *curr == ElementState::Pressed && *prev != ElementState::Pressed
        })
    }

    /// True if key is held down (pressed now regardless of last frame)
    pub fn key_held(&self, key: KeyCode) -> bool {
        self.keyboard
            .get(&key)
            .is_some_and(|(curr, _)| *curr == ElementState::Pressed)
    }

    /// True if key was just released this frame
    pub fn key_released(&self, key: KeyCode) -> bool {
        self.keyboard
            .get(&key)
            .is_some_and(|(curr, _)| *curr == ElementState::Released)
    }

    /// True if any key in slice was just pressed
    pub fn keys_pressed(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|&key| self.key_pressed(key))
    }

    /// True if any key in slice is held
    pub fn keys_held(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|&key| self.key_held(key))
    }

    /// True if any key in slice was released
    pub fn keys_released(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|&key| self.key_released(key))
    }

    /// True if all keys in slice were just pressed
    pub fn all_keys_pressed(&self, keys: &[KeyCode]) -> bool {
        keys.iter().all(|&key| self.key_pressed(key))
    }

    /// True if all keys in slice are held
    pub fn all_keys_held(&self, keys: &[KeyCode]) -> bool {
        keys.iter().all(|&key| self.key_held(key))
    }

    /// True if all keys in slice were released this frame
    pub fn all_keys_released(&self, keys: &[KeyCode]) -> bool {
        keys.iter().all(|&key| self.key_released(key))
    }

    /// True if mouse button was just pressed this frame
    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.get(&button).is_some_and(|(curr, prev)| {
            *curr == ElementState::Pressed && *prev != ElementState::Pressed
        })
    }

    /// True if mouse button is held down
    pub fn mouse_held(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .get(&button)
            .is_some_and(|(curr, _)| *curr == ElementState::Pressed)
    }

    /// True if mouse button was released this frame
    pub fn mouse_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .get(&button)
            .is_some_and(|(curr, _)| *curr == ElementState::Released)
    }

    /// Current mouse cursor position in window coords
    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    /// Delta mouse movement since last frame
    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    /// Mouse wheel delta this frame (positive = scroll up, negative = scroll down)
    pub fn mouse_scroll(&self) -> f32 {
        self.mouse_wheel_delta
    }

    /// Returns all active touches this frame
    pub fn touches(&self) -> Vec<Touch> {
        self.touches.values().copied().collect()
    }

    /// Get a specific touch by its id, if it exists
    pub fn touch(&self, id: u64) -> Option<Touch> {
        self.touches.get(&id).copied()
    }

    /// Returns true if any touch is active (finger on screen)
    pub fn is_touched(&self) -> bool {
        !self.touches.is_empty()
    }

    /// Returns the number of active touches
    pub fn touch_count(&self) -> usize {
        self.touches.len()
    }
}

#[cfg(test)]
impl Input {
    pub fn inject_key(&mut self, key: KeyCode, state: ElementState) {
        let prev = self
            .keyboard
            .get(&key)
            .map_or(ElementState::Released, |(curr, _)| *curr);
        self.keyboard.insert(key, (state, prev));
    }

    pub fn inject_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let prev = self
            .mouse_buttons
            .get(&button)
            .map_or(ElementState::Released, |(curr, _)| *curr);
        self.mouse_buttons.insert(button, (state, prev));
    }

    pub fn inject_cursor(&mut self, x: f32, y: f32) {
        let prev = self.mouse_position;
        self.mouse_position = (x, y);
        self.mouse_delta = (x - prev.0, y - prev.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::{
        event::ElementState::{Pressed, Released},
        event::MouseButton,
        keyboard::KeyCode,
    };

    #[test]
    fn key_press_and_release_behavior() {
        // test key press -> hold -> release flow
        let mut input = Input::default();
        input.inject_key(KeyCode::Space, Pressed);
        assert!(input.key_pressed(KeyCode::Space));
        assert!(input.key_held(KeyCode::Space));
        assert!(!input.key_released(KeyCode::Space));

        input.end_frame(); // clears pressed flag
        assert!(!input.key_pressed(KeyCode::Space));
        assert!(input.key_held(KeyCode::Space));

        input.inject_key(KeyCode::Space, Released);
        assert!(input.key_released(KeyCode::Space));
        assert!(!input.key_held(KeyCode::Space));

        input.end_frame(); // drops released key from map
        assert!(!input.key_held(KeyCode::Space));
        assert!(!input.key_released(KeyCode::Space));
    }

    #[test]
    fn mouse_button_and_cursor() {
        // test mouse press & cursor movement/delta
        let mut input = Input::default();
        input.inject_mouse_button(MouseButton::Left, Pressed);
        assert!(input.mouse_pressed(MouseButton::Left));
        assert!(input.mouse_held(MouseButton::Left));
        assert!(!input.mouse_released(MouseButton::Left));

        input.inject_cursor(100.0, 200.0);
        assert_eq!(input.mouse_position(), (100.0, 200.0));
        assert_eq!(input.mouse_delta(), (100.0, 200.0)); // moved from (0, 0)

        input.inject_cursor(110.0, 190.0);
        assert_eq!(input.mouse_position(), (110.0, 190.0));
        assert_eq!(input.mouse_delta(), (10.0, -10.0));

        input.end_frame(); // delta should reset
        assert_eq!(input.mouse_delta(), (0.0, 0.0));
    }

    #[test]
    fn end_frame_cleans_released_keys_and_resets_mouse_delta() {
        // confirms end_frame clears out released input & resets delta
        let mut input = Input::default();

        input.inject_key(KeyCode::KeyA, Pressed);
        input.inject_key(KeyCode::KeyB, Released);
        input.inject_mouse_button(MouseButton::Right, Released);
        input.inject_cursor(50.0, 75.0);

        input.end_frame();

        assert!(input.key_held(KeyCode::KeyA));
        assert!(!input.key_held(KeyCode::KeyB));
        assert!(!input.mouse_held(MouseButton::Right));
        assert_eq!(input.mouse_delta(), (0.0, 0.0));
    }

    #[test]
    fn multiple_keys_and_buttons() {
        // tests helpers that check multiple keys/buttons at once
        let mut input = Input::default();

        input.inject_key(KeyCode::KeyA, Pressed);
        input.inject_key(KeyCode::KeyB, Pressed);
        input.inject_mouse_button(MouseButton::Left, Pressed);
        input.inject_mouse_button(MouseButton::Right, Released);

        assert!(input.keys_pressed(&[KeyCode::KeyA, KeyCode::KeyX]));
        assert!(input.all_keys_pressed(&[KeyCode::KeyA, KeyCode::KeyB]));
        assert!(!input.all_keys_pressed(&[KeyCode::KeyA, KeyCode::KeyX]));

        assert!(input.mouse_pressed(MouseButton::Left));
        assert!(!input.mouse_pressed(MouseButton::Right));
    }

    #[test]
    fn no_false_positives_for_untracked_keys_and_buttons() {
        // keys/buttons not touched shouldn't be considered active
        let input = Input::default();

        assert!(!input.key_pressed(KeyCode::KeyZ));
        assert!(!input.key_held(KeyCode::KeyZ));
        assert!(!input.key_released(KeyCode::KeyZ));

        assert!(!input.mouse_pressed(MouseButton::Middle));
        assert!(!input.mouse_held(MouseButton::Middle));
        assert!(!input.mouse_released(MouseButton::Middle));
    }

    #[test]
    fn rapid_press_release_press_sequence() {
        // catch edge case where a key is released & pressed again in same frame
        let mut input = Input::default();

        input.inject_key(KeyCode::KeyX, Pressed);
        assert!(input.key_pressed(KeyCode::KeyX));
        input.end_frame();

        input.inject_key(KeyCode::KeyX, Released);
        assert!(input.key_released(KeyCode::KeyX));

        input.inject_key(KeyCode::KeyX, Pressed); // re-press in same frame
        assert!(input.key_pressed(KeyCode::KeyX));
        assert!(input.key_held(KeyCode::KeyX));
        assert!(!input.key_released(KeyCode::KeyX));
    }
}
