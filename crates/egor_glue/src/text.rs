use egor_render::{Device, Queue, RenderPass, TextureFormat, math::Vec2};
use glyphon::{
    Attrs, Buffer, Cache, Color as GlyphonColor, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer as GlyphonRenderer, Viewport,
};

use crate::color::Color;

pub struct TextEntry {
    pub buffer: Buffer,
    pub position: Vec2,
}

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    renderer: GlyphonRenderer,
    entries: Vec<TextEntry>,
    viewport: Viewport,
}

impl TextRenderer {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let mut font_system = FontSystem::new();
        font_system
            .db_mut()
            .load_font_data(include_bytes!("../inter-v19-latin-regular.ttf").to_vec());
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer = GlyphonRenderer::new(&mut atlas, device, Default::default(), None);

        Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            entries: Vec::new(),
            viewport,
        }
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue, width: u32, height: u32) {
        self.viewport.update(queue, Resolution { width, height });
        let text_areas: Vec<TextArea> = self
            .entries
            .iter()
            .map(|entry| TextArea {
                buffer: &entry.buffer,
                left: entry.position.x,
                top: entry.position.y,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: width as i32,
                    bottom: height as i32,
                },
                scale: 1.0,
                default_color: GlyphonColor::rgb(255, 255, 255),
                custom_glyphs: &[],
            })
            .collect();

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();

        self.entries.clear();
    }

    pub fn render<'a>(&'a mut self, pass: &mut RenderPass<'a>) {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .unwrap();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        for entry in &mut self.entries {
            entry.buffer.set_size(
                &mut self.font_system,
                Some(width as f32),
                Some(height as f32),
            );
        }
    }

    // Internal access for TextBuilder
    fn font_system_mut(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }

    fn push_entry(&mut self, entry: TextEntry) {
        self.entries.push(entry);
    }
}

/// High-level text drawing API
///
/// Create with [`Graphics::text`] (or equivalent), configure,
/// and let it drop to submit the text for the current frame
///
/// Text is queued immediately and rendered on the next frame
///
/// # Example
/// ```no_run
/// graphics.text("Hello world")
///     .at((20.0, 40.0))
///     .size(18.0)
///     .color(Color::WHITE);
/// ```
pub struct TextBuilder<'a> {
    renderer: &'a mut TextRenderer,
    text: String,
    position: Vec2,
    size: f32,
    color: Color,
}

impl<'a> TextBuilder<'a> {
    pub fn new(renderer: &'a mut TextRenderer, text: String) -> Self {
        Self {
            renderer,
            text,
            position: Vec2::new(10.0, 10.0),
            size: 16.0,
            color: Color::BLACK,
        }
    }

    pub fn at(mut self, position: impl Into<Vec2>) -> Self {
        self.position = position.into();
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Drop for TextBuilder<'_> {
    fn drop(&mut self) {
        let mut buffer = Buffer::new(
            self.renderer.font_system_mut(),
            Metrics::new(self.size, 1.0),
        );

        buffer.set_text(
            self.renderer.font_system_mut(),
            &self.text,
            &Attrs::new().color(self.color.into()),
            Shaping::Basic,
        );

        self.renderer.push_entry(TextEntry {
            buffer,
            position: self.position,
        });
    }
}
