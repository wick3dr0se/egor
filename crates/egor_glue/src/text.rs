use egor_render::{Device, Queue, RenderPass, TextureFormat};
use glam::Vec2;
use glyphon::{
    Attrs, Buffer, Cache, Color as GlyphonColor, Family, FontSystem, Metrics, Resolution, Shaping,
    Style, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer as GlyphonRenderer, Viewport,
    Weight,
};

use crate::{color::Color, math::Rect};

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
    buffer_pool: Vec<Buffer>,
}

const MAX_POOLED_BUFFERS: usize = 64;

impl TextRenderer {
    pub(crate) fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let mut font_system = FontSystem::new();
        // Glyphon will use sytstem font but we embed one for wasm + consistency
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
            buffer_pool: Vec::new(),
        }
    }

    pub fn load_font_bytes(&mut self, bytes: &[u8]) -> Option<String> {
        self.font_system.db_mut().load_font_data(bytes.to_vec());
        let face = self.font_system.db().faces().last()?;
        let family = face.families.first()?.0.clone();
        Some(family)
    }

    /// Prepare the text renderer for drawing
    pub(crate) fn prepare(&mut self, device: &Device, queue: &Queue, width: u32, height: u32) {
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

        // Return buffers to the pool for reuse next frame
        for entry in self.entries.drain(..) {
            if self.buffer_pool.len() < MAX_POOLED_BUFFERS {
                self.buffer_pool.push(entry.buffer);
            }
        }
    }

    pub(crate) fn render<'a>(&'a self, pass: &mut RenderPass<'a>) {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .unwrap();
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32, queue: &Queue) {
        self.viewport.update(queue, Resolution { width, height });
    }

    /// Takes a buffer from the pool, or creates a new one with the given metrics
    fn take_buffer(&mut self, metrics: Metrics) -> Buffer {
        if let Some(mut buf) = self.buffer_pool.pop() {
            buf.set_metrics(&mut self.font_system, metrics);
            buf
        } else {
            Buffer::new(&mut self.font_system, metrics)
        }
    }
}

/// Alignment of text (for use with and) relative to a rectangle
pub enum Align {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
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
    /// Top-left anchor position; may be offset by alignment
    position: Vec2,
    /// Optional bounding rectangle for alignment (origin, size)
    rect: Option<Rect>,
    /// Line height in pixels; defaults to `size * 1.2`
    line_height: Option<f32>,
    size: f32,
    color: Color,
    /// Font family name used for matching
    family: String,
    weight: Weight,
    style: Style,
    align: Align,
}

impl<'a> TextBuilder<'a> {
    /// Create a new text builder that will push text to the renderer
    ///
    /// A default font family is selected automatically. Use [`Self::font`] to override it
    pub fn new(renderer: &'a mut TextRenderer, text: String) -> Self {
        Self {
            renderer,
            text,
            position: Vec2::new(10.0, 10.0),
            rect: None,
            size: 16.0,
            line_height: None,
            color: Color::BLACK,
            family: "Inter".into(),
            weight: Weight::NORMAL,
            style: Style::Normal,
            align: Align::TopLeft,
        }
    }

    /// Set the font family used to render the text
    ///
    /// The family must match a font that has been loaded into the renderer.
    /// If the family cannot be found, a fallback font will be used (Inter)
    pub fn font(mut self, family: String) -> Self {
        self.family = family;
        self
    }

    /// Set the screen-space position of the text (top-left corner)
    pub fn at(mut self, position: impl Into<Vec2>) -> Self {
        self.position = position.into();
        self
    }

    /// Sets a bounding rectangle for the text
    ///
    /// The text will be positioned inside `rect` according to the given
    /// [`Align`] value instead of using a raw point
    ///
    /// `rect.position` is the top-left corner and `rect.size` is its width/height
    pub fn in_rect(mut self, rect: Rect, align: Align) -> Self {
        self.rect = Some(rect);
        self.align = align;
        self
    }

    /// Set the font size in points
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the line height in pixels.
    ///
    /// Defaults to `size * 1.2` if not set.
    pub fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = Some(line_height);
        self
    }

    /// Set the text color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Render the text in bold
    pub fn bold(mut self) -> Self {
        self.weight = Weight::BOLD;
        self
    }

    /// Render the text in italic
    pub fn italic(mut self) -> Self {
        self.style = Style::Italic;
        self
    }

    /// Set a specific font weight (100–900).
    ///
    /// Overrides [`Self::bold`]. Common values: 400 = normal, 700 = bold.
    pub fn weight(mut self, weight: u16) -> Self {
        self.weight = Weight(weight);
        self
    }
}

impl Drop for TextBuilder<'_> {
    fn drop(&mut self) {
        let line_height = self.line_height.unwrap_or(self.size * 1.2);
        let mut buffer = self
            .renderer
            .take_buffer(Metrics::new(self.size, line_height));
        buffer.set_text(
            &mut self.renderer.font_system,
            &self.text,
            &Attrs::new()
                .family(Family::Name(&self.family))
                .color(self.color.into())
                .weight(self.weight)
                .style(self.style),
            Shaping::Basic,
        );

        // compute final position, applying alignment within rect if set
        let position = if let Some(rect) = self.rect {
            buffer.shape_until_scroll(&mut self.renderer.font_system, false);
            let text_w = buffer
                .layout_runs()
                .map(|r| r.line_w)
                .fold(0.0_f32, f32::max);
            let text_h = buffer.layout_runs().count() as f32 * line_height;

            let x = match self.align {
                Align::TopLeft | Align::MiddleLeft | Align::BottomLeft => rect.position.x,
                Align::TopCenter | Align::MiddleCenter | Align::BottomCenter => {
                    rect.position.x + (rect.size.x - text_w) * 0.5
                }
                Align::TopRight | Align::MiddleRight | Align::BottomRight => {
                    rect.position.x + rect.size.x - text_w
                }
            };
            let y = match self.align {
                Align::TopLeft | Align::TopCenter | Align::TopRight => rect.position.y,
                Align::MiddleLeft | Align::MiddleCenter | Align::MiddleRight => {
                    rect.position.y + (rect.size.y - text_h) * 0.5
                }
                Align::BottomLeft | Align::BottomCenter | Align::BottomRight => {
                    rect.position.y + rect.size.y - text_h
                }
            };

            Vec2::new(x, y)
        } else {
            self.position
        };

        self.renderer.entries.push(TextEntry { buffer, position });
    }
}
