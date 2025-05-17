use wgpu::Color;

use crate::camera::Camera;

use super::{Renderer, vertex::Vertex};

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
}

impl<'a> Drop for TriangleBuilder<'a> {
    fn drop(&mut self) {
        let [x, y] = self.position;
        let size = self.size;
        let half = size / 2.0;
        let (a, b, c) = match self.anchor {
            Anchor::TopLeft => ([x + half, y], [x, y + size], [x + size, y + size]),
            Anchor::Center => ([x, y - half], [x - half, y + half], [x + half, y + half]),
        };

        self.renderer.submit_geometry(
            &[
                Vertex::new(
                    transform_to(self.renderer, self.camera, a[0], a[1]),
                    self.color,
                    [-1.0, -1.0],
                ),
                Vertex::new(
                    transform_to(self.renderer, self.camera, b[0], b[1]),
                    self.color,
                    [-1.0, -1.0],
                ),
                Vertex::new(
                    transform_to(self.renderer, self.camera, c[0], c[1]),
                    self.color,
                    [-1.0, -1.0],
                ),
            ],
            &[0, 1, 2],
            0,
        );
    }
}

pub struct RectangleBuilder<'a> {
    renderer: &'a mut Renderer,
    camera: &'a Camera,
    anchor: Anchor,
    position: [f32; 2],
    size: [f32; 2],
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

    pub fn texture(mut self, idx: usize) -> Self {
        self.tex_idx = idx;
        self.tex_coords = [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
        self
    }
}

impl<'a> Drop for RectangleBuilder<'a> {
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

        self.renderer.submit_geometry(
            &[
                Vertex::new(
                    transform_to(self.renderer, self.camera, a, b),
                    self.color,
                    self.tex_coords[0],
                ),
                Vertex::new(
                    transform_to(self.renderer, self.camera, c, b),
                    self.color,
                    self.tex_coords[1],
                ),
                Vertex::new(
                    transform_to(self.renderer, self.camera, c, d),
                    self.color,
                    self.tex_coords[2],
                ),
                Vertex::new(
                    transform_to(self.renderer, self.camera, a, d),
                    self.color,
                    self.tex_coords[3],
                ),
            ],
            &[0, 1, 2, 2, 3, 0],
            self.tex_idx,
        );
    }
}
