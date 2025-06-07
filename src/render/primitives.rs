use wgpu::Color;

use super::{camera::Camera, renderer::Renderer, vertex::Vertex};

fn rotate_and_transform(
    x: f32,
    y: f32,
    cx: f32,
    cy: f32,
    angle: f32,
    renderer: &Renderer,
    camera: &Camera,
) -> [f32; 2] {
    let (dx, dy) = (x - cx, y - cy);
    let (sin, cos) = angle.sin_cos();
    let rx = cos * dx - sin * dy + cx;
    let ry = sin * dx + cos * dy + cy;
    transform_to(renderer, camera, rx, ry)
}

fn transform_to(renderer: &Renderer, camera: &Camera, x: f32, y: f32) -> [f32; 2] {
    let (screen_x, screen_y) =
        camera.world_to_screen(x, y, renderer.screen_width(), renderer.screen_height());
    renderer.to_ndc(screen_x, screen_y)
}

pub enum Anchor {
    Center,
    TopLeft,
}

pub struct TriangleBuilder<'a> {
    renderer: &'a mut Renderer,
    camera: &'a Camera,
    anchor: Anchor,
    position: [f32; 2],
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
            position: [0.0, 0.0],
            size: 64.0,
            rotation: 0.0,
            color: Color::RED,
        }
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = [x, y];
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
        let [x, y] = self.position;
        let size = self.size;
        let half = size / 2.0;

        let points = match self.anchor {
            Anchor::TopLeft => [[x + half, y], [x, y + size], [x + size, y + size]],
            Anchor::Center => [[x, y - half], [x - half, y + half], [x + half, y + half]],
        };

        let verts: Vec<Vertex> = points
            .iter()
            .map(|[px, py]| {
                Vertex::new(
                    transform_to(self.renderer, self.camera, *px, *py),
                    self.color,
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
    position: [f32; 2],
    size: [f32; 2],
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
            position: [0.0, 0.0],
            size: [64.0, 64.0],
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

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = [x, y];
        self
    }

    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.size = [w, h];
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
        let [x, y] = self.position;
        let [w, h] = self.size;
        let (a, b, c, d) = match self.anchor {
            Anchor::TopLeft => (x, y, x + w, y + h),
            Anchor::Center => {
                let (hw, hh) = (w / 2.0, h / 2.0);
                (x - hw, y - hh, x + hw, y + hh)
            }
        };
        let verts = [
            (a, b, self.tex_coords[0]),
            (c, b, self.tex_coords[1]),
            (c, d, self.tex_coords[2]),
            (a, d, self.tex_coords[3]),
        ];
        let vertices: Vec<_> = verts
            .iter()
            .map(|&(vx, vy, uv)| {
                Vertex::new(
                    rotate_and_transform(vx, vy, x, y, self.rotation, self.renderer, self.camera),
                    self.color,
                    uv,
                )
            })
            .collect();

        self.renderer
            .submit(&vertices, &[0, 1, 2, 2, 3, 0], self.tex_idx);
    }
}
