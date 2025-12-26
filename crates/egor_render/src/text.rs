use glam::Vec2;
pub use glyphon::{Attrs, Buffer, Metrics, Shaping};

use glyphon::{
    Cache, FontSystem, Resolution, SwashCache, TextArea, TextAtlas, TextBounds, Viewport,
};
use wgpu::{Device, MultisampleState, Queue, RenderPass, TextureFormat};

use crate::color::Color;

pub struct TextEntry {
    pub buffer: Buffer,
    pub position: Vec2,
}

/// Handles text rendering using [`glyphon`] & [`wgpu`]
pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    inner: glyphon::TextRenderer,
    entries: Vec<TextEntry>,
}

impl TextRenderer {
    /// Creates a new text renderer with the default embedded Inter font
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let mut font_system = FontSystem::new();
        font_system
            .db_mut()
            .load_font_data(include_bytes!("../inter-v19-latin-regular.ttf").to_vec());
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let inner =
            glyphon::TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);
        let dummy_buffer = Buffer::new(&mut font_system, Metrics::new(12.0, 14.0));

        Self {
            inner,
            font_system,
            swash_cache,
            viewport,
            atlas,
            entries: vec![TextEntry {
                buffer: dummy_buffer,
                position: Vec2::new(0.0, 0.0),
            }],
        }
    }

    /// Resizes internal text buffers for a new viewport size
    pub fn resize(&mut self, width: u32, height: u32) {
        for entry in &mut self.entries {
            entry.buffer.set_size(
                &mut self.font_system,
                Some(width as f32),
                Some(height as f32),
            );
        }
    }

    /// Prepares the text layout for this frame
    /// Must be called before [`render()`](Self::render)
    ///
    /// Automatically clears all entries after preparing
    pub fn prepare(&mut self, device: &Device, queue: &Queue, w: u32, h: u32) {
        self.viewport.update(
            queue,
            Resolution {
                width: w,
                height: h,
            },
        );

        let mut areas = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            areas.push(TextArea {
                buffer: &entry.buffer,
                left: entry.position.x,
                top: entry.position.y,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: w as i32,
                    bottom: h as i32,
                },
                scale: 1.0,
                default_color: Color::BLACK.into(),
                custom_glyphs: &[],
            });
        }

        self.inner
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                areas,
                &mut self.swash_cache,
            )
            .unwrap();

        self.entries.clear();
    }

    /// Renders all prepared text
    pub fn render(&self, pass: &mut RenderPass) {
        self.inner
            .render(&self.atlas, &self.viewport, pass)
            .unwrap();
    }

    pub fn font_system_mut(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }

    /// Add a prepared `TextEntry` to be rendered this frame
    pub fn push_entry(&mut self, entry: TextEntry) {
        self.entries.push(entry);
    }
}
