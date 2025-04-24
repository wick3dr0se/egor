use super::{Renderer, vertex::Vertex};
pub use wgpu::Color;

pub fn color_to_f32(color: Color) -> [f32; 4] {
    [
        color.r as f32,
        color.g as f32,
        color.b as f32,
        color.a as f32,
    ]
}

pub struct Graphics<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> Graphics<'a> {
    pub fn new(renderer: &'a mut Renderer) -> Self {
        Self { renderer }
    }

    pub fn clear(&mut self, color: Color) {
        self.renderer.clear_color = color;
    }

    pub fn screen_size(&self) -> [f32; 2] {
        [self.renderer.screen_width(), self.renderer.screen_height()]
    }

    pub fn quad(&mut self) -> QuadBuilder {
        QuadBuilder::new(self.renderer)
    }

    pub fn circle(&mut self) -> CircleBuilder {
        CircleBuilder::new(self.renderer)
    }
}

pub struct QuadBuilder<'a> {
    renderer: &'a mut Renderer,
    position: [f32; 2],
    size: [f32; 2],
    tex_coords: [[f32; 2]; 4],
    tex_idx: usize,
    color: Color,
}

impl<'a> QuadBuilder<'a> {
    pub fn new(renderer: &'a mut Renderer) -> Self {
        let [x, y] = renderer.screen_center();
        Self {
            renderer,
            position: [x, y],
            size: [32.0, 32.0],
            tex_coords: [[-1.0, -1.0]; 4],
            tex_idx: 0,
            color: Color::WHITE,
        }
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

    pub fn draw(self) {
        let [x, y] = self.renderer.pixels_to_ndc(self.position);
        let [w, h] = self.renderer.pixels_to_ndc_scale(self.size);
        let color = color_to_f32(self.color);
        let vertices = [
            Vertex::new([x + w, y], self.tex_coords[0], color),
            Vertex::new([x, y], self.tex_coords[1], color),
            Vertex::new([x, y - h], self.tex_coords[2], color),
            Vertex::new([x + w, y - h], self.tex_coords[3], color),
        ];
        let indices = [0, 1, 2, 0, 2, 3];

        self.renderer
            .submit_geometry(&vertices, &indices, self.tex_idx);
    }
}

pub struct CircleBuilder<'a> {
    renderer: &'a mut Renderer,
    position: [f32; 2],
    radius: f32,
    segments: u16,
    color: Color,
}

impl<'a> CircleBuilder<'a> {
    pub fn new(renderer: &'a mut Renderer) -> Self {
        let [x, y] = renderer.screen_center();
        Self {
            renderer,
            position: [x, y],
            radius: 32.0,
            segments: 32,
            color: Color::RED,
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = [x, y];
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn segments(mut self, segments: u16) -> Self {
        self.segments = segments;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn draw(self) {
        let [x, y] = self.renderer.pixels_to_ndc(self.position);
        let radius = self
            .renderer
            .pixels_to_ndc_scale([self.radius, self.radius])[0];
        let mut vertices = Vec::with_capacity((self.segments + 1) as usize);
        let mut indices = Vec::with_capacity(self.segments as usize * 3);
        let color = color_to_f32(self.color);

        for i in 0..=self.segments {
            let angle = 2.0 * std::f32::consts::PI * (i as f32 / self.segments as f32);
            vertices.push(Vertex::new(
                [x + radius * angle.cos(), y + radius * angle.sin()],
                [-1.0, -1.0],
                color,
            ));
        }

        for i in 1..=self.segments {
            indices.extend_from_slice(&[0, i, i + 1]);
        }
        indices.extend_from_slice(&[0, self.segments, 1]);

        self.renderer.submit_geometry(&vertices, &indices, 0);
    }
}
