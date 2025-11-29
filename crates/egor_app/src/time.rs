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
    /// Time in seconds since the last frame
    pub delta: f32,
    /// Frames per second, updated once per second
    pub fps: u32,
    /// Total number of frames rendered since start
    pub frame: u64,
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            start: Instant::now(),
            last_time: 0.0,
            accumulator: 0.0,
            frame_count: 0,
            delta: 0.0,
            fps: 0,
            frame: 0,
        }
    }
}

/// Internal trait for `egor_app` integration or direct use outside `egor`
/// Calculates delta time & updates FPS once per second  
pub trait FrameTimerInternal {
    fn update(&mut self);
}

impl FrameTimerInternal for FrameTimer {
    /// Updates delta time & calculates FPS
    fn update(&mut self) {
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
        self.frame += 1;

        if self.accumulator >= 1.0 {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.accumulator = 0.0;
        }
    }
}
