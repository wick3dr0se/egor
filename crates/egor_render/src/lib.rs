pub mod pipeline;
pub mod texture;
pub mod vertex;

pub use wgpu::{Device, Queue, RenderPass, TextureFormat};

use wgpu::{
    Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, Color,
    CommandEncoder, DeviceDescriptor, IndexFormat, Instance, LoadOp, Operations, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, SurfaceTarget, SurfaceTexture, TextureView, WindowHandle,
    util::{BufferInitDescriptor, DeviceExt, new_instance_with_webgpu_detection},
};

use crate::{pipeline::Pipelines, texture::Texture, vertex::Vertex};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

/// A batch of geometry (vertices + indices) that can be drawn in a single GPU call
///
/// Tracks CPU vertex/index data, lazily uploads GPU buffers and prevents overflowing `u16` indices
pub struct GeometryBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    vertices_dirty: bool,
    indices_dirty: bool,
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

impl GeometryBatch {
    const MAX_VERTICES: usize = u16::MAX as usize;
    const MAX_INDICES: usize = Self::MAX_VERTICES * 6;

    // Returns true if adding verts/indices would exceed max allowed
    fn would_overflow(&self, vert_count: usize, idx_count: usize) -> bool {
        self.vertices.len() + vert_count > Self::MAX_VERTICES
            || self.indices.len() + idx_count > Self::MAX_INDICES
    }

    /// Adds vertices/indices, returns false if it would overflow
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

    // Uploads buffers to GPU only if needed
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

/// Trait for presenting rendered frames
pub trait Presentable {
    fn present(self: Box<Self>);
}

impl Presentable for SurfaceTexture {
    fn present(self: Box<Self>) {
        (*self).present();
    }
}

pub struct Frame {
    pub view: TextureView,
    pub encoder: CommandEncoder,
    presentable: Option<Box<dyn Presentable>>,
}

impl Frame {
    fn finish(self, queue: &Queue) {
        queue.submit(Some(self.encoder.finish()));
        if let Some(p) = self.presentable {
            p.present();
        }
    }
}

/// Trait for render targets (backbuffers, offscreen textures, etc.)
pub trait RenderTarget {
    fn format(&self) -> TextureFormat;
    fn size(&self) -> (u32, u32);
    fn begin(&mut self) -> Option<(TextureView, Option<Box<dyn Presentable>>)>;
    fn resize(&mut self, device: &Device, w: u32, h: u32);
    // only useful for backbuffer targets
    fn set_vsync(&mut self, _device: &Device, _on: bool) {}
}

/// Renders to the window's backbuffer (swapchain)
pub struct Backbuffer {
    surface: Surface<'static>,
    config: SurfaceConfiguration,
}

impl Backbuffer {
    pub fn new(
        instance: &Instance,
        adapter: &Adapter,
        device: &Device,
        window: impl Into<SurfaceTarget<'static>> + WindowHandle,
        w: u32,
        h: u32,
    ) -> Self {
        let surface = instance.create_surface(window).unwrap();
        let mut config = surface.get_default_config(adapter, w, h).unwrap();
        config.present_mode = PresentMode::AutoVsync;
        surface.configure(device, &config);
        Self { surface, config }
    }
}

impl RenderTarget for Backbuffer {
    fn format(&self) -> TextureFormat {
        self.config.format
    }

    fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    fn begin(&mut self) -> Option<(TextureView, Option<Box<dyn Presentable>>)> {
        let surface_texture = self.surface.get_current_texture().ok()?;
        let view = surface_texture.texture.create_view(&Default::default());

        Some((view, Some(Box::new(surface_texture))))
    }

    fn resize(&mut self, device: &Device, w: u32, h: u32) {
        (self.config.width, self.config.height) = (w, h);
        self.surface.configure(device, &self.config);
    }

