use egor_render::{GeometryBatch, Renderer, color::Color, math::Vec2};

use crate::{
    camera::Camera,
    primitives::{PrimitiveBatch, RectangleBuilder},
    text::TextBuilder,
};

/// High-level 2D drawing interface that simplifies the [`Renderer`]
pub struct Graphics<'a> {
    renderer: &'a mut Renderer,
    batch: PrimitiveBatch,
    camera: Camera,
}

impl<'a> Graphics<'a> {
    /// Upload camera matrix & extract batched geometry for [`Renderer::render_frame()`]
    pub(crate) fn flush(&mut self) -> Vec<(usize, GeometryBatch)> {
        self.renderer
            .upload_camera_matrix(self.camera.view_proj(self.renderer.surface_size().into()));
        self.batch.take()
    }

    /// Create new `Graphics` tied to [`Renderer`]
    pub fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            renderer,
            batch: PrimitiveBatch::default(),
            camera: Camera::default(),
        }
    }

    /// Start building a rectangle primitive
    pub fn rect(&mut self) -> RectangleBuilder<'_> {
        RectangleBuilder::new(&mut self.batch)
    }

    /// Clear the screen to a color
    pub fn clear(&mut self, color: Color) {
        self.renderer.clear_color = color;
    }

    /// Get current surface size in pixels
    pub fn screen_size(&self) -> Vec2 {
        self.renderer.surface_size().into()
    }

    /// Mutable access to [`Camera`]
    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Draw a line of text
    pub fn text(&mut self, text: &str) -> TextBuilder<'_> {
        TextBuilder::new(&mut self.renderer.text, text.to_string())
    }

    /// Load a texture from raw image data (e.g., PNG bytes)
    ///
    /// Returns a texture ID that can be used with `.texture(id)` on primitives.
    /// Typically called once during initialization (when `timer.frame == 0`).
    pub fn load_texture(&mut self, data: &[u8]) -> usize {
        self.renderer.add_texture(data)
    }

    /// Update texture data by index
    pub fn update_texture(&mut self, index: usize, data: &[u8]) {
        self.renderer.update_texture(index, data);
    }

    /// Update texture data by index with raw width/height
    pub fn update_texture_raw(&mut self, index: usize, w: u32, h: u32, data: &[u8]) {
        self.renderer.update_texture_raw(index, w, h, data);
    }
}
