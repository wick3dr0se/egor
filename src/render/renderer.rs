use super::{texture::Texture, vertex::Vertex};
use wgpu::{
    BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, Device, DeviceDescriptor,
    FragmentState, IndexFormat, Instance, Limits, LoadOp, Operations, PipelineLayoutDescriptor,
    PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration,
    VertexState, include_wgsl,
};
use winit::{event_loop::EventLoopProxy, window::Window};

use crate::Rc;

pub use wgpu::Color;

const MAX_VERTICES: usize = 43_690;
const MAX_INDICES: usize = u16::MAX as usize;

pub struct RenderBatch {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture_index: usize,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl RenderBatch {
    pub fn new(device: &Device) -> Self {
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (MAX_VERTICES * size_of::<Vertex>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (MAX_INDICES * size_of::<u16>()) as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertices: Vec::with_capacity(MAX_VERTICES),
            indices: Vec::with_capacity(MAX_INDICES),
            texture_index: 0,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn submit(&mut self, vertices: &[Vertex], indices: &[u16], tex_idx: usize) {
        let idx = self.vertices.len() as u16;
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.iter().map(|i| i + idx));
        self.texture_index = tex_idx;
    }

    pub fn upload(&self, queue: &Queue) {
        assert!(
            self.vertices.len() <= MAX_VERTICES,
            "Vertex buffer overflow"
        );
        assert!(self.indices.len() <= MAX_INDICES, "Index buffer overflow");

        if !self.vertices.is_empty() {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        }
        if !self.indices.is_empty() {
            let mut data = bytemuck::cast_slice(&self.indices).to_vec();
            data.resize((data.len() + 3) & !3, 0); // force align to 4 bytes
            queue.write_buffer(&self.index_buffer, 0, &data);
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }
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
    batches: Vec<RenderBatch>,
    clear_color: Color,
    bind_group_layout: BindGroupLayout,
    textures: Vec<Texture>,
    default_texture: Texture,
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
        let bind_group_layout = Texture::create_bind_group_layout(&device);
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
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

        let default_texture = Texture::create_default(&device, &queue, &bind_group_layout);

        let _ = proxy.send_event(Renderer {
            gpu: Gpu {
                device: device.clone(),
                queue,
            },
            target: RenderTarget {
                surface,
                config: surface_cfg,
            },
            pipeline,
            batches: vec![RenderBatch::new(&device)],
            clear_color: Color::BLACK,
            bind_group_layout,
            textures: Vec::new(),
            default_texture,
        });
    }

    pub fn render_frame(&mut self) {
        let frame = self.target.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        {
            let mut r_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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

            r_pass.set_pipeline(&self.pipeline);

            for batch in &self.batches {
                if batch.vertices.is_empty() {
                    continue;
                }

                let texture = self
                    .textures
                    .get(batch.texture_index)
                    .unwrap_or(&self.default_texture);
                texture.bind(&mut r_pass, 0);

                batch.upload(&self.gpu.queue);
                r_pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
                r_pass.set_index_buffer(batch.index_buffer.slice(..), IndexFormat::Uint16);
                r_pass.draw_indexed(0..batch.index_count(), 0, 0..1);
            }
        }

        self.gpu.queue.submit(Some(encoder.finish()));
        frame.present();

        for batch in &mut self.batches {
            batch.clear();
        }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        (self.target.config.width, self.target.config.height) = (w, h);
        self.target
            .surface
            .configure(&self.gpu.device, &self.target.config);
    }

    pub fn clear(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub fn screen_width(&self) -> f32 {
        self.target.config.width as f32
    }

    pub fn screen_height(&self) -> f32 {
        self.target.config.height as f32
    }

    pub fn submit(&mut self, vertices: &[Vertex], indices: &[u16], texture_index: usize) {
        for batch in &mut self.batches {
            if batch.texture_index == texture_index {
                batch.submit(vertices, indices, texture_index);
                return;
            }
        }

        let mut new_batch = RenderBatch::new(&self.gpu.device);
        new_batch.submit(vertices, indices, texture_index);
        self.batches.push(new_batch);
    }

    pub fn to_ndc(&self, x: f32, y: f32) -> [f32; 2] {
        let (w, h) = (self.screen_width(), self.screen_height());
        [(x / w) * 2.0 - 1.0, 1.0 - (y / h) * 2.0]
    }

    pub fn add_texture(&mut self, data: &[u8]) -> usize {
        let img = image::load_from_memory(data).unwrap().to_rgba8();
        let (w, h) = img.dimensions();

        let tex = Texture::from_bytes(
            &self.gpu.device,
            &self.gpu.queue,
            &self.bind_group_layout,
            &img,
            w,
            h,
        );
        let texture_idx = self.textures.len();

        self.textures.push(tex);
        texture_idx
    }
}
