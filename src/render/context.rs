use wgpu::Color;

use super::{
    Renderer,
    primitives::{RectangleBuilder, TriangleBuilder},
};

pub struct GraphicsContext<'a> {
    pub(crate) renderer: &'a mut Renderer,
}

impl<'a> GraphicsContext<'a> {
    pub fn tri(&mut self) -> TriangleBuilder {
        TriangleBuilder::new(self.renderer)
    }

    pub fn rect(&mut self) -> RectangleBuilder {
        RectangleBuilder::new(self.renderer)
    }

    pub fn clear(&mut self, color: Color) {
        self.renderer.clear(color);
    }

    pub fn screen_size(&self) -> [f32; 2] {
        [
            self.renderer.screen_width() as f32,
            self.renderer.screen_height() as f32,
        ]
    }
}
