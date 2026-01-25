use egor_render::{Device, Queue, RenderPass, TextureFormat};
use glam::Vec2;
use glyphon::{
    Attrs, Buffer, Cache, Color as GlyphonColor, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer as GlyphonRenderer, Viewport,
};

use crate::color::Color;

struct TextEntry {
    buffer: Buffer,
    position: Vec2,
}

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    renderer: GlyphonRenderer,
    viewport: Viewport,
    entries: Vec<TextEntry>,
}

impl TextRenderer {
    pub(crate) fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
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
            viewport,
            entries: Vec::new(),
        }
    }

    /// Prepare the text renderer for drawing
    pub(crate) fn prepare(&mut self, device: &Device, queue: &Queue, width: u32, height: u32) {
        if self.entries.is_empty() {
            return;
        }

        let text_areas: Vec<TextArea> = self
            .entries
            .iter()
            .map(|entry| TextArea {
                buffer: &entry.buffer,
                left: entry.position.x,
                top: entry.position.y,
                bounds: TextBounds {
                    right: width as i32,
                    bottom: height as i32,
                    ..Default::default()
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

    pub(crate) fn render<'a>(&'a self, pass: &mut RenderPass<'a>) {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .unwrap();
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32, queue: &Queue) {
        self.viewport.update(queue, Resolution { width, height });
    }
}

/// A builder for queuing a single line of text to the [`TextRenderer`].
/// The text is uploaded and rendered on the next frame
///
/// # Example
/// ```ignore
/// gfx.text("Hello World").at((100.0, 50.0)).size(24.0).color(Color::WHITE);
/// ```
pub struct TextBuilder<'a> {
    /// Reference to the renderer that will draw this text
    renderer: &'a mut TextRenderer,
    /// The string content to render
    text: String,
    /// Screen-space position of the text (top-left corner)
    position: Vec2,
    size: f32,
    color: Color,
}

impl<'a> TextBuilder<'a> {
    /// Create a new text builder that will push text to the renderer
    pub fn new(renderer: &'a mut TextRenderer, text: String) -> Self {
        Self {
            renderer,
            text,
            position: Vec2::new(10.0, 10.0),
            size: 16.0,
            color: Color::BLACK,
        }
    }

    /// Set the position of text in screen space
    pub fn at(mut self, position: impl Into<Vec2>) -> Self {
        self.position = position.into();
        self
    }
    /// Set the font size in points
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
    /// Set the text color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Drop for TextBuilder<'_> {
    fn drop(&mut self) {
        let mut buffer = Buffer::new(&mut self.renderer.font_system, Metrics::new(self.size, 1.0));
        buffer.set_text(
            &mut self.renderer.font_system,
            &self.text,
            &Attrs::new().color(self.color.into()),
            Shaping::Basic,
        );

        self.renderer.entries.push(TextEntry {
            buffer,
            position: self.position,
        });
    }
}
