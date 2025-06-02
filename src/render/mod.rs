pub mod camera;
pub mod primitives;
mod renderer;
mod text;
mod texture;
pub mod vertex;

pub use renderer::Renderer;

use camera::Camera;
use primitives::{RectangleBuilder, TriangleBuilder};
use text::TextBuilder;
use wgpu::Color;

pub struct Graphics<'a> {
    renderer: &'a mut Renderer,
    camera: &'a mut Camera,
}

impl<'a> Graphics<'a> {
    pub fn new(renderer: &'a mut Renderer, camera: &'a mut Camera) -> Self {
        Self { renderer, camera }
    }

    pub fn tri(&mut self) -> TriangleBuilder {
        TriangleBuilder::new(self.renderer, &self.camera)
    }

    pub fn rect(&mut self) -> RectangleBuilder {
        RectangleBuilder::new(self.renderer, &self.camera)
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

    pub fn camera(&mut self) -> &mut Camera {
        self.camera
    }

    pub fn text(&mut self, text: &'a str) -> TextBuilder {
        TextBuilder::new(&mut self.renderer.text, text)
    }
}
