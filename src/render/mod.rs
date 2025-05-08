pub mod vertex;

use vertex::Vertex;
use wgpu::{
    BufferUsages, Device, DeviceDescriptor, FragmentState, IndexFormat, Instance, Limits, LoadOp,
    Operations, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, VertexState, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::{event_loop::EventLoopProxy, window::Window};

use crate::Rc;

pub use wgpu::Color;

pub struct GeometryBatch {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

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
    pipeline: RenderPipeline,
    geometry_batches: Vec<GeometryBatch>,
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

        let shader = device.create_shader_module(include_wgsl!("../../shader.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&Default::default());
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(surface_cfg.format.into())],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        let _ = proxy.send_event(Renderer {
            gpu: Gpu { device, queue },
            target: RenderTarget {
                surface,
                config: surface_cfg,
            },
            pipeline,
            geometry_batches: Vec::new(),
        });
    }

    pub fn render_frame(&self) {
        let frame = self.target.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        {
            let mut r_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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

            r_pass.set_pipeline(&self.pipeline);

            for batch in &self.geometry_batches {
                let vertex_buffer = self.gpu.device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&batch.vertices),
                    usage: BufferUsages::VERTEX,
                });
                let index_buffer = self.gpu.device.create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&batch.indices),
                    usage: BufferUsages::INDEX,
                });

                r_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                r_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
                r_pass.draw_indexed(0..batch.indices.len() as u32, 0, 0..1);
            }
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

    pub fn submit_geometry(&mut self, vertices: &[Vertex], indices: &[u16]) {
        self.geometry_batches.push(GeometryBatch {
            vertices: vertices.into(),
            indices: indices.into(),
        });
    }
}
