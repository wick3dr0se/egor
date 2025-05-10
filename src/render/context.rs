use super::{Renderer, vertex::Vertex};

pub struct GraphicsContext<'a> {
    pub(crate) renderer: &'a mut Renderer,
}

impl<'a> GraphicsContext<'a> {
    pub fn triangle(&mut self, x: f32, y: f32, size: f32) {
        let half = size / 2.0;
        let vertices = [
            Vertex::new([x, y + half], [1.0, 0.0, 0.0, 1.0]),
            Vertex::new([x + half, y - half], [0.0, 1.0, 0.0, 1.0]),
            Vertex::new([x - half, y - half], [0.0, 0.0, 1.0, 1.0]),
        ];
        let indices = [0, 1, 2];
        self.renderer.submit_geometry(&vertices, &indices);
    }
    pub fn rectangle(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let vertices = [
            Vertex::new([x, y], [1.0, 0.0, 0.0, 1.0]),
            Vertex::new([x + w, y], [0.0, 1.0, 0.0, 1.0]),
            Vertex::new([x + w, y + h], [0.0, 0.0, 1.0, 1.0]),
            Vertex::new([x, y + h], [1.0, 1.0, 0.0, 1.0]),
        ];
        let indices = [0, 1, 2, 2, 3, 0];
        self.renderer.submit_geometry(&vertices, &indices);
    }
}
