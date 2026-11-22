use glam::{Mat2, Mat4, Vec2, vec2};

use crate::math::Rect;

/// A basic camera for controlling view & projection
///
/// Useful for culling & rendering transformations
#[derive(Clone, Copy)]
pub struct Camera {
    position: Vec2,
    zoom: f32,
    rotation: f32,
    /// Framebuffer size, needed to rotate around the view center
    view_size: Vec2,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
            view_size: Vec2::ZERO,
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

        let ortho = Mat4::orthographic_lh(left, right, bottom, top, -1.0, 1.0);
        if self.rotation == 0.0 {
            return ortho;
        }

        // rotate around the view center so `center` still points at the same world position
        let center = self.position + vec2(width, height) * 0.5;
        let spin = Mat4::from_translation(center.extend(0.0))
            * Mat4::from_rotation_z(self.rotation)
            * Mat4::from_translation(-center.extend(0.0));
        ortho * spin
    }

    /// Set the view rotation in radians, around the view center
    pub fn set_rotation(&mut self, radians: f32) {
        self.rotation = radians;
    }

    /// Current view rotation in radians
    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    /// Tell the camera the framebuffer size, `Graphics` does this each frame
    pub(crate) fn set_view_size(&mut self, size: Vec2) {
        self.view_size = size;
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

    /// Current zoom level
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Returns the viewport rectangle in world coordinates, factoring in zoom
    ///
    /// With rotation this is the bounding box of the turned view, the only part that
    /// stays axis aligned. Useful for culling or visibility checks
    pub fn viewport(&self, screen_size: Vec2) -> Rect {
        let size = screen_size / self.zoom;
        if self.rotation == 0.0 {
            return Rect::new(self.position, size);
        }

        // only the bounding box of a turned view stays axis aligned
        let center = self.position + size * 0.5;
        let spin = Mat2::from_angle(self.rotation);
        let mut min = Vec2::splat(f32::MAX);
        let mut max = Vec2::splat(f32::MIN);
        for corner in [
            self.position,
            self.position + vec2(size.x, 0.0),
            self.position + size,
            self.position + vec2(0.0, size.y),
        ] {
            let turned = spin * (corner - center) + center;
            min = min.min(turned);
            max = max.max(turned);
        }
        Rect::new(min, max - min)
    }
    /// Converts a point from world space to screen space (pixels)
    pub fn world_to_screen(&self, world: Vec2) -> Vec2 {
        (self.turn(world) - self.position) * self.zoom
    }

    /// Converts a point from screen space back to world space
    pub fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        self.unturn(screen / self.zoom + self.position)
    }

    /// Rotate a world point around the view center
    fn turn(&self, world: Vec2) -> Vec2 {
        if self.rotation == 0.0 {
            return world;
        }
        let center = self.position + self.view_half();
        Mat2::from_angle(self.rotation) * (world - center) + center
    }

    /// Rotate a world point back, the inverse of [`Self::turn`]
    fn unturn(&self, world: Vec2) -> Vec2 {
        if self.rotation == 0.0 {
            return world;
        }
        let center = self.position + self.view_half();
        Mat2::from_angle(-self.rotation) * (world - center) + center
    }

    /// Half the view size in world units
    fn view_half(&self) -> Vec2 {
        self.view_size / (2.0 * self.zoom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec2;

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
    fn view_proj_agrees_with_world_to_screen() {
        // the GPU matrix and the CPU projection must describe the same camera,
        // otherwise shapes and text drift apart (or vanish) once the view turns
        let size = vec2(800.0, 600.0);
        for rotation in [0.0, 0.7, -1.2] {
            let mut cam = Camera::default();
            cam.set_view_size(size);
            cam.set_zoom(1.5);
            cam.target(vec2(-40.0, 25.0));
            cam.set_rotation(rotation);

            let matrix = cam.view_proj(size);
            for world in [vec2(-40.0, 25.0), vec2(120.0, 90.0), vec2(-10.0, 200.0)] {
                let clip = matrix * world.extend(0.0).extend(1.0);
                let ndc = clip.truncate().truncate() / clip.w;

                let screen = cam.world_to_screen(world);
                let expected = vec2(screen.x / size.x * 2.0 - 1.0, 1.0 - screen.y / size.y * 2.0);
                assert!(
                    (ndc - expected).length() < 0.001,
                    "rotation {rotation}: {ndc:?} != {expected:?}"
                );
            }
        }
    }

    #[test]
    fn rotation_keeps_the_center_fixed() {
        let mut cam = Camera::default();
        cam.set_view_size(vec2(800.0, 600.0));
        cam.set_rotation(std::f32::consts::FRAC_PI_2);

        // the center of the view is the one world point that should not move
        let center = cam.position + cam.view_half();
        let screen = cam.world_to_screen(center);
        assert!((screen - vec2(400.0, 300.0)).length() < 0.001);
        assert!((cam.screen_to_world(screen) - center).length() < 0.001);
    }

    #[test]
    fn turned_viewport_is_the_bounding_box() {
        let mut cam = Camera::default();
        cam.set_view_size(vec2(800.0, 600.0));
        cam.set_rotation(std::f32::consts::FRAC_PI_4);

        let rect = cam.viewport(vec2(800.0, 600.0));
        // a square view turned 90 degrees keeps its size, 45 degrees grows the box
        assert!(rect.size.x > 800.0 && rect.size.y > 600.0);
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

    #[test]
    fn world_screen_round_trip_at_any_view_size() {
        // the view size is logical, so it varies with the display scale
        let mut cam = Camera::default();
        cam.set_view_size(vec2(456.0, 508.0));
        cam.target(vec2(100.0, 50.0));

        let world = vec2(140.0, 80.0);
        let screen = cam.world_to_screen(world);
        let world2 = cam.screen_to_world(screen);

        assert!((world - world2).length() < 0.001);
    }

    #[test]
    fn world_screen_round_trip_while_turned() {
        // aiming relies on this: a cursor position must come back as the same world point
        let mut cam = Camera::default();
        cam.set_view_size(vec2(800.0, 600.0));
        cam.target(vec2(100.0, 50.0));
        cam.set_zoom(2.0);
        cam.set_rotation(0.7);

        for screen in [vec2(0.0, 0.0), vec2(400.0, 300.0), vec2(799.0, 12.0)] {
            let world = cam.screen_to_world(screen);
            let back = cam.world_to_screen(world);
            assert!((screen - back).length() < 0.001, "{screen:?} -> {back:?}");
        }
    }
}
