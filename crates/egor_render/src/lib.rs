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
use primitives::RectangleBuilder;
use text::TextBuilder;

use crate::{color::Color, vertex::Vertex};

pub struct Graphics<'a> {
    renderer: &'a mut Renderer,
    camera: Camera,
}

impl<'a> Graphics<'a> {
    pub fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            renderer,
            camera: Camera::default(),
        }
    }

    pub fn rect(&mut self) -> RectangleBuilder<'_> {
        RectangleBuilder::new(self)
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

    pub fn text(&mut self, text: &str) -> TextBuilder<'_> {
        TextBuilder::new(&mut self.renderer.text, text.to_string())
    }

    pub fn update_texture(&mut self, index: usize, data: &[u8]) {
        self.renderer.update_texture(index, data);
    }

    pub fn update_texture_raw(&mut self, index: usize, w: u32, h: u32, data: &[u8]) {
        self.renderer.update_texture_raw(index, w, h, data);
    }
}

pub(crate) trait GeometrySink {
    fn world_to_ndc(&self, pos: Vec2) -> [f32; 2];
    fn queue(&mut self, verts: &[Vertex], indices: &[u16], texture_index: usize);
}

impl<'a> GeometrySink for Graphics<'a> {
    fn world_to_ndc(&self, pos: Vec2) -> [f32; 2] {
        let screen = self
            .camera
            .world_to_screen(pos, self.renderer.surface_size().into());
        self.renderer.to_ndc(screen.x, screen.y)
    }

    fn queue(&mut self, verts: &[Vertex], indices: &[u16], texture_index: usize) {
        self.renderer.queue_geometry(verts, indices, texture_index);
    }
}
