use glam::{Vec2, vec2};

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

    pub fn viewport(&self, screen_w: f32, screen_h: f32) -> (Vec2, Vec2) {
        let half_extents = vec2(screen_w, screen_h) * 0.5 / self.zoom;
        let min = self.position - half_extents;
        let max = self.position + half_extents;
        (min, max)
    }

    pub fn world_to_screen(&self, world: Vec2, screen_w: f32, screen_h: f32) -> Vec2 {
        let screen_center = vec2(screen_w, screen_h) * 0.5;
        (world - self.position) * self.zoom + screen_center
    }

    pub fn screen_to_world(&self, screen: Vec2, screen_w: f32, screen_h: f32) -> Vec2 {
        let screen_center = vec2(screen_w, screen_h) * 0.5;
        (screen - screen_center) / self.zoom + self.position
    }
}
