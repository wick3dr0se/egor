pub mod color;
pub mod math;
pub mod pipeline;
pub mod text;
pub mod texture;
pub mod vertex;

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, CommandEncoder, Device,
    DeviceDescriptor, IndexFormat, Instance, Limits, LoadOp, Operations, PresentMode, Queue,
    RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp,
    Surface, SurfaceConfiguration, SurfaceError, SurfaceTarget, SurfaceTexture, TextureFormat,
    TextureView, WindowHandle,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    color::Color, pipeline::Pipelines, text::TextRenderer, texture::Texture, vertex::Vertex,
};

const MAX_INDICES: usize = u16::MAX as usize * 32;
const MAX_VERTICES: usize = (MAX_INDICES / 6) * 4;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

#[derive(Clone)]
pub struct GeometryBatch {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Default for GeometryBatch {
    fn default() -> Self {
        Self {
            vertices: Vec::with_capacity(MAX_VERTICES),
            indices: Vec::with_capacity(MAX_INDICES),
        }
    }
}

impl GeometryBatch {
    pub fn push(&mut self, verts: &[Vertex], indices: &[u16]) {
        let idx_offset = self.vertices.len() as u16;
        self.vertices.extend_from_slice(verts);
        self.indices.extend(indices.iter().map(|i| i + idx_offset));
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
    pub text: TextRenderer,
    pub clear_color: Color,
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
        let instance = Instance::default();
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

        // WebGPU throws error 'size is zero' if not set
        let (w, h) = (inner_width.max(1), inner_height.max(1));

        let mut surface_cfg = surface.get_default_config(&adapter, w, h).unwrap();
        surface_cfg.present_mode = PresentMode::AutoVsync;
        surface.configure(&device, &surface_cfg);

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[CameraUniform {
                view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            }]),
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
        let text = TextRenderer::new(&device, &queue, surface_cfg.format);

        Renderer {
            gpu: Gpu {
                device: device.clone(),
                queue,
            },
            target: RenderTarget {
                surface,
                config: surface_cfg,
            },
            pipelines,
            clear_color: Color::BLACK,
            camera_bind_group,
            camera_buffer,
            textures: Vec::new(),
            default_texture,
            text,
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
        batch: &GeometryBatch,
        texture_id: usize,
    ) {
        if batch.vertices.is_empty() || batch.indices.is_empty() {
            return;
        }

        let texture = self
            .textures
            .get(texture_id)
            .unwrap_or(&self.default_texture);
        texture.bind(r_pass, 0);

        let vertex_buffer = self.gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&batch.vertices),
            usage: BufferUsages::VERTEX,
        });

        let mut index_data = bytemuck::cast_slice(&batch.indices).to_vec();
        index_data.resize((index_data.len() + 3) & !3, 0);

        let index_buffer = self.gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &index_data,
            usage: BufferUsages::INDEX,
        });

        r_pass.set_pipeline(&self.pipelines.primitive);
        r_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        r_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        r_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
        r_pass.draw_indexed(0..batch.indices.len() as u32, 0, 0..1);
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
                    load: LoadOp::Clear(self.clear_color.into()),
                    store: StoreOp::Store,
                },
            })],
            ..Default::default()
        })
    }

    /// Convenience wrapper around begin/end
    /// Renders a frame using the given geometry batches grouped by texture ID
    ///
    /// Each `(usize, GeometryBatch)` tuple represents a texture index & associated geometry  
    /// Text is rendered afterward automatically
    pub fn render_frame(&mut self, geometry: Vec<(usize, GeometryBatch)>) {
        let Some(mut frame) = self.begin_frame() else {
            return;
        };

        self.text.prepare(
            &self.gpu.device,
            &self.gpu.queue,
            self.target.config.width,
            self.target.config.height,
        );

        {
            let mut r_pass = self.begin_render_pass(&mut frame.encoder, &frame.view);

            for (tex_id, batch) in &geometry {
                self.draw_batch(&mut r_pass, batch, *tex_id);
            }

            self.text.render(&mut r_pass);
        }

        self.end_frame(frame);
    }

    /// Resizes the surface & updates internal render targets
    pub fn resize(&mut self, w: u32, h: u32) {
        (self.target.config.width, self.target.config.height) = (w, h);
        self.target
            .surface
            .configure(&self.gpu.device, &self.target.config);
        self.text.resize(w, h);
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

    /// Uploads the given view-projection matrix to the GPU for use in vertex transforms
    pub fn upload_camera_matrix(&mut self, mat: glam::Mat4) {
        let cam_uniform = CameraUniform {
            view_proj: mat.to_cols_array_2d(),
        };
        self.gpu
            .queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&cam_uniform));
    }

    /// Adds a new texture from image bytes & returns its id
    pub fn add_texture(&mut self, data: &[u8]) -> usize {
        let img = image::load_from_memory(data).unwrap().to_rgba8();
        let (w, h) = img.dimensions();
        self.add_texture_raw(w, h, &img)
    }

    /// Adds a texture from raw RGBA bytes & returns its id
    pub fn add_texture_raw(&mut self, w: u32, h: u32, data: &[u8]) -> usize {
        let tex = Texture::from_bytes(
            &self.gpu.device,
            &self.gpu.queue,
            &self.pipelines.texture_layout,
            data,
            w,
            h,
        );
        let texture_idx = self.textures.len();
        self.textures.push(tex);
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
        let tex = Texture::from_bytes(
            &self.gpu.device,
            &self.gpu.queue,
            &self.pipelines.texture_layout,
            data,
            w,
            h,
        );
        self.textures[index] = tex;
    }
}
