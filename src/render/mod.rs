use wgpu::{
    Device, DeviceDescriptor, Instance, Limits, LoadOp, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration,
};
use winit::{event_loop::EventLoopProxy, window::Window};

use crate::Rc;

pub use wgpu::Color;

struct RenderTarget {
    surface: Surface<'static>,
    config: SurfaceConfiguration,
}

struct Gpu {
    device: Device,
    queue: Queue,
}

pub struct Renderer {
    gpu: Gpu,
    target: RenderTarget,
}

impl Renderer {
    pub async fn create_graphics(window: Rc<Window>, proxy: EventLoopProxy<Self>) {
        let instance = Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                // Force find adapter that can present to this surface
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                required_limits: if cfg!(target_arch = "wasm32") {
                    // WebGL doesn't support all of wgpu's features; disable some
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::default()
                },
                ..Default::default()
            })
            .await
            .unwrap();

        let size = window.inner_size();
        // WebGPU throws error 'size is zero' if not set
        let (w, h) = (size.width.max(1), size.height.max(1));

        let mut surface_cfg = surface.get_default_config(&adapter, w, h).unwrap();
        surface_cfg.present_mode = PresentMode::Fifo;
        surface.configure(&device, &surface_cfg);

        let _ = proxy.send_event(Renderer {
            gpu: Gpu { device, queue },
            target: RenderTarget {
                surface,
                config: surface_cfg,
            },
        });
    }

    pub fn render_frame(&self) {
        let frame = self.target.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        {
            let mut _r_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::GREEN),
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }

        self.gpu.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        (self.target.config.width, self.target.config.height) = (w, h);
        self.target
            .surface
            .configure(&self.gpu.device, &self.target.config);
    }
}
