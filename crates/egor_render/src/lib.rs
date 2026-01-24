pub mod pipeline;
pub mod texture;
pub mod vertex;

pub use wgpu::{Device, Queue, RenderPass, TextureFormat};

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, Color, CommandEncoder,
    DeviceDescriptor, IndexFormat, LoadOp, Operations, PresentMode, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration,
    SurfaceError, SurfaceTarget, SurfaceTexture, TextureView, WindowHandle,
    util::{BufferInitDescriptor, DeviceExt, new_instance_with_webgpu_detection},
};

use crate::{pipeline::Pipelines, texture::Texture, vertex::Vertex};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

pub struct GeometryBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    vertices_dirty: bool,
    indices_dirty: bool,
}

impl GeometryBatch {
    const MAX_VERTICES: usize = u16::MAX as usize;
    const MAX_INDICES: usize = Self::MAX_VERTICES * 6;

    fn would_overflow(&self, vert_count: usize, idx_count: usize) -> bool {
        self.vertices.len() + vert_count > Self::MAX_VERTICES
            || self.indices.len() + idx_count > Self::MAX_INDICES
    }

    pub fn push(&mut self, verts: &[Vertex], indices: &[u16]) -> bool {
        if self.would_overflow(verts.len(), indices.len()) {
            return false;
        }

        let idx_offset = self.vertices.len() as u16;
        self.vertices.extend_from_slice(verts);
        self.indices.extend(indices.iter().map(|i| *i + idx_offset));

        self.vertices_dirty = true;
        self.indices_dirty = true;

        true
    }

    fn upload(&mut self, device: &Device, queue: &Queue) {
        if self.is_empty() || (!self.vertices_dirty && !self.indices_dirty) {
            return;
        }

        if self.vertex_buffer.is_none() {
            self.vertex_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("GeometryBatch Vertex Buffer"),
                size: (Self::MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        if self.index_buffer.is_none() {
            self.index_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("GeometryBatch Index Buffer"),
                size: (Self::MAX_INDICES * std::mem::size_of::<u16>()) as u64,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        if self.vertices_dirty {
            queue.write_buffer(
                self.vertex_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&self.vertices),
            );
            self.vertices_dirty = false;
        }
        if self.indices_dirty {
            let mut indices_bytes: Vec<u8> = bytemuck::cast_slice(&self.indices).to_vec();
            let remainder = indices_bytes.len() % wgpu::COPY_BUFFER_ALIGNMENT as usize;
            if remainder != 0 {
                let pad_len = wgpu::COPY_BUFFER_ALIGNMENT as usize - remainder;
                indices_bytes.extend_from_slice(&vec![0u8; pad_len]);
            }

            queue.write_buffer(self.index_buffer.as_ref().unwrap(), 0, &indices_bytes);
            self.indices_dirty = false;
        }
    }

    fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }

    fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.vertices_dirty = true;
        self.indices_dirty = true;
    }

    fn draw(&self, r_pass: &mut RenderPass) {
        if self.is_empty() {
            return;
        }

        r_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        r_pass.set_index_buffer(
            self.index_buffer.as_ref().unwrap().slice(..),
            IndexFormat::Uint16,
        );
        r_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
    }
}

impl Default for GeometryBatch {
    fn default() -> Self {
        Self {
            vertices: Vec::with_capacity(Self::MAX_VERTICES),
            indices: Vec::with_capacity(Self::MAX_INDICES),
            vertex_buffer: None,
            index_buffer: None,
            vertices_dirty: false,
            indices_dirty: false,
        }
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

pub struct Frame {
    pub view: TextureView,
    pub encoder: CommandEncoder,
    surface_texture: SurfaceTexture,
}

/// Low-level GPU renderer built on `wgpu`
///
/// Handles rendering pipelines, surface configuration, resources (textures, buffers), & drawing
pub struct Renderer {
    gpu: Gpu,
    target: RenderTarget,
    pipelines: Pipelines,
    camera_bind_group: BindGroup,
    camera_buffer: Buffer,
    textures: Vec<Texture>,
    default_texture: Texture,
    clear_color: Color,
}

impl Renderer {
    /// Creates a new `Renderer` with a configured surface, pipeline & default resources
    ///
    /// Initializes `wgpu`, sets up a basic alpha-blended render pipeline, default texture,
    /// camera uniform, internal text renderer & more
    pub async fn new(
        inner_width: u32,
        inner_height: u32,
        window: impl Into<SurfaceTarget<'static>> + WindowHandle,
    ) -> Renderer {
        let instance = new_instance_with_webgpu_detection(&Default::default()).await;
        let surface = instance.create_surface(window).unwrap();
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
                #[cfg(target_arch = "wasm32")]
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                ..Default::default()
            })
            .await
            .unwrap();

        let mut surface_cfg = surface
            .get_default_config(&adapter, inner_width, inner_height)
            .unwrap();
        surface_cfg.present_mode = PresentMode::AutoVsync;
        surface.configure(&device, &surface_cfg);

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&CameraUniform {
                view_proj: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            }),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let pipelines = Pipelines::new(&device, surface_cfg.format);
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipelines.camera_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let default_texture = Texture::create_default(&device, &queue, &pipelines.texture_layout);

