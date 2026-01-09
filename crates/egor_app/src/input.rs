pub use winit::{event::MouseButton, keyboard::KeyCode};

use std::collections::HashMap;

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent},
    keyboard::PhysicalKey,
};

#[derive(Default)]
pub struct Input {
    keyboard: HashMap<KeyCode, (ElementState, ElementState)>, // (current, previous) state
    mouse_buttons: HashMap<MouseButton, (ElementState, ElementState)>,
    mouse_position: (f32, f32),
    mouse_delta: (f32, f32),
    focused: bool,
    touches: HashMap<u64, (f32, f32)>, // Track active touches by id -> (x, y)
    touch_start_positions: HashMap<u64, (f32, f32)>, // Track where each touch started
    swipe_left: bool,  // Swipe left detected this frame
    swipe_right: bool, // Swipe right detected this frame
}

impl Input {
    /// Update keyboard state from a `winit` KeyEvent
    pub(crate) fn keyboard(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            let prev = self
                .keyboard
                .get(&key_code)
                .map_or(ElementState::Released, |(curr, _)| *curr);
            self.keyboard.insert(key_code, (event.state, prev));
        }
    }

    /// Update mouse button state
    pub(crate) fn mouse(&mut self, button: MouseButton, state: ElementState) {
        let prev = self
            .mouse_buttons
            .get(&button)
            .map_or(ElementState::Released, |(curr, _)| *curr);
        self.mouse_buttons.insert(button, (state, prev));
    }

    /// Update cursor position & compute delta
    pub(crate) fn cursor(&mut self, position: PhysicalPosition<f64>) {
        let prev_pos = self.mouse_position;
        let pos: (f32, f32) = position.into();
        self.mouse_delta = (pos.0 - prev_pos.0, pos.1 - prev_pos.1);
        self.mouse_position = pos;
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
        
        // Reset swipe flags each frame
        self.swipe_left = false;
        self.swipe_right = false;
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

    pub(crate) fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn has_focus(&self) -> bool {
        self.focused
    }

    pub(crate) fn touch(&mut self, touch: winit::event::Touch) {
        use winit::event::TouchPhase;
        
        let (x, y) = (touch.location.x as f32, touch.location.y as f32);
        
        match touch.phase {
            TouchPhase::Started => {
                // Store touch start position for swipe detection
                self.touch_start_positions.insert(touch.id, (x, y));
                self.touches.insert(touch.id, (x, y));
                
                // Convert touch start to mouse press for compatibility
                let prev_pos = self.mouse_position;
                self.mouse_position = (x, y);
                self.mouse_delta = (x - prev_pos.0, y - prev_pos.1);
                
                // Simulate mouse button press (new press this frame)
                let prev = self
                    .mouse_buttons
                    .get(&MouseButton::Left)
                    .map_or(ElementState::Released, |(curr, _)| *curr);
                self.mouse_buttons.insert(MouseButton::Left, (ElementState::Pressed, prev));
            }
            TouchPhase::Moved => {
                // Update touch position
                if self.touches.contains_key(&touch.id) {
                    let prev_pos = self.mouse_position;
                    self.mouse_position = (x, y);
                    self.mouse_delta = (x - prev_pos.0, y - prev_pos.1);
                    self.touches.insert(touch.id, (x, y));
                    
                    // Keep mouse button pressed during move (update current state, keep previous)
                    if let Some((curr, prev)) = self.mouse_buttons.get_mut(&MouseButton::Left) {
                        *prev = *curr;
                        *curr = ElementState::Pressed;
                    } else {
                        // If somehow not tracked, add it
                        self.mouse_buttons.insert(MouseButton::Left, (ElementState::Pressed, ElementState::Released));
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                // Check if this was a swipe (movement > threshold)
                if let Some(start_pos) = self.touch_start_positions.remove(&touch.id) {
                    let dx = x - start_pos.0;
                    let dy = y - start_pos.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    // Swipe detection constants
                    const SWIPE_THRESHOLD: f32 = 50.0; // Minimum distance in pixels
                    const SWIPE_HORIZONTAL_RATIO: f32 = 0.6; // Horizontal movement must be at least 60% of total
                    
                    // Check if this qualifies as a swipe
                    if distance >= SWIPE_THRESHOLD {
                        let abs_dx = dx.abs();
                        
                        // Check if horizontal movement is dominant (prevents diagonal swipes)
                        if abs_dx >= distance * SWIPE_HORIZONTAL_RATIO {
                            // Determine swipe direction
                            if dx < 0.0 {
                                // Swipe left (negative x movement)
                                self.swipe_left = true;
                            } else {
                                // Swipe right (positive x movement)
                                self.swipe_right = true;
                            }
                        }
                    }
                }
                
                self.touches.remove(&touch.id);
                
                // Simulate mouse button release
                let prev = self
                    .mouse_buttons
                    .get(&MouseButton::Left)
                    .map_or(ElementState::Released, |(curr, _)| *curr);
                self.mouse_buttons.insert(MouseButton::Left, (ElementState::Released, prev));
            }
        }
    }

    /// Get the number of active touches
    pub fn touch_count(&self) -> usize {
        self.touches.len()
    }

    /// Get primary touch position (first active touch, or mouse position if no touches)
    pub fn primary_touch_position(&self) -> (f32, f32) {
        self.touches.values().next()
            .copied()
            .unwrap_or(self.mouse_position)
    }

    /// True if a swipe left gesture was detected this frame
    pub fn swipe_left(&self) -> bool {
        self.swipe_left
    }

    /// True if a swipe right gesture was detected this frame
    pub fn swipe_right(&self) -> bool {
        self.swipe_right
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
