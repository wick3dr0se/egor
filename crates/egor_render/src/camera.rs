use glam::{Mat4, Vec2};

use crate::math::Rect;

/// A basic camera for controlling view & projection
///
/// Useful for culling & rendering transformations
pub struct Camera {
    position: Vec2,
    zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Camera {
    /// Set the camera's target position (center of view)
    pub fn target(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Set zoom level, clamped between 0.1 & 10.0 to avoid insanity
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 10.0);
    }

    /// Returns the viewport rectangle in world coordinates, factoring in zoom  
    /// Useful for culling or visibility checks
    pub fn viewport(&self, screen_size: Vec2) -> Rect {
        Rect::new(self.position, screen_size / self.zoom)
    }

    /// Converts a point from world space to screen space (pixels)
    pub fn world_to_screen(&self, world: Vec2, screen_size: Vec2) -> Vec2 {
        (world - self.position) * self.zoom + (screen_size / 2.0)
    }

    /// Converts a point from screen space back to world space
    pub fn screen_to_world(&self, screen: Vec2, screen_size: Vec2) -> Vec2 {
        (screen - screen_size / 2.0) / self.zoom + self.position
    }
}

/// Provides the view-projection matrix for GPU transforms  
/// Not needed by typical `egor` users; mainly for `egor_render` or advanced cases
pub trait CameraInternal {
    fn view_proj(&self, screen_size: Vec2) -> Mat4;
}

impl CameraInternal for Camera {
    /// Returns the orthographic view-projection matrix for the current camera state
    fn view_proj(&self, screen_size: Vec2) -> Mat4 {
        let half_width = screen_size.x / 2.0 / self.zoom;
        let half_height = screen_size.y / 2.0 / self.zoom;

        let left = self.position.x - half_width;
        let right = self.position.x + half_width;
        let bottom = self.position.y - half_height;
        let top = self.position.y + half_height;

        Mat4::orthographic_lh(left, right, top, bottom, -1.0, 1.0)
    }
}
