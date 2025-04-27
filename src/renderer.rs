use wgpu::{
    Color, Device, Instance, Limits, LoadOp, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, wgt::DeviceDescriptor,
};
use winit::{event_loop::EventLoopProxy, window::Window};

use crate::Rc;

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
struct FrameTimer {
    last_time: f64,
    frame_count: u32,
    fps: u32,
}

impl FrameTimer {
    fn new() -> Self {
        Self {
            last_time: now(),
            frame_count: 0,
            fps: 0,
        }
    }

    fn update(&mut self) -> u32 {
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

#[derive(Debug)]
pub struct RenderTarget {
    surface: Surface<'static>,
    config: SurfaceConfiguration,
}

#[derive(Debug)]
pub struct Gpu {
    device: Device,
    queue: Queue,
}

#[derive(Debug)]
pub struct Renderer {
    gpu: Gpu,
    target: RenderTarget,
    clear_color: Color,
    frame_timer: FrameTimer,
}

impl Renderer {
    pub async fn init(window: Rc<Window>, proxy: EventLoopProxy<Renderer>) {
        let instance = Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                // make find adapter that can present to this surface
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                required_limits: if cfg!(target_arch = "wasm32") {
                    // WebGL doesn't support all of wgpu's features; disble some
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let size = window.inner_size();
        let (w, h) = (size.width.max(1), size.height.max(1));
        let mut config = surface.get_default_config(&adapter, w, h).unwrap();
        config.present_mode = PresentMode::Fifo;

        #[cfg(not(target_arch = "wasm32"))]
        surface.configure(&device, &config);

        let renderer = Renderer {
            gpu: Gpu { device, queue },
            target: RenderTarget { surface, config },
            clear_color: Color::BLACK,
            frame_timer: FrameTimer::new(),
        };
        proxy.send_event(renderer).unwrap();
    }

    pub fn clear(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub fn fps(&self) -> u32 {
        self.frame_timer.fps
    }

    pub(crate) fn render_frame(&mut self) {
        let frame = self.target.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        {
            let mut _r_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.clear_color),
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }

        self.gpu.queue.submit(Some(encoder.finish()));
        frame.present();

        self.frame_timer.update();
    }

    pub(crate) fn resize(&mut self, w: u32, h: u32) {
        (self.target.config.width, self.target.config.height) = (w, h);
        self.target
            .surface
            .configure(&self.gpu.device, &self.target.config);
    }
}
