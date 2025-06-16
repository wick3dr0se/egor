use glam::Vec2;

use crate::render::math::Rect;

pub struct Camera {
    position: Vec2,
    zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
        }
    }

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
