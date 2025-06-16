pub mod camera;
pub mod color;
pub mod math;
pub mod primitives;
mod renderer;
mod text;
mod texture;
pub(crate) mod vertex;

use glam::Vec2;
pub use renderer::Renderer;

use camera::Camera;
use primitives::{RectangleBuilder, TriangleBuilder};
use text::TextBuilder;

use crate::Color;

pub struct Graphics<'a> {
    renderer: &'a mut Renderer,
    camera: Camera,
}

impl<'a> Graphics<'a> {
    pub fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            renderer,
            camera: Camera::new(),
        }
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

    pub fn screen_size(&self) -> Vec2 {
        self.renderer.surface_size().into()
    }

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn text(&mut self, text: &str) -> TextBuilder {
        TextBuilder::new(&mut self.renderer.text, text.to_string())
    }
    pub fn update_texture(&mut self, index: usize, data: &[u8]) {
        self.renderer.update_texture(index, data);
    }
    pub fn update_texture_raw(&mut self, index: usize, w: u32, h: u32, data: &[u8]) {
        self.renderer.update_texture_raw(index, w, h, data);
    }
}
