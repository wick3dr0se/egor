use std::collections::HashMap;

use cosmic_text::{
    Attrs, Buffer, CacheKey, Family, FontSystem, Metrics, Shaping, Style, SwashCache, SwashContent,
    SwashImage, Weight,
};
use egor_render::{Renderer, Space, batch::BatchKey, instance::Instance};
use glam::Vec2;

use crate::{
    camera::Camera, color::Color, graphics::to_clip_rect, math::Rect, primitives::BatchList,
};

/// Side length of the square glyph atlas, in pixels
const ATLAS_SIZE: u32 = 2048;
/// Keeps neighbours from bleeding into each other when the atlas is filtered
const ATLAS_PAD: u32 = 1;
const MAX_POOLED_BUFFERS: usize = 64;

/// Where a glyph sits in the atlas, & the pixels it covers around the pen
#[derive(Clone, Copy)]
struct GlyphSlot {
    left: i32,
    top: i32,
    width: u32,
    height: u32,
    uv: [f32; 4],
}

/// Glyphs live in one RGBA texture: white pixels, coverage in alpha, so text
/// draws through the same pipeline & shader as any other textured quad
struct GlyphAtlas {
    texture: usize,
    pixels: Vec<u8>,
    /// Shelf packer: current pen & the height of the shelf it is filling
    x: u32,
    y: u32,
    row: u32,
    slots: HashMap<CacheKey, GlyphSlot>,
    dirty: bool,
}

impl GlyphAtlas {
    fn new(renderer: &mut Renderer) -> Self {
        let pixels = vec![0; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize];
        // the texture exists from the start, so queued glyphs can name it
        let texture = renderer.add_texture_raw(ATLAS_SIZE, ATLAS_SIZE, &pixels);

        Self {
            texture,
            pixels,
            x: 0,
            y: 0,
            row: 0,
            slots: HashMap::new(),
            dirty: false,
        }
    }

    /// Reserve a spot, dropping down a shelf when the row is full
    fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if width > ATLAS_SIZE || height > ATLAS_SIZE {
            return None;
        }

        if self.x + width > ATLAS_SIZE {
            self.x = 0;
            self.y += self.row + ATLAS_PAD;
            self.row = 0;
        }

        if self.y + height > ATLAS_SIZE {
            // out of room: start over, the cache refills as text is queued
            self.slots.clear();
            self.x = 0;
            self.y = 0;
            self.row = 0;
        }

        let origin = (self.x, self.y);
        self.x += width + ATLAS_PAD;
        self.row = self.row.max(height);
        Some(origin)
    }

    /// Rasterize a glyph into the atlas, returning where it landed
    fn insert(&mut self, cache_key: CacheKey, image: &SwashImage) -> Option<GlyphSlot> {
        let (width, height) = (image.placement.width, image.placement.height);
        if width == 0 || height == 0 {
            return None;
        }

        let (x, y) = self.allocate(width, height)?;
        let channels = match image.content {
            SwashContent::Color => 4,
            _ => 1,
        };

        for row in 0..height {
            for col in 0..width {
                let src = ((row * width + col) as usize) * channels;
                let dst = ((((y + row) * ATLAS_SIZE) + x + col) * 4) as usize;

                if image.content == SwashContent::Color {
                    self.pixels[dst..dst + 4].copy_from_slice(&image.data[src..src + 4]);
                    continue;
                }

                // a subpixel mask carries a channel per subpixel, green reads best
                let coverage = match image.content {
                    SwashContent::SubpixelMask => image.data[src + 1],
                    _ => image.data[src],
                };
                self.pixels[dst..dst + 4].copy_from_slice(&[255, 255, 255, coverage]);
            }
        }

        self.dirty = true;

        let slot = GlyphSlot {
            left: image.placement.left,
            top: image.placement.top,
            width,
            height,
            uv: [
                x as f32 / ATLAS_SIZE as f32,
                y as f32 / ATLAS_SIZE as f32,
                (x + width) as f32 / ATLAS_SIZE as f32,
                (y + height) as f32 / ATLAS_SIZE as f32,
            ],
        };
        self.slots.insert(cache_key, slot);
        Some(slot)
    }

    /// Upload new glyphs once per frame, only when something changed
    fn flush(&mut self, renderer: &mut Renderer) {
        if self.dirty {
            renderer.update_texture_raw(self.texture, ATLAS_SIZE, ATLAS_SIZE, &self.pixels);
            self.dirty = false;
        }
    }
}

