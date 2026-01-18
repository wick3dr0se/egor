use egor_render::{
    GeometryBatch,
    math::{Mat2, Rect, Vec2, vec2},
    vertex::Vertex,
};

use crate::color::Color;

#[derive(Default)]
pub(crate) struct PrimitiveBatch {
    geometry: Vec<(usize, GeometryBatch)>,
}

impl PrimitiveBatch {
    // Add verts & indices to batch, preserving submission order & batching consecutive geometry per texture
    pub(crate) fn push(&mut self, verts: &[Vertex], indices: &[u16], texture_id: usize) {
        if let Some((last_texture, last_batch)) = self.geometry.last_mut() {
            if *last_texture == texture_id {
                last_batch.push(verts, indices);
                return;
            }
        }

        let mut batch = GeometryBatch::default();
        batch.push(verts, indices);
        self.geometry.push((texture_id, batch));
    }

    pub(crate) fn take(&mut self) -> Vec<(usize, GeometryBatch)> {
        std::mem::take(&mut self.geometry)
    }
}

/// Builder for polygons, triangles, circles, n-gons. Drawn on `Drop`
pub struct PolygonBuilder<'a> {
    batch: &'a mut PrimitiveBatch,
    position: Vec2,
    rotation: f32,
    points: Vec<Vec2>,
    radius: f32,
    segments: usize,
    color: Color,
}

impl<'a> PolygonBuilder<'a> {
    pub(crate) fn new(batch: &'a mut PrimitiveBatch) -> Self {
        Self {
            batch,
            position: Vec2::ZERO,
            rotation: 0.0,
            points: Vec::new(),
            radius: 10.0,
            segments: 3,
            color: Color::WHITE,
        }
    }
    /// Sets the world-space position of the polygon
    pub fn at(mut self, pos: Vec2) -> Self {
        self.position = pos;
        self
    }
    /// Sets rotation in radians around the polygon's origin (default center)
    pub fn rotate(mut self, angle: f32) -> Self {
        self.rotation = angle;
        self
    }
    /// Set explicit points for the polygon
    pub fn points(mut self, pts: &[Vec2]) -> Self {
        self.points.clear();
        self.points.extend_from_slice(pts);
        self
    }
    /// Set radius for a circle or regular n-gon
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = r;
        self
    }
    /// Set number of segments for circles/n-gons
    pub fn segments(mut self, segments: usize) -> Self {
        self.segments = segments.max(3);
        self
    }
    /// Sets the color of the polygon
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Drop for PolygonBuilder<'_> {
    fn drop(&mut self) {
        let points: Vec<Vec2> = if !self.points.is_empty() {
            self.points.clone()
        } else {
            let r = self.radius;
            (0..self.segments)
                .map(|i| {
                    let t = i as f32 / self.segments as f32 * std::f32::consts::TAU;
                    Vec2::new(t.cos(), t.sin()) * r
                })
                .collect()
        };

        let rot = Mat2::from_angle(self.rotation);
        let verts: Vec<Vertex> = points
            .iter()
            .map(|p| {
                let world = rot * p + self.position;
                Vertex::new(world.into(), self.color.components(), [0.0, 0.0])
            })
            .collect();

        // Convex fan triangulation
        let mut indices = Vec::new();
        for i in 1..points.len() - 1 {
            indices.push(0);
            indices.push(i as u16);
            indices.push((i + 1) as u16);
        }

        self.batch.push(&verts, &indices, 0);
    }
}

/// Common anchor options
pub enum Anchor {
    Center,
    TopLeft,
}

/// Builder for (textured) rectangles, drawn on `Drop`
pub struct RectangleBuilder<'a> {
    batch: &'a mut PrimitiveBatch,
    anchor: Anchor,
    position: Vec2,
    size: Vec2,
    rotation: f32,
    color: Color,
    uvs: [[f32; 2]; 4],
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
            uvs: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            tex_id: 0,
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
    pub fn at(mut self, position: impl Into<Vec2>) -> Self {
        self.position = position.into();
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
    /// Sets rotation (in radians) around the rectangle's center
    /// 0 radians points up (positive Y), increasing clockwise
    pub fn rotate(mut self, angle: f32) -> Self {
        self.rotation = angle + std::f32::consts::FRAC_PI_2;
        self
    }
    /// Sets the texture ID for the rectangle
    pub fn texture(mut self, id: usize) -> Self {
        self.tex_id = id;
        self
    }
    /// Custom UV coordinates
    /// Defaults to covering the full texture ((0,0) - (1,1))
    pub fn uv(mut self, coords: [[f32; 2]; 4]) -> Self {
        self.uvs = coords;
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
            .zip(self.uvs.iter())
            .map(|(&corner, &uv)| {
                let world = rot * (corner - rect.center()) + rect.center();
                Vertex::new(world.into(), self.color.components(), uv)
            })
            .collect();

        self.batch.push(&verts, &[0, 1, 2, 2, 3, 0], self.tex_id);
    }
}
