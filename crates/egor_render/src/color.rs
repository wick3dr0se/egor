use color::{AlphaColor, LinearSrgb};
use glyphon::cosmic_text;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    internal: AlphaColor<LinearSrgb>,
}

impl Color {
    pub const fn new(components: [f32; 4]) -> Self {
        Self {
            internal: AlphaColor::new(components),
        }
    }
    pub fn components(&self) -> [f32; 4] {
        self.internal.components
    }
}

impl Color {
    pub const BLACK: Color = Self {
        internal: AlphaColor::BLACK,
    };
    pub const WHITE: Color = Self {
        internal: AlphaColor::WHITE,
    };
    pub const TRANSPARENT: Color = Self {
        internal: AlphaColor::TRANSPARENT,
    };
    pub const RED: Color = Self {
        internal: AlphaColor::new([1., 0., 0., 1.]),
    };
    pub const GREEN: Color = Self {
        internal: AlphaColor::new([0., 1., 0., 1.]),
    };
    pub const BLUE: Color = Self {
        internal: AlphaColor::new([0., 0., 1., 1.]),
    };
}

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

impl From<Color> for cosmic_text::Color {
    fn from(value: Color) -> Self {
        let rgba_8 = value.internal.to_rgba8();
        cosmic_text::Color::rgba(rgba_8.r, rgba_8.g, rgba_8.b, rgba_8.a)
    }
}
