use super::{camera::Camera, renderer::Renderer, vertex::Vertex};
use crate::Color;
use glam::{Vec2, vec2};

fn rotate_and_transform(
    pos: Vec2,
    center: Vec2,
    angle: f32,
    renderer: &Renderer,
    camera: &Camera,
) -> [f32; 2] {
    let delta = pos - center;
    let (sin, cos) = angle.sin_cos();
    let rotated = vec2(cos * delta.x - sin * delta.y, sin * delta.x + cos * delta.y) + center;

    transform_to(renderer, camera, rotated)
}

fn transform_to(renderer: &Renderer, camera: &Camera, pos: Vec2) -> [f32; 2] {
    let screen = camera.world_to_screen(pos, renderer.screen_width(), renderer.screen_height());
    renderer.to_ndc(screen.x, screen.y)
}

#[derive(Clone, Copy)]
pub enum Anchor {
    Center,
    TopLeft,
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
        let half = self.size / 2.0;
        let points = match self.anchor {
            Anchor::TopLeft => [
                self.position + vec2(half, 0.0),
                self.position + vec2(0.0, self.size),
                self.position + vec2(self.size, self.size),
            ],
            Anchor::Center => [
                self.position + vec2(0.0, -half),
                self.position + vec2(-half, half),
                self.position + vec2(half, half),
            ],
        };

        let verts: Vec<_> = points
            .iter()
            .map(|&p| {
                Vertex::new(
                    transform_to(self.renderer, self.camera, p),
                    self.color.into(),
                    [-1.0, -1.0],
                )
            })
            .collect();

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
            anchor: Anchor::Center,
            position: Vec2::ZERO,
            size: vec2(64.0, 64.0),
            rotation: 0.0,
            color: Color::WHITE,
            tex_coords: [[-1.0, -1.0]; 4],
            tex_idx: 0,
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

    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.size = vec2(w, h);
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
        let half_size = self.size * 0.5;
        let corners = match self.anchor {
            Anchor::TopLeft => [
                self.position,
                self.position + vec2(self.size.x, 0.0),
                self.position + self.size,
                self.position + vec2(0.0, self.size.y),
            ],
            Anchor::Center => [
                self.position + vec2(-half_size.x, -half_size.y),
                self.position + vec2(half_size.x, -half_size.y),
                self.position + vec2(half_size.x, half_size.y),
                self.position + vec2(-half_size.x, half_size.y),
            ],
        };

        let vertices: Vec<_> = corners
            .iter()
            .zip(self.tex_coords.iter())
            .map(|(&pos, &uv)| {
                Vertex::new(
                    rotate_and_transform(
                        pos,
                        self.position,
                        self.rotation,
                        self.renderer,
                        self.camera,
                    ),
                    self.color.into(),
                    uv,
                )
            })
            .collect();

        self.renderer
            .submit(&vertices, &[0, 1, 2, 2, 3, 0], self.tex_idx);
    }
}
