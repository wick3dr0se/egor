use egor_render::{
    ClipRect, Renderer, TextureFormat,
    batch::GeometryBatch,
    target::{OffscreenTarget, RenderTarget},
};
use glam::Vec2;

use crate::primitives::PathBuilder;
use crate::{
    camera::Camera,
    color::Color,
    math::Rect,
    primitives::{PolygonBuilder, PolylineBuilder, PrimitiveBatch, RectangleBuilder},
    text::{TextBuilder, TextRenderer},
};

/// Converts a screen-space [`Rect`] into an integer [`ClipRect`]
///
/// The origin is floored & the far edge is ceiled so antialiased edges aren't
/// clipped by rounding. A zero-area result means fully clipped.
pub(crate) fn to_clip_rect(rect: Rect) -> ClipRect {
    let x = rect.position.x.max(0.0).floor();
    let y = rect.position.y.max(0.0).floor();
    let right = (rect.position.x + rect.size.x).max(0.0).ceil();
    let bottom = (rect.position.y + rect.size.y).max(0.0).ceil();
    ClipRect::new(
        x as u32,
        y as u32,
        (right - x).max(0.0) as u32,
        (bottom - y).max(0.0) as u32,
    )
}

/// High-level 2D drawing interface that simplifies the [`Renderer`]
pub struct Graphics<'a> {
    renderer: &'a mut Renderer,
    batch: &'a mut PrimitiveBatch,
    camera: Camera,
    text_renderer: &'a mut TextRenderer,
    target_format: TextureFormat,
    target_size: (u32, u32),
    current_shader: Option<usize>,
    clip: Option<Rect>,
}

impl<'a> Graphics<'a> {
    /// Create `Graphics` with [`Renderer`], [`TextRenderer`] & `TextureFormat`
    pub fn new(
        renderer: &'a mut Renderer,
        batch: &'a mut PrimitiveBatch,
        text_renderer: &'a mut TextRenderer,
        format: TextureFormat,
        w: u32,
        h: u32,
    ) -> Self {
        renderer.set_target_size(w, h);
        Self {
            renderer,
            batch,
            camera: Camera::default(),
            text_renderer,
            target_format: format,
            target_size: (w, h),
            current_shader: None,
            clip: None,
        }
    }

    /// Restricts drawing inside `render_fn` to `rect` (screen space)
    pub fn with_clip(&mut self, rect: Rect, mut render_fn: impl FnMut(&mut Self)) {
        let previous = self.clip;
        self.clip = Some(rect);
        render_fn(self);
        self.clip = previous;
    }

    /// Current clip as an integer [`ClipRect`], if a clip is active
    fn current_clip(&self) -> Option<ClipRect> {
        self.clip.map(to_clip_rect)
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
        render_fn: impl FnMut(&mut Graphics),
    ) {
        self.render_offscreen_with_limits(
            target,
            GeometryBatch::DEFAULT_MAX_VERTICES,
            GeometryBatch::DEFAULT_MAX_INDICES,
            render_fn,
        );
    }

    /// Render to an offscreen target using a temporary batch with custom vertex/index buffer limits.
    /// Use this when the default limits are too large for memory-constrained platforms,
    /// or too small for complex offscreen scenes.
    /// For most cases, prefer [`Self::render_offscreen`] which uses sensible defaults
    pub fn render_offscreen_with_limits(
        &mut self,
        target: &mut OffscreenTarget,
        max_verts: usize,
        max_indices: usize,
        mut render_fn: impl FnMut(&mut Graphics),
    ) {
        let (w, h) = target.size();
        let format = target.format();

        let mut offscreen_batch = PrimitiveBatch::new(max_verts, max_indices);

        {
            let mut offscreen_gfx = Graphics {
                renderer: self.renderer,
                batch: &mut offscreen_batch,
                camera: Camera::default(),
                text_renderer: self.text_renderer,
                target_size: (w, h),
                target_format: format,
                current_shader: None,
                clip: None,
            };

            // The offscreen pass has its own target dimensions; scissor resets use them
            offscreen_gfx.renderer.set_target_size(w, h);

            render_fn(&mut offscreen_gfx);
            offscreen_gfx.upload_camera();
        }

        let mut geometry = offscreen_batch.take();

        let mut encoder = self
            .renderer
            .device()
            .create_command_encoder(&Default::default());

        {
            let mut r_pass = self
                .renderer
                .begin_render_pass(&mut encoder, target.render_view());

            for (key, batch) in &mut geometry {
                self.renderer.draw_batch(&mut r_pass, batch, *key);
            }
        }

        target.copy_to_sample(&mut encoder);

        let _ = self.renderer.queue().submit(Some(encoder.finish()));

        // Restore the main target size for the remainder of the frame
        let (mw, mh) = self.target_size;
        self.renderer.set_target_size(mw, mh);
    }

