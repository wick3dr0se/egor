use egor_render::{GeometryBatch, RenderTarget, Renderer, TextureFormat, target::OffscreenTarget};
use glam::Vec2;

use crate::{
    camera::Camera,
    color::Color,
    primitives::{PolygonBuilder, PolylineBuilder, PrimitiveBatch, RectangleBuilder},
    text::{TextBuilder, TextRenderer},
};

/// High-level 2D drawing interface that simplifies the [`Renderer`]
pub struct Graphics<'a> {
    renderer: &'a mut Renderer,
    batch: PrimitiveBatch,
    camera: Camera,
    text_renderer: &'a mut TextRenderer,
    target_format: TextureFormat,
    target_size: (u32, u32),
}

impl<'a> Graphics<'a> {
    /// Create `Graphics` with [`Renderer`], [`TextRenderer`] & `TextureFormat`
    pub fn new(
        renderer: &'a mut Renderer,
        text_renderer: &'a mut TextRenderer,
        format: TextureFormat,
        w: u32,
        h: u32,
    ) -> Self {
        Self {
            renderer,
            batch: PrimitiveBatch::default(),
            camera: Camera::default(),
            text_renderer,
            target_format: format,
            target_size: (w, h),
        }
    }

    /// Create a new offscreen render target
    pub fn create_offscreen(&self, width: u32, height: u32) -> OffscreenTarget {
        self.renderer
            .create_offscreen_target(width, height, self.target_format)
    }

    /// Render to an offscreen target
    pub fn render_offscreen(
        &mut self,
        target: &mut OffscreenTarget,
        mut render_fn: impl FnMut(&mut Graphics),
    ) {
        let Some(mut frame) = self.renderer.begin_frame(target) else {
            return;
        };

        let (w, h) = target.size();
        let format = target.format();

        let mut offscreen_gfx = Graphics {
            renderer: self.renderer,
            batch: PrimitiveBatch::default(),
            camera: Camera::default(),
            text_renderer: self.text_renderer,
            target_size: (w, h),
            target_format: format,
        };

        render_fn(&mut offscreen_gfx);

        let mut geometry = offscreen_gfx.flush();
        let mut r_pass = self
            .renderer
            .begin_render_pass(&mut frame.encoder, &frame.view);

        for (tex_id, batch) in &mut geometry {
            self.renderer.draw_batch(&mut r_pass, batch, *tex_id);
        }

        drop(r_pass);
        self.renderer.end_frame(frame);
    }

    /// Use an offscreen target as a texture
    pub fn offscreen_as_texture(&mut self, target: &OffscreenTarget) -> usize {
        self.renderer.add_offscreen_texture(target)
    }

    /// Upload camera matrix & extract batched geometry
    pub(crate) fn flush(&mut self) -> Vec<(usize, GeometryBatch)> {
        let (w, h) = self.target_size;
        self.renderer.upload_camera_matrix(
            self.camera
                .view_proj((w as f32, h as f32).into())
                .to_cols_array_2d(),
        );
        self.batch.take()
    }

    /// Clear the screen to a color
    pub fn clear(&mut self, color: Color) {
        self.renderer.set_clear_color(color.into());
    }
    /// Get current surface size in pixels
    pub fn screen_size(&self) -> Vec2 {
        let (w, h) = self.target_size;
        (w as f32, h as f32).into()
    }
    /// Mutable access to [`Camera`]
    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Start building a rectangle primitive
    pub fn rect(&mut self) -> RectangleBuilder<'_> {
        RectangleBuilder::new(&mut self.batch)
    }
    /// Start building an arbitrary polygon primitive, capable of triangles, circles, n-gons
    pub fn polygon(&mut self) -> PolygonBuilder<'_> {
        PolygonBuilder::new(&mut self.batch)
    }
    /// Start building a polyline (stroked path) primitive
    pub fn polyline(&mut self) -> PolylineBuilder<'_> {
        PolylineBuilder::new(&mut self.batch)
    }

    /// Load a font from disk into the text system.
    pub fn load_font(&mut self, bytes: &[u8]) -> Option<String> {
        self.text_renderer.load_font_bytes(bytes)
    }
    /// Draw a line of text
    pub fn text(&mut self, text: &str) -> TextBuilder<'_> {
        TextBuilder::new(self.text_renderer, text.to_string())
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