    fn set_vsync(&mut self, device: &Device, on: bool) {
        self.config.present_mode = if on {
            PresentMode::Fifo
        } else {
            PresentMode::AutoNoVsync
        };
        self.surface.configure(device, &self.config);
    }
}

struct Gpu {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

/// Low-level GPU renderer built on `wgpu`
///
/// Handles rendering pipelines, surface configuration, resources (textures, buffers), & drawing
pub struct Renderer {
    gpu: Gpu,
    target: Option<Box<dyn RenderTarget>>,
    pipelines: Pipelines,
    camera_bind_group: BindGroup,
    camera_buffer: Buffer,
    textures: Vec<Texture>,
    default_texture: Texture,
    clear_color: Color,
    last_size: (u32, u32),
}

impl Renderer {
    /// Creates a new `Renderer` configured for the given surface format
    ///
    /// Initializes `wgpu`, sets up render pipelines, default texture & camera uniform
    pub async fn new(
        w: u32,
        h: u32,
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

        let mut surface_cfg = surface.get_default_config(&adapter, w, h).unwrap();
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
            gpu: Gpu {
                instance,
                adapter,
                device,
                queue,
            },
            target: Some(Box::new(Backbuffer {
                surface,
                config: surface_cfg,
            })),
            pipelines,
            camera_bind_group,
            camera_buffer,
            textures: Vec::new(),
            default_texture,
            clear_color: Color::BLACK,
            last_size: (w, h),
        }
    }

    /// Returns a reference to the underlying wgpu `Instance`
    pub fn instance(&self) -> &Instance {
        &self.gpu.instance
    }
    /// Returns a reference to the underlying wgpu `Adapter`
    pub fn adapter(&self) -> &Adapter {
        &self.gpu.adapter
    }
    /// Returns a reference to the underlying wgpu `Device`
    pub fn device(&self) -> &Device {
        &self.gpu.device
    }
    /// Returns a reference to the underlying wgpu `Queue`
    pub fn queue(&self) -> &Queue {
        &self.gpu.queue
    }

    /// Sets the active render target
    pub fn set_target(&mut self, target: impl RenderTarget + 'static) {
        self.target = Some(Box::new(target));
    }
    /// Returns the format of the current surface
    pub fn surface_format(&self) -> TextureFormat {
        self.target.as_ref().unwrap().format()
    }
    /// Returns the current surface dimensions (in pixels)
    pub fn surface_size(&self) -> (u32, u32) {
        self.target.as_ref().unwrap().size()
    }
    /// Resize the current render target
    pub fn resize(&mut self, w: u32, h: u32) {
        self.last_size = (w, h);
        self.target.as_mut().unwrap().resize(&self.gpu.device, w, h);
    }
    /// Enables/disables V‑Sync (only for backbuffer targets)
    pub fn set_vsync(&mut self, on: bool) {
        self.target
            .as_mut()
            .unwrap()
            .set_vsync(&self.gpu.device, on);
    }
    /// Stores the current surface size and invalidates render target
    pub fn destroy_surface(&mut self) {
        if let Some(target) = &self.target {
            self.last_size = target.size();
        }
        self.target = None;
    }
    /// Creates a new backbuffer using the last known surface size
    pub fn recreate_surface(&mut self, window: impl Into<SurfaceTarget<'static>> + WindowHandle) {
        let (w, h) = self.last_size;
        self.target = Some(Box::new(Backbuffer::new(
            &self.gpu.instance,
            &self.gpu.adapter,
            &self.gpu.device,
            window,
            w,
            h,
        )));
    }

    /// Sets the clear color for future render passes
    pub fn set_clear_color(&mut self, color: [f64; 4]) {
        self.clear_color = Color {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        };
    }

    /// Begins a new frame with the given render target
    pub fn begin_frame(&mut self) -> Option<Frame> {
        let (view, presentable) = self.target.as_mut()?.begin()?;
        let encoder = self.gpu.device.create_command_encoder(&Default::default());

        Some(Frame {
            view,
            encoder,
            presentable,
        })
    }

    /// Ends the frame by submitting commands and presenting
    pub fn end_frame(&mut self, frame: Frame) {
        frame.finish(&self.gpu.queue);
    }

    /// Begins a render pass with the given encoder and target view.
    /// Clears the view (set by [`Self::set_clear_color`])
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