/// Shapes text & turns it into glyph quads in the caller's batch
pub struct TextSystem {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: GlyphAtlas,
    buffer_pool: Vec<Buffer>,
}

impl TextSystem {
    pub(crate) fn new(renderer: &mut Renderer) -> Self {
        let mut font_system = FontSystem::new();
        // system fonts vary by machine & are absent on wasm, so embed one
        font_system
            .db_mut()
            .load_font_data(include_bytes!("inter-v19-latin-regular.ttf").to_vec());

        Self {
            font_system,
            swash_cache: SwashCache::new(),
            atlas: GlyphAtlas::new(renderer),
            buffer_pool: Vec::new(),
        }
    }

    pub fn load_font_bytes(&mut self, bytes: &[u8]) -> Option<String> {
        self.font_system.db_mut().load_font_data(bytes.to_vec());
        let face = self.font_system.db().faces().last()?;
        let family = face.families.first()?.0.clone();
        Some(family)
    }

    /// Upload any glyphs rasterized since the last call
    pub(crate) fn flush(&mut self, renderer: &mut Renderer) {
        self.atlas.flush(renderer);
    }

    /// Takes a buffer from the pool, or creates a new one with the given metrics
    fn take_buffer(&mut self, metrics: Metrics) -> Buffer {
        if let Some(mut buffer) = self.buffer_pool.pop() {
            buffer.set_metrics(&mut self.font_system, metrics);
            buffer
        } else {
            Buffer::new(&mut self.font_system, metrics)
        }
    }
}

/// Alignment of text (for use with and) relative to a rectangle
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// A builder for queuing a single line of text to the [`TextSystem`].
/// The text is uploaded and rendered on the next frame
///
/// # Example
/// ```ignore
/// gfx.text("Hello World").at((100.0, 50.0)).size(24.0).color(Color::WHITE);
/// ```
pub struct TextBuilder<'a> {
    /// Reference to the renderer that will draw this text
    renderer: &'a mut TextSystem,
    /// Batch the glyphs are queued into, in draw order with everything else
    batch: &'a mut BatchList,
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
    /// Screen-space rect the text is clipped to
    clip: Option<Rect>,
    /// Space the position and rect are given in
    space: Space,
    /// Camera used to project world positions into pixels
    camera: Camera,
    /// Whether world text grows and shrinks with the camera zoom
    scale_with_zoom: bool,
    /// Draw layer, lower layers are drawn first
    layer: i16,
}

