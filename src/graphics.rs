use egor_render::{
    ClipRect, Renderer, Space, Texture, TextureFormat,
    batch::{BatchKey, GeometryBatch},
    target::{OffscreenTarget, RenderTarget},
};
use glam::Vec2;

use crate::primitives::PathBuilder;
use crate::{
    camera::Camera,
    color::Color,
    math::Rect,
    primitives::{BatchList, PolygonBuilder, PolylineBuilder, RectangleBuilder},
    text::{TextBuilder, TextSystem},
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
    batches: &'a mut BatchList,
    camera: Camera,
    text_system: &'a mut TextSystem,
    target_format: TextureFormat,
    target_size: (u32, u32),
    current_shader: Option<usize>,
    clip: Option<Rect>,
}

impl<'a> Graphics<'a> {
    /// Create `Graphics` with [`Renderer`], [`TextSystem`] & `TextureFormat`
    pub(crate) fn new(
        renderer: &'a mut Renderer,
        batches: &'a mut BatchList,
        text_system: &'a mut TextSystem,
        format: TextureFormat,
        w: u32,
        h: u32,
    ) -> Self {
        renderer.set_target_size(w, h);
        let mut camera = Camera::default();
        camera.set_view_size((w as f32, h as f32).into());
        Self {
            renderer,
            batches,
            camera,
            text_system,
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

    /// The key a shape starts from; its builder overrides texture, space or layer
    fn draw_key(&self) -> BatchKey {
        BatchKey {
            texture: None,
            shader: self.current_shader,
            clip: self.current_clip(),
            space: Space::World,
            layer: 0,
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

        let mut offscreen_batch = BatchList::new(max_verts, max_indices);

        {
            let mut offscreen_camera = Camera::default();
            offscreen_camera.set_view_size((w as f32, h as f32).into());

            let mut offscreen_gfx = Graphics {
                renderer: self.renderer,
                batches: &mut offscreen_batch,
                camera: offscreen_camera,
                text_system: self.text_system,
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
    ///
    /// Released when the handle is dropped, independently of the target.
    pub fn offscreen_as_texture(&mut self, target: &mut OffscreenTarget) -> Texture {
        self.renderer.add_offscreen_texture(target)
    }

    pub(crate) fn set_target_size(&mut self, w: u32, h: u32) {
        self.target_size = (w, h);
        self.camera.set_view_size((w as f32, h as f32).into());
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
        RectangleBuilder::new(self.batches, self.draw_key())
    }
    /// Start building an arbitrary polygon primitive, capable of triangles, circles, n-gons
    pub fn polygon(&mut self) -> PolygonBuilder<'_> {
        PolygonBuilder::new(self.batches, self.draw_key())
    }
    /// Start building a polyline (stroked path) primitive
    pub fn polyline(&mut self) -> PolylineBuilder<'_> {
        PolylineBuilder::new(self.batches, self.draw_key())
    }
    /// Start building a vector path (lines + curves) to be filled or stroked
    pub fn path(&mut self) -> PathBuilder<'_> {
        PathBuilder::new(self.batches, self.draw_key())
    }
    /// Load a font from disk into the text system.
    pub fn load_font(&mut self, bytes: &[u8]) -> Option<String> {
        self.text_system.load_font_bytes(bytes)
    }
    /// Draw a line of text
    ///
    /// Text is screen-space by default and clipped by the active clip. Use
    /// `.world()` for world space.
    pub fn text(&mut self, text: &str) -> TextBuilder<'_> {
        // the atlas is allocated when text first asks for it
        self.text_system.ensure_atlas(self.renderer);
        TextBuilder::new(
            self.text_system,
            self.batches,
            text.to_string(),
            self.camera,
            self.clip,
        )
    }

    /// Load a texture from encoded image bytes, such as a PNG
    ///
    /// The texture is released when the returned handle is dropped, so keep it alive
    /// for as long as anything draws with it.
    /// Typically called once during initialization (when `timer.frame == 0`).
    pub fn load_texture(&mut self, data: &[u8]) -> Texture {
        self.renderer.add_texture(data)
    }
    /// Load a texture from raw RGBA pixels, with no decoding
    ///
    /// Use this for an atlas slice or pixels generated at runtime.
    pub fn load_texture_raw(&mut self, w: u32, h: u32, data: &[u8]) -> Texture {
        self.renderer.add_texture_raw(w, h, data)
    }
    /// Replace a texture's pixels, keeping its handle valid
    pub fn update_texture(&mut self, texture: &Texture, data: &[u8]) {
        self.renderer.update_texture(texture.id(), data);
    }
    /// Replace a texture's pixels from raw RGBA bytes
    pub fn update_texture_raw(&mut self, texture: &Texture, w: u32, h: u32, data: &[u8]) {
        self.renderer.update_texture_raw(texture.id(), w, h, data);
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
