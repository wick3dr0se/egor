// src/camera.rs
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    x: f32,
    y: f32,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        }
    }

    pub fn target(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn world_to_screen(
        &self,
        world_x: f32,
        world_y: f32,
        screen_width: f32,
        screen_height: f32,
    ) -> (f32, f32) {
        let scale = self.zoom;
        let screen_x = (world_x - self.x) * scale + screen_width * 0.5;
        let screen_y = (world_y - self.y) * scale + screen_height * 0.5;
        (screen_x, screen_y)
    }

    pub fn screen_to_world(
        &self,
        screen_x: f32,
        screen_y: f32,
        screen_width: f32,
        screen_height: f32,
    ) -> (f32, f32) {
        let inv_scale = 1.0 / self.zoom;
        let world_x = (screen_x - screen_width * 0.5) * inv_scale + self.x;
        let world_y = (screen_y - screen_height * 0.5) * inv_scale + self.y;
        (world_x, world_y)
    }
}