impl<'a> TextBuilder<'a> {
    /// Create a new text builder that will push text to the renderer
    ///
    /// A default font family is selected automatically. Use [`Self::font`] to override it
    pub(crate) fn new(
        renderer: &'a mut TextSystem,
        batch: &'a mut BatchList,
        text: String,
        camera: Camera,
        clip: Option<Rect>,
    ) -> Self {
        Self {
            renderer,
            batch,
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
            clip,
            space: Space::Screen,
            camera,
            scale_with_zoom: false,
            layer: 0,
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

    /// Set the position of the text (top-left corner, or the anchor if one is set)
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

    /// Anchor the text around [`Self::at`] instead of treating it as the top-left.
    /// Ignored when [`Self::in_rect`] is set.
    pub fn anchor(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Draw the text in world space, following the camera
    ///
    /// Size stays constant in pixels unless [`Self::scale_with_zoom`] is set.
    pub fn world(mut self) -> Self {
        self.space = Space::World;
        self
    }

    /// Draw the text in screen space, the default
    pub fn screen(mut self) -> Self {
        self.space = Space::Screen;
        self
    }

    /// Scale world text with the camera zoom, so it keeps its size in world units
    pub fn scale_with_zoom(mut self) -> Self {
        self.scale_with_zoom = true;
        self
    }

    /// Set the draw layer, lower layers are drawn first
    pub fn layer(mut self, layer: i16) -> Self {
        self.layer = layer;
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
        // glyphs are laid out in pixels, so world text is projected here
        let scale = match (self.space, self.scale_with_zoom) {
            (Space::World, true) => self.camera.zoom(),
            _ => 1.0,
        };
        let size = self.size * scale;
        let line_height = self.line_height.unwrap_or(self.size * 1.2) * scale;

        let (position, rect) = match self.space {
            Space::Screen => (self.position, self.rect),
            Space::World => (
                self.camera.world_to_screen(self.position),
                self.rect.map(|rect| {
                    Rect::new(
                        self.camera.world_to_screen(rect.position),
                        rect.size * self.camera.zoom(),
                    )
                }),
            ),
        };

        let key = BatchKey {
            texture: Some(self.renderer.atlas.texture),
            shader: None,
            clip: self.clip.map(to_clip_rect),
            // glyphs are already in pixels, whatever space they came from
            space: Space::Screen,
            layer: self.layer,
        };

        let mut buffer = self.renderer.take_buffer(Metrics::new(size, line_height));
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
        buffer.shape_until_scroll(&mut self.renderer.font_system, false);

        let mut text_w: f32 = 0.0;
        let mut lines = 0.0;
        for run in buffer.layout_runs() {
            text_w = text_w.max(run.line_w);
            lines += 1.0;
        }
        let text_h = lines * line_height;

        let position = if let Some(rect) = rect {
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
            // anchor the text around the point instead of starting at it
            let dx = match self.align {
                Align::TopLeft | Align::MiddleLeft | Align::BottomLeft => 0.0,
                Align::TopCenter | Align::MiddleCenter | Align::BottomCenter => -text_w * 0.5,
                Align::TopRight | Align::MiddleRight | Align::BottomRight => -text_w,
            };
            let dy = match self.align {
                Align::TopLeft | Align::TopCenter | Align::TopRight => 0.0,
                Align::MiddleLeft | Align::MiddleCenter | Align::MiddleRight => -text_h * 0.5,
                Align::BottomLeft | Align::BottomCenter | Align::BottomRight => -text_h,
            };

            position + Vec2::new(dx, dy)
        };

        let color = self.color.components();
        let TextSystem {
            font_system,
            swash_cache,
            atlas,
            ..
        } = &mut *self.renderer;

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((position.x, position.y), 1.0);

                let slot = match atlas.slots.get(&physical.cache_key).copied() {
                    Some(slot) => slot,
                    None => {
                        let Some(image) = swash_cache
                            .get_image(font_system, physical.cache_key)
                            .as_ref()
                        else {
                            continue;
                        };
                        match atlas.insert(physical.cache_key, image) {
                            Some(slot) => slot,
                            None => continue,
                        }
                    }
                };

                let (w, h) = (slot.width as f32, slot.height as f32);
                // the glyph's box starts at the pen, offset by its placement
                let x = (physical.x + slot.left) as f32;
                let y = run.line_y.round() + physical.y as f32 - slot.top as f32;

                self.batch.push_instance(
                    Instance::new([w, 0.0, 0.0, h], [x + w * 0.5, y + h * 0.5], color, slot.uv),
                    key,
                );
            }
        }

        if self.renderer.buffer_pool.len() < MAX_POOLED_BUFFERS {
            self.renderer.buffer_pool.push(buffer);
        }
    }
}
