use super::{camera::Camera, renderer::Renderer, vertex::Vertex};
use crate::{Color, render::math::Rect};
use glam::{Mat2, Vec2, vec2};

pub enum Anchor {
    Center,
    TopLeft,
}

fn transform(renderer: &Renderer, camera: &Camera, position: Vec2) -> [f32; 2] {
    let Vec2 { x, y } = camera.world_to_screen(position, renderer.surface_size().into());
    renderer.to_ndc(x, y)
}

pub struct TriangleBuilder<'a> {
    renderer: &'a mut Renderer,
    camera: &'a Camera,
    anchor: Anchor,
    position: Vec2,
    size: f32,
    rotation: f32,
    color: Color,
}

impl<'a> TriangleBuilder<'a> {
    pub fn new(renderer: &'a mut Renderer, camera: &'a Camera) -> Self {
        Self {
            renderer,
            camera,
            anchor: Anchor::Center,
            position: Vec2::ZERO,
            size: 64.0,
            rotation: 0.0,
            color: Color::RED,
        }
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn at(mut self, position: Vec2) -> Self {
        self.position = position;
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

    pub fn rotation(mut self, angle: f32) -> Self {
        self.rotation = angle;
        self
    }
}

impl Drop for TriangleBuilder<'_> {
    fn drop(&mut self) {
        let offset = match self.anchor {
            Anchor::TopLeft => Vec2::ZERO,
            Anchor::Center => vec2(0.0, -self.size / 3.0),
        };
        let p0 = self.position + offset + vec2(-self.size / 2.0, self.size / 3.0);
        let p1 = self.position + offset + vec2(self.size / 2.0, self.size / 3.0);
        let p2 = self.position + offset + vec2(0.0, -2.0 * self.size / 3.0);
        let center = self.position + offset;
        let rot = Mat2::from_angle(self.rotation);
        let verts = [p0, p1, p2]
            .iter()
            .map(|&p| {
                let rotated = rot * (p - center) + center;
                Vertex::new(
                    transform(self.renderer, self.camera, rotated),
                    self.color.into(),
                    [-1.0, -1.0],
                )
            })
            .collect::<Vec<_>>();

        self.renderer.submit(&verts, &[0, 1, 2], 0);
    }
}

pub struct RectangleBuilder<'a> {
    renderer: &'a mut Renderer,
    camera: &'a Camera,
    anchor: Anchor,
    position: Vec2,
    size: Vec2,
    rotation: f32,
    color: Color,
    tex_coords: [[f32; 2]; 4],
    tex_idx: usize,
}

impl<'a> RectangleBuilder<'a> {
    pub fn new(renderer: &'a mut Renderer, camera: &'a Camera) -> Self {
        Self {
            renderer,
            camera,
            anchor: Anchor::TopLeft,
            position: Vec2::ZERO,
            size: vec2(64.0, 64.0),
            rotation: 0.0,
            color: Color::WHITE,
            tex_coords: [[-1.0, -1.0]; 4],
            tex_idx: 0,
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

        if self.tex_coords == [[-1.0, -1.0]; 4] {
            self.tex_coords = [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
        }

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
                Vertex::new(
                    transform(self.renderer, self.camera, rotated),
                    self.color.into(),
                    uv,
                )
            })
            .collect();

        self.renderer
            .submit(&verts, &[0, 1, 2, 2, 3, 0], self.tex_idx);
    }
}
