use color::{AlphaColor, LinearSrgb};
use glyphon::cosmic_text;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    inner: AlphaColor<LinearSrgb>,
}

impl Color {
    /// Create a new Color from RGBA components in [0..1]
    pub const fn new(components: [f32; 4]) -> Self {
        Self {
            inner: AlphaColor::new(components),
        }
    }

    /// Get raw RGBA components
    pub fn components(&self) -> [f32; 4] {
        self.inner.components
    }
}

impl Color {
    pub const BLACK: Color = Self {
        inner: AlphaColor::BLACK,
    };
    pub const WHITE: Color = Self {
        inner: AlphaColor::WHITE,
    };
    pub const TRANSPARENT: Color = Self {
        inner: AlphaColor::TRANSPARENT,
    };
    pub const RED: Color = Self {
        inner: AlphaColor::new([1., 0., 0., 1.]),
    };
    pub const GREEN: Color = Self {
        inner: AlphaColor::new([0., 1., 0., 1.]),
    };
    pub const BLUE: Color = Self {
        inner: AlphaColor::new([0., 0., 1., 1.]),
    };
}

// Convert Color to wgpu::Color (f64 RGBA)
impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        let [r, g, b, a] = value.components();
        wgpu::Color {
            r: r as f64,
            g: g as f64,
            b: b as f64,
            a: a as f64,
        }
    }
}

// Convert Color to cosmic_text::Color (u8 RGBA)
impl From<Color> for cosmic_text::Color {
    fn from(value: Color) -> Self {
        let [r, g, b, a] = value.inner.to_rgba8().to_u8_array();
        cosmic_text::Color::rgba(r, g, b, a)
    }
}
