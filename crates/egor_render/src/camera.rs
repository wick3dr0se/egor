use glam::{Mat4, Vec2};

use crate::math::Rect;

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
    pub fn target(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 10.0);
    }

    pub fn viewport(&self, screen_size: Vec2) -> Rect {
        Rect::new(self.position, screen_size / self.zoom)
    }

    pub fn world_to_screen(&self, world: Vec2, screen_size: Vec2) -> Vec2 {
        (world - self.position) * self.zoom + (screen_size / 2.0)
    }

    pub fn screen_to_world(&self, screen: Vec2, screen_size: Vec2) -> Vec2 {
        (screen - screen_size / 2.0) / self.zoom + self.position
    }
}

pub trait CameraInternal {
    fn view_proj(&self, screen_size: Vec2) -> Mat4;
}

impl CameraInternal for Camera {
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
