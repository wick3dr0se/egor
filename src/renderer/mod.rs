use std::sync::Arc;

use graphics::Graphics;
use texture::Texture;
use vertex::Vertex;
use wgpu::{
    BindGroupLayout, BufferUsages, Color, Device, FragmentState, IndexFormat, Instance, LoadOp,
    Operations, PipelineLayoutDescriptor, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor,
    ShaderSource, StoreOp, Surface, SurfaceConfiguration, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::{dpi::PhysicalSize, window::Window};

pub mod graphics;
pub mod texture;
pub mod vertex;

#[derive(Debug)]
pub struct GeometryBatch {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture_index: usize,
}

pub struct Renderer {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    config: SurfaceConfiguration,
    geometry_batches: Vec<GeometryBatch>,
    textures: Vec<Texture>,
    pub screen_size: PhysicalSize<u32>,
    pub(crate) clear_color: Color,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = Instance::default();
        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: Default::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        let bind_group_layout = Texture::create_bind_group_layout(&device);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });

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
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(config.format.into())],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        Renderer {
            device,
            queue,
            surface,
            pipeline,
            bind_group_layout,
            config,
            geometry_batches: Vec::new(),
            textures: Vec::new(),
            screen_size: size,
            clear_color: Color::BLACK,
        }
    }

    pub fn screen_width(&self) -> f32 {
        self.screen_size.width as f32
    }

    pub fn screen_height(&self) -> f32 {
        self.screen_size.height as f32
    }

    pub fn screen_center(&self) -> [f32; 2] {
        [self.screen_width() / 2.0, self.screen_height() / 2.0]
    }

    pub fn graphics(&mut self) -> Graphics {
        Graphics::new(self)
    }

    pub fn pixels_to_ndc(&self, point: [f32; 2]) -> [f32; 2] {
        let (w, h) = (self.screen_width(), self.screen_height());
        [(point[0] / w) * 2.0 - 1.0, 1.0 - (point[1] / h) * 2.0]
    }

    pub fn pixels_to_ndc_scale(&self, dimensions: [f32; 2]) -> [f32; 2] {
        let (w, h) = (self.screen_width(), self.screen_height());
        [(dimensions[0] / w) * 2.0, (dimensions[1] / h) * 2.0]
    }

    pub fn submit_geometry(&mut self, vertices: &[Vertex], indices: &[u16], tex_idx: usize) {
        self.geometry_batches.push(GeometryBatch {
            vertices: vertices.into(),
            indices: indices.into(),
            texture_index: tex_idx,
        });
    }

    pub fn add_texture(&mut self, data: &[u8]) -> usize {
        let texture = Texture::load(&self.device, &self.queue, &self.bind_group_layout, data);
        let texture_idx = self.textures.len();

        self.textures.push(texture);
        texture_idx
    }

    pub fn render(&mut self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

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

            for batch in &self.geometry_batches {
                let vertex_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&batch.vertices),
                    usage: BufferUsages::VERTEX,
                });
                let index_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&batch.indices),
                    usage: BufferUsages::INDEX,
                });

                if let Some(texture) = self.textures.get(batch.texture_index) {
                    r_pass.set_bind_group(0, &texture.bind_group, &[]);
                }

                r_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                r_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
                r_pass.draw_indexed(0..batch.indices.len() as u32, 0, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        self.geometry_batches.clear();
    }
}
