pub mod camera;
pub mod color;
pub mod math;
pub mod primitives;
mod renderer;
pub mod text;
pub mod texture;
pub mod vertex;

use glam::Vec2;

pub use renderer::Renderer;

use crate::{
    camera::{Camera, CameraInternal},
    color::Color,
    primitives::RectangleBuilder,
    renderer::GeometryBatch,
    text::TextBuilder,
    vertex::Vertex,
};

#[derive(Default)]
struct PrimitiveBatch {
    geometry: Vec<(usize, GeometryBatch)>,
}

impl PrimitiveBatch {
    fn push(&mut self, verts: &[Vertex], indices: &[u16], texture_id: usize) {
        if let Some((_, batch)) = self.geometry.iter_mut().find(|(id, _)| *id == texture_id) {
            batch.push(verts, indices);
        } else {
            let mut batch = GeometryBatch::default();
            batch.push(verts, indices);
            self.geometry.push((texture_id, batch));
        }
    }

    fn take(&mut self) -> Vec<(usize, GeometryBatch)> {
        std::mem::take(&mut self.geometry)
    }
}

pub struct Graphics<'a> {
    renderer: &'a mut Renderer,
    batch: PrimitiveBatch,
    camera: Camera,
}

impl<'a> Graphics<'a> {
    pub fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            renderer,
            batch: PrimitiveBatch::default(),
            camera: Camera::default(),
        }
    }

    pub fn rect(&mut self) -> RectangleBuilder<'_> {
        RectangleBuilder::new(&mut self.batch)
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

pub trait GraphicsInternal {
    fn flush(&mut self) -> Vec<(usize, GeometryBatch)>;
}

impl GraphicsInternal for Graphics<'_> {
    fn flush(&mut self) -> Vec<(usize, GeometryBatch)> {
        self.renderer
            .upload_camera_matrix(self.camera.view_proj(self.renderer.surface_size().into()));
        self.batch.take()
    }
}
