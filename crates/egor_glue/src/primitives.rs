use egor_render::{GeometryBatch, vertex::Vertex};
use glam::{Mat2, Vec2, vec2};

use crate::{color::Color, math::Rect};

#[derive(Default)]
pub(crate) struct PrimitiveBatch {
    geometry: Vec<(usize, GeometryBatch)>,
}

impl PrimitiveBatch {
    // Add verts & indices to batch, preserving submission order & batching consecutive geometry per texture
    pub(crate) fn push(&mut self, verts: &[Vertex], indices: &[u16], texture_id: usize) {
        if let Some((last_texture, last_batch)) = self.geometry.last_mut()
            && *last_texture == texture_id
        {
            last_batch.push(verts, indices);
            return;
        }

        let mut batch = GeometryBatch::default();
        batch.push(verts, indices);
        self.geometry.push((texture_id, batch));
    }

    pub(crate) fn take(&mut self) -> Vec<(usize, GeometryBatch)> {
        std::mem::take(&mut self.geometry)
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
        let mut indices = Vec::new();

        // Convex fan triangulation
        for i in 1..points.len() - 1 {
            indices.push(0);
            indices.push(i as u16);
            indices.push((i + 1) as u16);
        }
        self.batch.push(&verts, &indices, 0);
    }
}

/// Builder for stroked paths (polylines)
///
/// Expands each line segment into quad (triangle) geometry on `Drop`
pub struct PolylineBuilder<'a> {
    batch: &'a mut PrimitiveBatch,
    position: Vec2,
    rotation: f32,
    points: Vec<Vec2>,
    thickness: f32,
    color: Color,
    closed: bool,
}

impl<'a> PolylineBuilder<'a> {
    pub(crate) fn new(batch: &'a mut PrimitiveBatch) -> Self {
        Self {
            batch,
            position: Vec2::ZERO,
            rotation: 0.0,
            points: vec![vec2(0.0, 0.0), vec2(10.0, 0.0)],
            thickness: 1.0,
            color: Color::WHITE,
            closed: false,
        }
    }
    /// Sets the world-space position of the polyline
    pub fn at(mut self, pos: Vec2) -> Self {
        self.position = pos;
        self
    }
    /// Sets rotation in radians around the polyline origin
    pub fn rotate(mut self, angle: f32) -> Self {
        self.rotation = angle;
        self
    }
    /// Sets the points of the polyline  
    /// At least two points are required to generate geometry
    pub fn points(mut self, pts: &[Vec2]) -> Self {
        self.points.clear();
        self.points.extend_from_slice(pts);
        self
    }
    /// Sets the stroke thickness in world units
    pub fn thickness(mut self, t: f32) -> Self {
        self.thickness = t.max(0.001);
        self
    }
    /// Sets the color of the polyline
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    /// When enabled, the last point is connected back to the first
    pub fn closed(mut self, closed: bool) -> Self {
        self.closed = closed;
        self
    }
}

impl Drop for PolylineBuilder<'_> {
    fn drop(&mut self) {
        if self.points.len() < 2 {
            return;
        }

        let rot = Mat2::from_angle(self.rotation);
        let mut verts = Vec::new();
        let mut indices = Vec::new();
        let mut add_segment = |a: Vec2, b: Vec2| {
            let dir = (b - a).normalize();
            let normal = vec2(-dir.y, dir.x) * (self.thickness * 0.5);
            let base = verts.len() as u16;
            let p0 = rot * (a + normal) + self.position;
            let p1 = rot * (a - normal) + self.position;
            let p2 = rot * (b - normal) + self.position;
            let p3 = rot * (b + normal) + self.position;
            let color = self.color.components();

            verts.extend_from_slice(&[
                Vertex::new(p0.into(), color, [0.0, 0.0]),
                Vertex::new(p1.into(), color, [0.0, 0.0]),
                Vertex::new(p2.into(), color, [0.0, 0.0]),
                Vertex::new(p3.into(), color, [0.0, 0.0]),
            ]);

            indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
        };

        for i in 0..self.points.len() - 1 {
            add_segment(self.points[i], self.points[i + 1]);
        }

        if self.closed {
            add_segment(*self.points.last().unwrap(), self.points[0]);
        }

        self.batch.push(&verts, &indices, 0);
    }
}
