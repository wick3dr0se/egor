pub struct SpriteFrame {
    pub uv_coords: [[f32; 2]; 4],
    pub duration: f32,
}

pub struct SpriteAnim {
    frames: Vec<SpriteFrame>,
    timer: f32,
    current: usize,
}

impl SpriteAnim {
    pub fn new(rows: usize, cols: usize, total: usize, dur: f32) -> Self {
        let mut frames = Vec::with_capacity(total);
        let (fw, fh) = (1.0 / cols as f32, 1.0 / rows as f32);
        for i in 0..total {
            let (x, y) = ((i % cols) as f32 * fw, (i / cols) as f32 * fh);
            frames.push(SpriteFrame {
                uv_coords: [[x, y], [x + fw, y], [x + fw, y + fh], [x, y + fh]],
                duration: dur,
            });
        }
        Self {
            frames,
            timer: 0.0,
            current: 0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.frames.is_empty() {
            return;
        }
        self.timer += dt;
        if self.timer >= self.frames[self.current].duration {
            self.timer = 0.0;
            self.current = (self.current + 1) % self.frames.len();
        }
    }

    pub fn uv(&self) -> [[f32; 2]; 4] {
        self.frames[self.current].uv_coords
    }
    pub fn frame_uv(&self, f: usize) -> [[f32; 2]; 4] {
        self.frames[f].uv_coords
    }
}
