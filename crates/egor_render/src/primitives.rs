use glam::{Mat2, Vec2, vec2};

use super::{Color, GeometrySink, math::Rect, vertex::Vertex};

pub enum Anchor {
    Center,
    TopLeft,
}

pub struct RectangleBuilder<'a> {
    sink: &'a mut dyn GeometrySink,
    anchor: Anchor,
    position: Vec2,
    size: Vec2,
    rotation: f32,
    color: Color,
    tex_coords: [[f32; 2]; 4],
    tex_idx: usize,
}

impl<'a> RectangleBuilder<'a> {
    pub(crate) fn new(sink: &'a mut dyn GeometrySink) -> Self {
        Self {
            sink,
            anchor: Anchor::TopLeft,
            position: Vec2::ZERO,
            size: vec2(64.0, 64.0),
            rotation: 0.0,
            color: Color::WHITE,
            tex_coords: [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            tex_idx: usize::MAX,
        }
    }

    pub fn with(mut self, rect: &Rect) -> Self {
        self.position = rect.position;
        self.size = rect.size;
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn at(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn rotation(mut self, angle: f32) -> Self {
        self.rotation = angle;
        self
    }

    pub fn texture(mut self, idx: usize) -> Self {
        self.tex_idx = idx;
        self
    }

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
                Vertex::new(self.sink.world_to_ndc(rotated), self.color.into(), uv)
            })
            .collect();

        self.sink.queue(&verts, &[0, 1, 2, 2, 3, 0], self.tex_idx);
    }
}
