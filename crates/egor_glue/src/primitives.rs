use crate::{color::Color, math::Rect};
use egor_render::{GeometryBatch, vertex::Vertex};
use glam::{Mat2, Vec2, vec2};
use lyon::geom::euclid::Point2D;
use lyon::geom::{Box2D, Point};
use lyon::path::Winding;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::*;

const MIN_THICKNESS: f32 = 0.001;

#[derive(Default)]
struct BatchEntry {
    texture_id: Option<usize>,
    shader_id: Option<usize>,
    geometry: GeometryBatch,
}

#[derive(Default)]
pub struct PrimitiveBatch {
    batches: Vec<BatchEntry>,
}

impl PrimitiveBatch {
    /// Allocates space for vertices & indices in the correct batch for `texture_id` + `shader_id`
    pub(crate) fn allocate(
        &mut self,
        vert_count: usize,
        idx_count: usize,
        texture_id: Option<usize>,
        shader_id: Option<usize>,
    ) -> Option<(&mut [Vertex], &mut [u16], u16)> {
        if let Some(i) = self.batches.iter().position(|e| {
            e.texture_id == texture_id
                && e.shader_id == shader_id
                && !e.geometry.would_overflow(vert_count, idx_count)
        }) {
            return self.batches[i].geometry.try_allocate(vert_count, idx_count);
        }

        self.batches.push(BatchEntry {
            texture_id,
            shader_id,
            geometry: GeometryBatch::default(),
        });
        self.batches
            .last_mut()
            .unwrap()
            .geometry
            .try_allocate(vert_count, idx_count)
    }

    /// Moves all batch entries out, consuming their geometry.
    /// Used for ephemeral paths (offscreen rendering) where batch reuse isn't needed
    pub(crate) fn take(&mut self) -> Vec<(Option<usize>, Option<usize>, GeometryBatch)> {
        std::mem::take(&mut self.batches)
            .into_iter()
            .map(|entry| (entry.texture_id, entry.shader_id, entry.geometry))
            .collect()
    }

    /// Iterates over active batch entries for drawing.
    /// Returns (texture_id, shader_id, &mut GeometryBatch) for each entry
    pub(crate) fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (Option<usize>, Option<usize>, &mut GeometryBatch)> {
        self.batches
            .iter_mut()
            .map(|e| (e.texture_id, e.shader_id, &mut e.geometry))
    }

    /// Clears CPU-side vertex/index data from all batches but retains the
    /// `BatchEntry` objects and their GPU buffers for reuse next frame
    pub(crate) fn reset(&mut self) {
        for batch in &mut self.batches {
            batch.geometry.clear();
        }
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
    shader_id: Option<usize>,
    anchor: Anchor,
    position: Vec2,
    size: Vec2,
    rotation: f32,
    color: Color,
    uvs: [[f32; 2]; 4],
    tex_id: Option<usize>,
}

/// Builds a rectangle with configurable position, size, color, anchor, rotation, & texture
impl<'a> RectangleBuilder<'a> {
    pub(crate) fn new(batch: &'a mut PrimitiveBatch, shader_id: Option<usize>) -> Self {
        Self {
            batch,
            shader_id,
            anchor: Anchor::TopLeft,
            position: Vec2::ZERO,
            size: vec2(64.0, 64.0),
            rotation: 0.0,
            color: Color::WHITE,
            uvs: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            tex_id: None,
        }
    }
    /// Sets the position & size from a [`Rect`].
    pub fn with(mut self, rect: &Rect) -> Self {
        self.position = rect.position;
        self.size = rect.size;
        self
    }
    /// Sets the anchor point of the rectangle.
    /// Defaults to [`Anchor::TopLeft`]
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
        self.tex_id = Some(id);
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

        let corners = rect.corners();
        let center = rect.center();
        let color = self.color.components();

        if let Some((verts, indices, base)) = self.batch.allocate(4, 6, self.tex_id, self.shader_id)
        {
            for i in 0..4 {
                let world = rot * (corners[i] - center) + center;
                verts[i] = Vertex::new(world.into(), color, self.uvs[i]);
            }

            indices.copy_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
        }
    }
}

