pub use glam::{Vec2, vec2};

use crate::Anchor;

pub struct Rect {
    pub origin: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(origin: Vec2, size: Vec2) -> Self {
        Self { origin, size }
    }

    pub(crate) fn from_anchor(position: Vec2, size: Vec2, anchor: Anchor) -> Self {
        match anchor {
            Anchor::TopLeft => Self {
                origin: position + size * 0.5,
                size,
            },
            Anchor::Center => Self {
                origin: position,
                size,
            },
        }
    }

    pub fn center(&self) -> Vec2 {
        self.origin + self.size * 0.5
    }

    pub fn corners(&self) -> [Vec2; 4] {
        let half = self.size * 0.5;
        let tl = self.origin - half;
        let br = self.origin + half;
        [
            vec2(tl.x, tl.y),
            vec2(br.x, tl.y),
            vec2(br.x, br.y),
            vec2(tl.x, br.y),
        ]
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.cmpge(self.origin).all() && point.cmple(self.origin + self.size).all()
    }
}
