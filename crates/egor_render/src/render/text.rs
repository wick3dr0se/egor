use crate::Color;
use glyphon::{
    Attrs, Buffer, Cache, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, Viewport,
};
use wgpu::{Device, MultisampleState, Queue, TextureFormat};

pub struct TextEntry {
    buffer: Buffer,
    position: (f32, f32),
}

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: glyphon::TextRenderer,
    entries: Vec<TextEntry>,
}

impl TextRenderer {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let mut font_system = FontSystem::new();
        font_system
            .db_mut()
            .load_font_data(include_bytes!("../../inter-v19-latin-regular.ttf").to_vec());
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer =
            glyphon::TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);
        let dummy_buffer = Buffer::new(&mut font_system, Metrics::new(12.0, 14.0));

        Self {
            font_system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            entries: vec![TextEntry {
                buffer: dummy_buffer,
                position: (0.0, 0.0),
            }],
        }
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
                left: entry.position.0,
                top: entry.position.1,
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

        self.renderer
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

    pub fn render<'rp>(&'rp self, pass: &mut wgpu::RenderPass<'rp>) {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .unwrap();
    }
}

pub struct TextBuilder<'a> {
    renderer: &'a mut TextRenderer,
    text: String,
    position: (f32, f32),
    size: f32,
    color: Color,
}

impl<'a> TextBuilder<'a> {
    pub fn new(renderer: &'a mut TextRenderer, text: String) -> Self {
        Self {
            renderer,
            text,
            position: (0.0, 0.0),
            size: 16.0,
            color: Color::BLACK,
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
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
        let mut buffer = Buffer::new(&mut self.renderer.font_system, Metrics::new(self.size, 1.0));
        buffer.set_text(
            &mut self.renderer.font_system,
            &self.text,
            &Attrs::new().color(self.color.into()),
            Shaping::Advanced,
        );

        self.renderer.entries.push(TextEntry {
            buffer,
            position: self.position,
        });
    }
}