    /// Use an offscreen target as a texture
    pub fn offscreen_as_texture(&mut self, target: &mut OffscreenTarget) -> usize {
        self.renderer.add_offscreen_texture(target)
    }

    pub(crate) fn set_target_size(&mut self, w: u32, h: u32) {
        self.target_size = (w, h);
        self.renderer.set_target_size(w, h);
    }

    /// Upload camera matrix to the GPU.
    /// Call after user drawing is complete and before the render pass
    pub(crate) fn upload_camera(&mut self) {
        let (w, h) = self.target_size;
        let size = (w as f32, h as f32).into();
        self.renderer
            .upload_camera_matrix(self.camera.view_proj(size).to_cols_array_2d());
        // Screen-space batches ignore the camera & map pixels directly:
        // a default camera at the same size is exactly the orthographic
        // identity projection we want for UI.
        self.renderer
            .upload_screen_matrix(Camera::default().view_proj(size).to_cols_array_2d());
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
        RectangleBuilder::new(self.batch, self.current_shader, self.current_clip())
    }
    /// Start building an arbitrary polygon primitive, capable of triangles, circles, n-gons
    pub fn polygon(&mut self) -> PolygonBuilder<'_> {
        PolygonBuilder::new(self.batch, self.current_shader, self.current_clip())
    }
    /// Start building a polyline (stroked path) primitive
    pub fn polyline(&mut self) -> PolylineBuilder<'_> {
        PolylineBuilder::new(self.batch, self.current_shader, self.current_clip())
    }
    /// Start building a vector path (lines + curves) to be filled or stroked
    pub fn path(&mut self) -> PathBuilder<'_> {
        PathBuilder::new(self.batch, self.current_shader, self.current_clip())
    }
    /// Load a font from disk into the text system.
    pub fn load_font(&mut self, bytes: &[u8]) -> Option<String> {
        self.text_renderer.load_font_bytes(bytes)
    }
    /// Draw a line of text
    ///
    /// Text is screen-space by default and clipped by the active clip. Use
    /// [`TextBuilder::world`] for world space.
    pub fn text(&mut self, text: &str) -> TextBuilder<'_> {
        let clip = self.clip;
        TextBuilder::new(
            self.text_renderer,
            self.batch,
            text.to_string(),
            self.camera,
        )
        .clip(clip)
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

    /// Load a custom shader from WGSL source code
    pub fn load_shader(&mut self, wgsl_source: &str) -> usize {
        self.renderer.add_shader(wgsl_source)
    }

    /// Create a uniform buffer from raw bytes, returns a uniform id
    pub fn create_uniform(&mut self, data: &[u8]) -> usize {
        self.renderer.add_uniform(data)
    }

    /// Update an existing uniform buffer with raw bytes
    pub fn update_uniform(&mut self, id: usize, data: &[u8]) {
        self.renderer.update_uniform(id, data);
    }

    /// Load a custom shader with associated uniform buffers
    pub fn load_shader_with_uniforms(&mut self, wgsl_source: &str, uniform_ids: &[usize]) -> usize {
        self.renderer
            .add_shader_with_uniforms(wgsl_source, uniform_ids)
    }

    /// Execute drawing commands with a custom shader
    ///
    /// The shader is automatically reset to default after the closure drops
    pub fn with_shader(&mut self, shader_id: usize, mut render_fn: impl FnMut(&mut Self)) {
        let previous_shader = self.current_shader;
        self.current_shader = Some(shader_id);
        render_fn(self);
        self.current_shader = previous_shader;
    }
}
