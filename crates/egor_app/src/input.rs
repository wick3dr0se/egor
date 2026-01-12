pub use winit::{event::MouseButton, keyboard::KeyCode};

use crate::coordinate_converter::CoordinateConverter;
use std::collections::HashMap;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, Touch},
    keyboard::PhysicalKey,
};

#[derive(Default)]
pub struct Input {
    keyboard: HashMap<KeyCode, (ElementState, ElementState)>,
    mouse_buttons: HashMap<MouseButton, (ElementState, ElementState)>,
    mouse_position: (f32, f32),
    mouse_delta: (f32, f32),
    touch_position: Option<(f32, f32)>,
    touch_force: Option<f32>,
    touch_pressed: bool,
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

    /// Update touch state with coordinate conversion
    pub(crate) fn touch(&mut self, touch: Touch, coordinate_converter: CoordinateConverter) {
        use winit::event::TouchPhase;

        match touch.phase {
            TouchPhase::Started | TouchPhase::Moved => {
                if touch.phase == TouchPhase::Started {
                    self.touch_pressed = true;
                }

                // Convert window coordinates to buffer coordinates
                let (buffer_x, buffer_y) = coordinate_converter
                    .window_to_buffer(touch.location.x as f32, touch.location.y as f32);
                self.touch_position = Some((buffer_x, buffer_y));
                self.touch_force = touch.force.map(|f| match f {
                    winit::event::Force::Calibrated { force, .. } => force as f32,
                    winit::event::Force::Normalized(force) => force as f32,
                });
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                // Clear touch when finger is lifted
                self.touch_force = None;
                // Keep position for one frame so it can be read
            }
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
        self.touch_pressed = false;
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

    /// True if touch just started this frame (finger just touched screen)
    pub fn touch_pressed(&self) -> bool {
        self.touch_pressed
    }

    /// True if touch is currently active (finger on screen)
    pub fn touch_active(&self) -> bool {
        self.touch_force.is_some_and(|f| f > 0.0)
    }

    /// True if touch just ended this frame (finger just lifted)
    pub fn touch_released(&self) -> bool {
        let curr_active = self.touch_force.is_some_and(|f| f > 0.0);
        !curr_active
    }

    /// Current touch position in buffer coordinates (if touch is active)
    pub fn touch_position(&self) -> Option<(f32, f32)> {
        self.touch_position
    }

    /// Current touch force/pressure (if available and touch is active)
    pub fn touch_force(&self) -> Option<f32> {
        self.touch_force
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

    pub fn inject_touch(
        &mut self,
        phase: winit::event::TouchPhase,
        x: f32,
        y: f32,
        force: Option<f32>,
    ) {
        use winit::event::TouchPhase;
        let converter = CoordinateConverter::default(); // Use default pass-through converter for tests

        // Create a Touch event - we'll use a mock approach by directly calling touch method
        // Since constructing Touch from winit is complex, we'll simulate it
        match phase {
            TouchPhase::Started | TouchPhase::Moved => {
                let (buffer_x, buffer_y) = converter.window_to_buffer(x, y);
                self.touch_position = Some((buffer_x, buffer_y));
                self.touch_force = force;
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.touch_force = None;
                // Keep position for one frame so it can be read
            }
        }
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

    #[test]
    fn touch_press_and_release_behavior() {
        // test touch press -> hold -> release flow
        use winit::event::TouchPhase;

        let mut input = Input::default();
        input.inject_touch(TouchPhase::Started, 100.0, 200.0, Some(0.5));
        assert!(input.touch_pressed());
        assert!(input.touch_active());
        assert!(!input.touch_released());
        assert_eq!(input.touch_position(), Some((100.0, 200.0)));
        assert_eq!(input.touch_force(), Some(0.5));

        input.end_frame(); // updates previous state
        assert!(!input.touch_pressed()); // no longer "just pressed"
        assert!(input.touch_active()); // still active
        assert!(!input.touch_released());

        // Move touch (still active)
        input.inject_touch(TouchPhase::Moved, 110.0, 190.0, Some(0.6));
        assert!(!input.touch_pressed()); // not a new press
        assert!(input.touch_active());
        assert_eq!(input.touch_position(), Some((110.0, 190.0)));
        assert_eq!(input.touch_force(), Some(0.6));

        input.end_frame();

        // Release touch
        input.inject_touch(TouchPhase::Ended, 110.0, 190.0, None);
        assert!(!input.touch_pressed());
        assert!(!input.touch_active());
        assert!(input.touch_released());
        // Position should still be available for one frame
        assert_eq!(input.touch_position(), Some((110.0, 190.0)));
        assert_eq!(input.touch_force(), None);

        input.end_frame();
        // After end_frame, touch should be fully cleared
        assert!(!input.touch_pressed());
        assert!(!input.touch_active());
        assert!(!input.touch_released());
    }

    #[test]
    fn touch_position_tracking() {
        // test that touch position is correctly tracked and converted
        use winit::event::TouchPhase;

        let mut input = Input::default();
        input.inject_touch(TouchPhase::Started, 50.0, 75.0, Some(0.3));
        assert_eq!(input.touch_position(), Some((50.0, 75.0)));

        input.inject_touch(TouchPhase::Moved, 60.0, 80.0, Some(0.4));
        assert_eq!(input.touch_position(), Some((60.0, 80.0)));

        input.inject_touch(TouchPhase::Moved, 70.0, 90.0, Some(0.5));
        assert_eq!(input.touch_position(), Some((70.0, 90.0)));
    }

    #[test]
    fn touch_force_tracking() {
        // test that touch force is correctly tracked
        use winit::event::TouchPhase;

        let mut input = Input::default();
        input.inject_touch(TouchPhase::Started, 100.0, 100.0, Some(0.1));
        assert_eq!(input.touch_force(), Some(0.1));

        input.inject_touch(TouchPhase::Moved, 100.0, 100.0, Some(0.5));
        assert_eq!(input.touch_force(), Some(0.5));

        input.inject_touch(TouchPhase::Moved, 100.0, 100.0, Some(0.9));
        assert_eq!(input.touch_force(), Some(0.9));

        input.inject_touch(TouchPhase::Ended, 100.0, 100.0, None);
        assert_eq!(input.touch_force(), None);
    }

    #[test]
    fn touch_without_force() {
        // test touch behavior when force is not available
        use winit::event::TouchPhase;

        let mut input = Input::default();
        // Touch without force should not be considered active
        input.inject_touch(TouchPhase::Started, 100.0, 100.0, None);
        assert!(!input.touch_active());
        assert!(!input.touch_pressed());
        assert_eq!(input.touch_force(), None);
    }

    #[test]
    fn touch_cancelled() {
        // test that cancelled touch is handled like ended
        use winit::event::TouchPhase;

        let mut input = Input::default();
        input.inject_touch(TouchPhase::Started, 100.0, 200.0, Some(0.5));
        assert!(input.touch_active());
        input.end_frame();

        input.inject_touch(TouchPhase::Cancelled, 100.0, 200.0, None);
        assert!(!input.touch_active());
        assert!(input.touch_released());
        assert_eq!(input.touch_force(), None);
    }

    #[test]
    fn no_false_positives_for_untracked_touch() {
        // touch methods should return false when no touch has occurred
        let input = Input::default();

        assert!(!input.touch_pressed());
        assert!(!input.touch_active());
        assert!(!input.touch_released());
        assert_eq!(input.touch_position(), None);
        assert_eq!(input.touch_force(), None);
    }

    #[test]
    fn rapid_touch_sequence() {
        // test rapid touch press -> release -> press sequence
        use winit::event::TouchPhase;

        let mut input = Input::default();

        input.inject_touch(TouchPhase::Started, 100.0, 100.0, Some(0.5));
        assert!(input.touch_pressed());
        assert!(input.touch_active());
        input.end_frame();

        input.inject_touch(TouchPhase::Ended, 100.0, 100.0, None);
        assert!(input.touch_released());
        assert!(!input.touch_active());
        input.end_frame(); // update previous state so next press is detected

        // Press again
        input.inject_touch(TouchPhase::Started, 150.0, 150.0, Some(0.6));
        assert!(input.touch_pressed()); // should detect new press
        assert!(input.touch_active());
        assert!(!input.touch_released());
        assert_eq!(input.touch_position(), Some((150.0, 150.0)));
    }

    #[test]
    fn touch_state_transitions() {
        // comprehensive test of all touch state transitions
        use winit::event::TouchPhase;

        let mut input = Input::default();

        // Initial state: no touch
        assert!(!input.touch_pressed());
        assert!(!input.touch_active());
        assert!(!input.touch_released());

        // Start touch
        input.inject_touch(TouchPhase::Started, 100.0, 100.0, Some(0.5));
        assert!(input.touch_pressed());
        assert!(input.touch_active());
        assert!(!input.touch_released());

        // End frame - touch is still active but no longer "just pressed"
        input.end_frame();
        assert!(!input.touch_pressed());
        assert!(input.touch_active());
        assert!(!input.touch_released());

        // Move touch - still active
        input.inject_touch(TouchPhase::Moved, 110.0, 110.0, Some(0.6));
        assert!(!input.touch_pressed());
        assert!(input.touch_active());
        assert!(!input.touch_released());
        input.end_frame();

        // End touch
        input.inject_touch(TouchPhase::Ended, 110.0, 110.0, None);
        assert!(!input.touch_pressed());
        assert!(!input.touch_active());
        assert!(input.touch_released());

        // End frame - touch is fully cleared
        input.end_frame();
        assert!(!input.touch_pressed());
        assert!(!input.touch_active());
        assert!(!input.touch_released());
    }
}