        Renderer {
            gpu: Gpu { device, queue },
            target: RenderTarget {
                surface,
                config: surface_cfg,
            },
            pipelines,
            camera_bind_group,
            camera_buffer,
            textures: Vec::new(),
            default_texture,
            clear_color: Color::BLACK,
        }
    }

    pub fn device(&self) -> &Device {
        &self.gpu.device
    }

    pub fn queue(&self) -> &Queue {
        &self.gpu.queue
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.target.config.format
    }

    pub fn surface_config(&self) -> &SurfaceConfiguration {
        &self.target.config
    }

    /// Begins a new frame, returning the surface texture and command encoder
    pub fn begin_frame(&mut self) -> Option<Frame> {
        let surface_texture = match self.target.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(SurfaceError::OutOfMemory) => {
                panic!("Out of GPU memory!");
            }
            Err(_) => return None,
        };

        let view = surface_texture.texture.create_view(&Default::default());
        let encoder = self.gpu.device.create_command_encoder(&Default::default());

        Some(Frame {
            view,
            encoder,
            surface_texture,
        })
    }

    /// Ends the frame by submitting commands and presenting
    pub fn end_frame(&mut self, frame: Frame) {
        self.gpu.queue.submit(Some(frame.encoder.finish()));
        frame.surface_texture.present();
    }

    /// Draws a geometry batch within an existing render pass
    pub fn draw_batch(
        &self,
        r_pass: &mut RenderPass<'_>,
        batch: &mut GeometryBatch,
        texture_id: usize,
    ) {
        if batch.is_empty() {
            return;
        }
        batch.upload(&self.gpu.device, &self.gpu.queue);

        let texture = self
            .textures
            .get(texture_id)
            .unwrap_or(&self.default_texture);
        texture.bind(r_pass, 0);

        r_pass.set_pipeline(&self.pipelines.primitive);
        r_pass.set_bind_group(1, &self.camera_bind_group, &[]);

        batch.draw(r_pass);
        batch.clear();
    }

    pub fn begin_render_pass<'a>(
        &'a self,
        encoder: &'a mut CommandEncoder,
        view: &'a TextureView,
    ) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(self.clear_color),
                    store: StoreOp::Store,
                },
            })],
            ..Default::default()
        })
    }

    /// Resizes the surface & updates internal render targets
    pub fn resize(&mut self, w: u32, h: u32) {
        (self.target.config.width, self.target.config.height) = (w, h);
        self.target
            .surface
            .configure(&self.gpu.device, &self.target.config);
    }

    /// Returns the current surface dimensions (in pixels)
    pub fn surface_size(&self) -> (f32, f32) {
        (
            self.target.config.width as f32,
            self.target.config.height as f32,
        )
    }

    /// Enables/disables V‑Sync by changing the surface present mode
    ///
    /// `vsync = true` → [`PresentMode::Fifo`] (V‑Sync ON)  
    /// `vsync = false` → [`PresentMode::AutoNoVsync`] (V‑Sync OFF)
    ///
    /// Reconfigures the surface immediately
    pub fn set_vsync(&mut self, on: bool) {
        self.target.config.present_mode = if on {
            PresentMode::Fifo
        } else {
            PresentMode::AutoNoVsync
        };

        self.target
            .surface
            .configure(&self.gpu.device, &self.target.config);
    }

    pub fn set_clear_color(&mut self, color: [f64; 4]) {
        self.clear_color = Color {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        };
    }

    /// Uploads the given view-projection matrix to the GPU for use in vertex transforms
    pub fn upload_camera_matrix(&mut self, view_proj: [[f32; 4]; 4]) {
        self.gpu.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&CameraUniform { view_proj }),
        );
    }

    /// Adds a new texture from image bytes & returns its id
    pub fn add_texture(&mut self, data: &[u8]) -> usize {
        let img = image::load_from_memory(data).unwrap().to_rgba8();
        let (w, h) = img.dimensions();
        self.add_texture_raw(w, h, &img)
    }

    /// Adds a texture from raw RGBA bytes & returns its id
    pub fn add_texture_raw(&mut self, w: u32, h: u32, data: &[u8]) -> usize {
        let texture_idx = self.textures.len();
        self.textures.push(Texture::from_bytes(
            &self.gpu.device,
            &self.gpu.queue,
            &self.pipelines.texture_layout,
            data,
            w,
            h,
        ));
        texture_idx
    }

    /// Replaces an existing texture with new image data
    pub fn update_texture(&mut self, index: usize, data: &[u8]) {
        let img = image::load_from_memory(data).unwrap().to_rgba8();
        let (w, h) = img.dimensions();
        self.update_texture_raw(index, w, h, &img)
    }

    /// Replaces an existing texture with raw RGBA bytes
    pub fn update_texture_raw(&mut self, index: usize, w: u32, h: u32, data: &[u8]) {
        self.textures[index] = Texture::from_bytes(
            &self.gpu.device,
            &self.gpu.queue,
            &self.pipelines.texture_layout,
            data,
            w,
            h,
        );
    }
}
