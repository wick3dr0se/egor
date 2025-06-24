use glam::{Mat2, Vec2, vec2};

use crate::{Color, PrimitiveBatch, math::Rect, vertex::Vertex};

// Anchor point options for positioning primitives
///  
/// - `TopLeft`: Position is at the rectangle’s top-left corner (default)
/// - `Center`: Position is at the rectangle’s center
pub enum Anchor {
    Center,
    TopLeft,
}

/// Builder for drawing rectangles with configurable  
/// position, size, rotation, color, texture, anchor, etc
///
/// Constructed via [`crate::Graphics::rect()`]  
/// The rectangle is automatically submitted when this builder is dropped  
/// No explicit "finalize" call is needed
///
/// Use method chaining to customize before `Drop`:
/// Similar to Rust's RAII pattern (<https://rust-unofficial.github.io/patterns/patterns/behavioural/RAII.html>)
/// ```
/// graphics.rect()
///     .at(vec2(100.0, 50.0))
///     .size(vec2(200.0, 100.0))
///     .color(Color::RED)
///     .rotate(std::f32::consts::PI / 4.0);
/// ```
pub struct RectangleBuilder<'a> {
    batch: &'a mut PrimitiveBatch,
    anchor: Anchor,
    position: Vec2,
    size: Vec2,
    rotation: f32,
    color: Color,
    tex_coords: [[f32; 2]; 4],
    tex_id: usize,
}

/// Builds a rectangle with configurable position, size, color, anchor, rotation, & texture
impl<'a> RectangleBuilder<'a> {
    pub(crate) fn new(batch: &'a mut PrimitiveBatch) -> Self {
        Self {
            batch,
            anchor: Anchor::TopLeft,
            position: Vec2::ZERO,
            size: vec2(64.0, 64.0),
            rotation: 0.0,
            color: Color::WHITE,
            tex_coords: [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            tex_id: usize::MAX,
        }
    }

    /// Sets the position & size from a [`Rect`].
    pub fn with(mut self, rect: &Rect) -> Self {
        self.position = rect.position;
        self.size = rect.size;
        self
    }

    /// Sets the anchor point of the rectangle  
    /// Defaults to [`Anchor::TopLeft`].
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets the world-space position of the rectangle
    pub fn at(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Sets the size of the rectangle
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    /// Sets the color of the rectangle
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the rotation (in radians) around the rectangle's center
    pub fn rotate(mut self, angle: f32) -> Self {
        self.rotation = angle;
        self
    }

    /// Sets the texture ID for the rectangle
    pub fn texture(mut self, id: usize) -> Self {
        self.tex_id = id;
        self
    }

    /// Sets custom UV coordinates
    /// Defaults to covering the full texture ((0,0) - (1,1))
    pub fn uv(mut self, coords: [[f32; 2]; 4]) -> Self {
        self.tex_coords = coords;
        self
    }
}

impl Drop for RectangleBuilder<'_> {
    fn drop(&mut self) {
        let offset = match self.anchor {
            Anchor::TopLeft => Vec2::ZERO,
            Anchor::Center => -self.size / 2.0,
        };
        let top_left = self.position + offset;
        let rect = Rect::new(top_left, self.size);
        let rot = Mat2::from_angle(self.rotation);
        let verts: Vec<_> = rect
            .corners()
            .iter()
            .zip(self.tex_coords.iter())
            .map(|(&corner, &uv)| {
                let rotated = rot * (corner - rect.center()) + rect.center();
                Vertex::new(rotated.into(), self.color, uv)
            })
            .collect();

        self.batch.push(&verts, &[0, 1, 2, 2, 3, 0], self.tex_id);
    }
}
