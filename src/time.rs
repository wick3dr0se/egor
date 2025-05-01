#[cfg(not(target_arch = "wasm32"))]
fn now() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

#[cfg(target_arch = "wasm32")]
fn now() -> f64 {
    web_sys::window().unwrap().performance().unwrap().now()
}

#[derive(Debug)]
pub struct FrameTimer {
    last_time: f64,
    frame_count: u32,
    pub(crate) fps: u32,
}

impl FrameTimer {
    pub fn new() -> Self {
        Self {
            last_time: now(),
            frame_count: 0,
            fps: 0,
        }
    }

    pub fn update(&mut self) -> u32 {
        self.frame_count += 1;
        let current_time = now();
        if current_time - self.last_time >= 1.0 {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.last_time = current_time;
        }
        self.fps
    }
}
