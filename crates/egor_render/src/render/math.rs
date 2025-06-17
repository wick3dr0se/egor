pub use glam::{Vec2, vec2};

pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }

    pub fn min(&self) -> Vec2 {
        self.position
    }

    pub fn max(&self) -> Vec2 {
        self.position + self.size
    }

    pub fn center(&self) -> Vec2 {
        self.position + self.size * 0.5
    }

    pub fn translate(&mut self, delta: Vec2) {
        self.position += delta;
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.cmpge(self.position).all() && point.cmple(self.position + self.size).all()
    }

    pub fn corners(&self) -> [Vec2; 4] {
        let tl = self.position;
        let tr = vec2(tl.x + self.size.x, tl.y);
        let br = vec2(tl.x + self.size.x, tl.y + self.size.y);
        let bl = vec2(tl.x, tl.y + self.size.y);
        [tl, tr, br, bl]
    }
}