/// Builder for polygons, triangles, circles, n-gons. Drawn on `Drop`
pub struct PolygonBuilder<'a> {
    batch: &'a mut PrimitiveBatch,
    shader_id: Option<usize>,
    position: Vec2,
    rotation: f32,
    points: Vec<Vec2>,
    radius: f32,
    segments: usize,
    color: Color,
}

impl<'a> PolygonBuilder<'a> {
    pub(crate) fn new(batch: &'a mut PrimitiveBatch, shader_id: Option<usize>) -> Self {
        Self {
            batch,
            shader_id,
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
        let center = self.position;
        let color = self.color.components();

        let vert_count = points.len();
        let idx_count = (points.len().saturating_sub(2)) * 3;

        if let Some((verts, indices, base)) =
            self.batch
                .allocate(vert_count, idx_count, None, self.shader_id)
        {
            for (i, p) in points.iter().enumerate() {
                let world = rot * *p + center;
                verts[i] = Vertex::new(world.into(), color, [0.0, 0.0]);
            }

            // Convex fan triangulation
            for i in 0..points.len().saturating_sub(2) {
                let offset = i * 3;
                indices[offset] = base;
                indices[offset + 1] = base + (i as u16 + 1);
                indices[offset + 2] = base + (i as u16 + 2);
            }
        }
    }
}

/// Builder for stroked paths (polylines)
///
/// Expands each line segment into quad (triangle) geometry on `Drop`
pub struct PolylineBuilder<'a> {
    batch: &'a mut PrimitiveBatch,
    shader_id: Option<usize>,
    position: Vec2,
    rotation: f32,
    points: Vec<Vec2>,
    thickness: f32,
    color: Color,
    closed: bool,
}

impl<'a> PolylineBuilder<'a> {
    pub(crate) fn new(batch: &'a mut PrimitiveBatch, shader_id: Option<usize>) -> Self {
        Self {
            batch,
            shader_id,
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
        self.thickness = t.max(MIN_THICKNESS);
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
        let n = self.points.len();
        if n < 2 {
            return;
        }

        let rot = Mat2::from_angle(self.rotation);
        let color = self.color.components();
        let segments = if self.closed { n } else { n - 1 };
        let vert_count = segments * 4;
        let idx_count = segments * 6;

        if let Some((verts, indices, mut base)) =
            self.batch
                .allocate(vert_count, idx_count, None, self.shader_id)
        {
            let mut vi = 0;
            let mut ii = 0;

            for s in 0..segments {
                let a = self.points[s];
                let b = self.points[(s + 1) % n]; // wraps if closed

                let dir = (b - a).normalize();
                let nrm = vec2(-dir.y, dir.x) * (self.thickness * 0.5);

                let p = [
                    rot * (a + nrm) + self.position,
                    rot * (a - nrm) + self.position,
                    rot * (b - nrm) + self.position,
                    rot * (b + nrm) + self.position,
                ];

                for &pos in &p {
                    verts[vi] = Vertex::new(pos.into(), color, [0.0, 0.0]);
                    vi += 1;
                }

                indices[ii..ii + 6].copy_from_slice(&[
                    base,
                    base + 1,
                    base + 2,
                    base + 2,
                    base + 3,
                    base,
                ]);
                ii += 6;
                base += 4;
            }
        }
    }
}

pub enum Shape {
    Path { steps: Vec<PathStep> },
    Rect { size: Vec2 },
    Circle { center: Vec2, radius: f32 },
}

#[derive(Copy, Clone, Debug)]
pub enum PathStep {
    Begin(Vec2),
    LineTo(Vec2),
    QuadBezierTo(Vec2, Vec2),
    CubicBezierTo(Vec2, Vec2, Vec2),
}

pub struct ShapeBuilder<'a> {
    batch: &'a mut PrimitiveBatch,
    shader_id: Option<usize>,
    position: Vec2,
    rotation: f32,
    scale: Vec2,
    thickness: f32,
    stroke_color: Option<Color>,
    fill_color: Option<Color>,
    shape: Option<Shape>,
}

