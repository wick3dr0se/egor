#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
fn now() -> f32 {
    (web_sys::window().unwrap().performance().unwrap().now() / 1000.0) as f32
}

#[cfg(not(target_arch = "wasm32"))]
fn now(start: Instant) -> f32 {
    start.elapsed().as_secs_f32()
}

pub struct FrameTimer {
    #[cfg(not(target_arch = "wasm32"))]
    start: Instant,
    last_time: f32,
    accumulator: f32,
    frame_count: u32,
    pub delta: f32,
    pub fps: u32,
}

impl FrameTimer {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            start: Instant::now(),
            last_time: 0.0,
            accumulator: 0.0,
            frame_count: 0,
            delta: 0.0,
            fps: 0,
        }
    }

    pub fn update(&mut self) -> u32 {
        let cur_time = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                now(self.start)
            }
            #[cfg(target_arch = "wasm32")]
            {
                now()
            }
        };

        self.delta = cur_time - self.last_time;
        self.last_time = cur_time;

        self.accumulator += self.delta;
        self.frame_count += 1;

        if self.accumulator >= 1.0 {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.accumulator = 0.0;
        }

        self.fps
    }
}
