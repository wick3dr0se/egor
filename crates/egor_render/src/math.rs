pub use glam::{IVec2, Mat2, Mat4, Vec2, ivec2, vec2};

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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec2;

    #[test]
    fn new_and_accessors() {
        // sanity check for Rect::new & min/max/center
        let r = Rect::new(vec2(1.0, 2.0), vec2(3.0, 4.0));
        assert_eq!(r.position, vec2(1.0, 2.0));
        assert_eq!(r.size, vec2(3.0, 4.0));
        assert_eq!(r.min(), vec2(1.0, 2.0));
        assert_eq!(r.max(), vec2(4.0, 6.0));
        assert_eq!(r.center(), vec2(2.5, 4.0));
    }

    #[test]
    fn contains() {
        // checks whether a point is inside or on the edge
        let r = Rect::new(vec2(0.0, 0.0), vec2(2.0, 2.0));
        assert!(r.contains(vec2(1.0, 1.0))); // inside
        assert!(r.contains(vec2(0.0, 0.0))); // on min edge
        assert!(r.contains(vec2(2.0, 2.0))); // on max edge
        assert!(!r.contains(vec2(-0.1, 1.0))); // outside left
        assert!(!r.contains(vec2(1.0, 2.1))); // outside top
    }

    #[test]
    fn corners() {
        // returns the 4 corners in TL, TR, BR, BL order
        let r = Rect::new(vec2(0.0, 0.0), vec2(2.0, 2.0));
        let corners = r.corners();
        assert_eq!(corners[0], vec2(0.0, 0.0)); // top-left
        assert_eq!(corners[1], vec2(2.0, 0.0)); // top-right
        assert_eq!(corners[2], vec2(2.0, 2.0)); // bottom-right
        assert_eq!(corners[3], vec2(0.0, 2.0)); // bottom-left
    }
}
