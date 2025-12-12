use egor_render::{
    color::Color,
    math::Vec2,
    text::{Attrs, Buffer, Metrics, Shaping, TextEntry, TextRenderer},
};

/// Builder for a single line of text to be drawn on screen
///
/// Automatically pushed to the text renderer when dropped.  
/// This must be constructed **before** `TextRenderer::prepare()` is called.
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
            position: Vec2::new(0.0, 0.0),
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
            Shaping::Advanced,
        );

        self.renderer.push_entry(TextEntry {
            buffer,
            position: self.position,
        });
    }
}
