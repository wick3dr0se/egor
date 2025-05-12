use wgpu::Color;

use super::{Renderer, vertex::Vertex};

pub enum Anchor {
    Center,
    TopLeft,
}

pub struct TriangleBuilder<'a> {
    renderer: &'a mut Renderer,
    anchor: Anchor,
    position: [f32; 2],
    size: f32,
    color: Color,
}

impl<'a> TriangleBuilder<'a> {
    fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            renderer,
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
                Vertex::new(self.renderer.to_ndc(a[0], a[1]), self.color),
                Vertex::new(self.renderer.to_ndc(b[0], b[1]), self.color),
                Vertex::new(self.renderer.to_ndc(c[0], c[1]), self.color),
            ],
            &[0, 1, 2],
        );
    }
}

pub struct RectangleBuilder<'a> {
    renderer: &'a mut Renderer,
    anchor: Anchor,
    position: [f32; 2],
    size: [f32; 2],
    color: Color,
}

impl<'a> RectangleBuilder<'a> {
    fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            renderer,
            anchor: Anchor::Center,
            position: [0.0, 0.0],
            size: [64.0, 64.0],
            color: Color::BLUE,
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
                Vertex::new(self.renderer.to_ndc(a, b), self.color),
                Vertex::new(self.renderer.to_ndc(c, b), self.color),
                Vertex::new(self.renderer.to_ndc(c, d), self.color),
                Vertex::new(self.renderer.to_ndc(a, d), self.color),
            ],
            &[0, 1, 2, 2, 3, 0],
        );
    }
}

pub struct GraphicsContext<'a> {
    pub(crate) renderer: &'a mut Renderer,
}

impl<'a> GraphicsContext<'a> {
    pub fn tri(&mut self) -> TriangleBuilder {
        TriangleBuilder::new(self.renderer)
    }

    pub fn rect(&mut self) -> RectangleBuilder {
        RectangleBuilder::new(self.renderer)
    }

    pub fn clear(&mut self, color: Color) {
        self.renderer.clear(color);
    }

    pub fn screen_size(&self) -> [f32; 2] {
        [
            self.renderer.screen_width() as f32,
            self.renderer.screen_height() as f32,
        ]
    }
}
