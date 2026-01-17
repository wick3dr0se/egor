use egor_render::math::{Mat4, Rect, Vec2};

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
    /// Returns the orthographic view-projection matrix for the current camera state
    pub(crate) fn view_proj(&self, screen_size: Vec2) -> Mat4 {
        let width = screen_size.x / self.zoom;
        let height = screen_size.y / self.zoom;

        let left = self.position.x;
        let right = self.position.x + width;
        let top = self.position.y;
        let bottom = self.position.y + height;

        Mat4::orthographic_lh(left, right, bottom, top, -1.0, 1.0)
    }

    /// Set the camera's position (top-left corner of view)
    pub fn target(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Center the camera on a position
    pub fn center(&mut self, position: Vec2, screen_size: Vec2) {
        self.position = position - screen_size / (2.0 * self.zoom);
    }

    /// Set zoom level, clamped between 0.1 & 10.0 to avoid insanity
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 10.0);
    }

    /// Returns the viewport rectangle in world coordinates, factoring in zoom  
    /// Useful for culling or visibility checks
    pub fn viewport(&self, screen_size: Vec2) -> Rect {
        let size = screen_size / self.zoom;
        Rect::new(self.position, size)
    }
    /// Converts a point from world space to screen space (pixels)
    pub fn world_to_screen(&self, world: Vec2) -> Vec2 {
        (world - self.position) * self.zoom
    }

    /// Converts a point from screen space back to world space
    pub fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        screen / self.zoom + self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egor_render::math::vec2;

    #[test]
    fn view_proj_matrix() {
        // check that the camera's view-projection matrix matches expected ortho math
        let mut cam = Camera::default();
        cam.target(vec2(0.0, 0.0));
        cam.set_zoom(1.0);

        let mat = cam.view_proj(vec2(800.0, 600.0));
        let expected = Mat4::orthographic_lh(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
        assert_eq!(mat, expected);
    }

    #[test]
    fn viewport_rect() {
        // check that viewport is centered on camera & scales correctly with zoom
        let mut cam = Camera::default();
        cam.target(vec2(50.0, 50.0));
        cam.set_zoom(2.0);

        let rect = cam.viewport(vec2(200.0, 100.0));
        // Position is top-left corner, size is screen_size / zoom
        assert_eq!(rect.position, vec2(50.0, 50.0));
        assert!((rect.size - vec2(100.0, 50.0)).length() < 0.001); // allow for float fuzz
    }

    #[test]
    fn world_screen_round_trip() {
        // converting world -> screen -> world should come back to where we started
        let mut cam = Camera::default();
        cam.target(vec2(100.0, 50.0));
        cam.set_zoom(2.0);

        let world = vec2(110.0, 55.0);
        let screen = cam.world_to_screen(world);
        let world2 = cam.screen_to_world(screen);

        assert!((world - world2).length() < 0.001);
    }
}
