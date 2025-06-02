use glyphon::{
    Attrs, Buffer, Cache, Color, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer, Viewport,
};
use wgpu::{Device, MultisampleState, Queue, TextureFormat};

pub struct Text {
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: TextRenderer,
    buffer: Buffer,
    text: String,
    color: Color,
    position: (f32, f32),
}

impl Text {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer = TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);
        let dummy_buffer = Buffer::new(&mut font_system, Metrics::new(12.0, 14.0));

        Self {
            font_system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            buffer: dummy_buffer,
            text: String::new(),
            color: Color::rgb(255, 255, 255),
            position: (0.0, 0.0),
        }
    }

    pub fn set_text(&mut self, text: &str) {
        if text != self.text {
            let attrs = Attrs::new().color(self.color);
            self.buffer
                .set_text(&mut self.font_system, text, &attrs, Shaping::Advanced);
            self.text = text.to_string();
        }
    }

    pub fn set_color(&mut self, color: Color) {
        if color != self.color {
            self.color = color;
            let current_text = self.text.clone();
            self.set_text(&current_text);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.buffer.set_size(
            &mut self.font_system,
            Some(width as f32),
            Some(height as f32),
        );
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue, w: u32, h: u32) {
        self.viewport.update(
            queue,
            Resolution {
                width: w,
                height: h,
            },
        );
        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [TextArea {
                    buffer: &self.buffer,
                    left: self.position.0,
                    top: self.position.1,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: w as i32,
                        bottom: h as i32,
                    },
                    default_color: self.color,
                    custom_glyphs: &[],
                    scale: 1.0,
                }],
                &mut self.swash_cache,
            )
            .unwrap();
    }

    pub fn render<'rp>(&'rp self, pass: &mut wgpu::RenderPass<'rp>) {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .unwrap();
    }
}

pub struct TextBuilder<'a> {
    renderer: &'a mut Text,
    text: &'a str,
    position: (f32, f32),
    size: f32,
    color: Color,
}

impl<'a> TextBuilder<'a> {
    pub fn new(renderer: &'a mut Text, text: &'a str) -> Self {
        Self {
            renderer,
            text,
            position: (0.0, 0.0),
            size: 12.0,
            color: Color::rgb(255, 255, 255),
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

    pub fn color(mut self, color: wgpu::Color) -> Self {
        self.color = Color::rgba(
            (color.r * 255.0).round() as u8,
            (color.g * 255.0).round() as u8,
            (color.b * 255.0).round() as u8,
            (color.a * 255.0).round() as u8,
        );
        self
    }
}

impl<'a> Drop for TextBuilder<'a> {
    fn drop(&mut self) {
        let mut buffer = Buffer::new(&mut self.renderer.font_system, Metrics::new(self.size, 1.0));

        let attrs = Attrs::new().color(self.color);
        buffer.set_text(
            &mut self.renderer.font_system,
            self.text,
            &attrs,
            Shaping::Advanced,
        );

        self.renderer.buffer = buffer;
        self.renderer.text = self.text.to_string();
        self.renderer.color = self.color;
        self.renderer.position = self.position;
    }
}
