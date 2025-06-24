pub use glam::{Vec2, vec2};

/// Axis-aligned rectangle defined by position (top-left corner) & size
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

impl Rect {
    /// Create a new rectangle from position (top-left) & size
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }

    /// Returns the top-left corner (min coords)
    pub fn min(&self) -> Vec2 {
        self.position
    }

    /// Returns the bottom-right corner (max coords)
    pub fn max(&self) -> Vec2 {
        self.position + self.size
    }

    /// Returns the center point of the rectangle
    pub fn center(&self) -> Vec2 {
        self.position + self.size * 0.5
    }

    // Move the rectangle by the given delta vector
    pub fn translate(&mut self, delta: Vec2) {
        self.position += delta;
    }

    /// Returns true if the point is inside of the rectangle
    pub fn contains(&self, point: Vec2) -> bool {
        point.cmpge(self.position).all() && point.cmple(self.position + self.size).all()
    }

    /// Returns the four corners in this order: top-left, top-right, bottom-right, bottom-left
    pub fn corners(&self) -> [Vec2; 4] {
        let tl = self.position;
        let tr = vec2(tl.x + self.size.x, tl.y);
        let br = vec2(tl.x + self.size.x, tl.y + self.size.y);
        let bl = vec2(tl.x, tl.y + self.size.y);
        [tl, tr, br, bl]
    }
}