impl<'a> ShapeBuilder<'a> {
    pub(crate) fn new(batch: &'a mut PrimitiveBatch, shader_id: Option<usize>) -> Self {
        Self {
            batch,
            shader_id,
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            thickness: 1.0,
            stroke_color: None,
            fill_color: None,
            shape: None,
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
    /// Sets the scale of the path
    pub fn scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }
    /// Sets the stroke thickness in world units
    pub fn thickness(mut self, t: f32) -> Self {
        self.thickness = t.max(MIN_THICKNESS);
        self
    }
    /// Sets the stroke color of the path
    pub fn stroke_color(mut self, color: Color) -> Self {
        self.stroke_color = Some(color);
        self
    }
    /// Sets the fill color of the path
    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }
    /// Sets the shape to be drawn
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = Some(shape);
        self
    }
}

impl Drop for ShapeBuilder<'_> {
    fn drop(&mut self) {
        let mut builder = Path::builder();

        if let Some(shape) = &self.shape {
            match shape {
                Shape::Path { steps } => {
                    for step in steps {
                        match step {
                            PathStep::Begin(v) => {
                                builder.begin(point(v.x, v.y));
                            }
                            PathStep::LineTo(v) => {
                                builder.line_to(point(v.x, v.y));
                            }
                            PathStep::QuadBezierTo(v1, v2) => {
                                builder.quadratic_bezier_to(point(v1.x, v1.y), point(v2.x, v2.y));
                            }
                            PathStep::CubicBezierTo(v1, v2, v3) => {
                                builder.cubic_bezier_to(
                                    point(v1.x, v1.y),
                                    point(v2.x, v2.y),
                                    point(v3.x, v3.y),
                                );
                            }
                        }
                    }

                    builder.end(true);
                }
                Shape::Rect { size } => {
                    builder.add_rectangle(
                        &Box2D::new(
                            Point2D::new(self.position.x, self.position.y),
                            Point2D::new(self.position.x + size.x, self.position.y + size.y),
                        ),
                        Winding::Positive,
                    );
                }
                Shape::Circle { center, radius } => {
                    builder.add_circle(Point::new(center.x, center.y), *radius, Winding::Positive);
                }
            }
        }

        let path = builder.build();
        let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();

        if let Some(fill_color) = self.fill_color {
            let mut tessellator = FillTessellator::new();
            {
                tessellator
                    .tessellate_path(
                        &path,
                        &FillOptions::default(),
                        &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                            let [x, y] = vertex.position().to_array();
                            Vertex {
                                position: [x, y],
                                color: fill_color.components(),
                                tex_coords: [0.0, 0.0],
                            }
                        }),
                    )
                    .unwrap();
            }
        }

        if let Some(stroke_color) = self.stroke_color {
            let mut tessellator = StrokeTessellator::new();
            {
                tessellator
                    .tessellate_path(
                        &path,
                        &StrokeOptions::default().with_line_width(self.thickness),
                        &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                            let [x, y] = vertex.position().to_array();
                            Vertex {
                                position: [x, y],
                                color: stroke_color.components(),
                                tex_coords: [0.0, 0.0],
                            }
                        }),
                    )
                    .unwrap();
            }
        }

        let rot = Mat2::from_angle(self.rotation);

        let vert_count = geometry.vertices.len();
        let idx_count = geometry.indices.len();

        if let Some((verts, indices, base)) =
            self.batch.allocate(vert_count, idx_count, None, self.shader_id)
        {
            let mut vi = 0;
            for mut vo in geometry.vertices {
                let mut p: Vec2 = vo.position.into();
                p = rot * (self.scale * p) + self.position;
                vo.position = p.to_array();

                verts[vi] = vo;
                vi += 1;
            }

            for i in 0..idx_count {
                indices[i] = base + geometry.indices[i];
            }
        }
    }
}
