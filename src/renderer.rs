use wgpu::{
    Color, Device, Instance, Limits, LoadOp, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, wgt::DeviceDescriptor,
};
use winit::{event_loop::EventLoopProxy, window::Window};

use crate::{Rc, time::FrameTimer};

pub struct RenderTarget {
    surface: Surface<'static>,
    config: SurfaceConfiguration,
}

pub struct Gpu {
    device: Device,
    queue: Queue,
}

pub struct Renderer {
    gpu: Gpu,
    target: RenderTarget,
    pub(crate) clear_color: Color,
    pub(crate) frame_timer: FrameTimer,
}

impl Renderer {
    pub async fn init(window: Rc<Window>, proxy: EventLoopProxy<Renderer>) {
        let instance = Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                // force find adapter that can present to this surface
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

        let _ = proxy.send_event(Renderer {
            gpu: Gpu { device, queue },
            target: RenderTarget { surface, config },
            clear_color: Color::BLACK,
            frame_timer: FrameTimer::new(),
        });
    }

    pub(crate) fn render_frame(&mut self) {
        self.frame_timer.update();

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
    }

    pub(crate) fn resize(&mut self, w: u32, h: u32) {
        (self.target.config.width, self.target.config.height) = (w, h);
        self.target
            .surface
            .configure(&self.gpu.device, &self.target.config);
    }
}
