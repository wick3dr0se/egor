use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferUsages, ColorTargetState,
    ColorWrites, Device, DeviceDescriptor, FragmentState, IndexFormat, Instance, Limits, LoadOp,
    Operations, PipelineLayoutDescriptor, PresentMode, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderStages, StoreOp, Surface, SurfaceConfiguration, SurfaceTarget, VertexState, WindowHandle,
    include_wgsl, util::DeviceExt,
};

use crate::{Color, text::TextRenderer, texture::Texture, vertex::Vertex};

const MAX_INDICES: usize = u16::MAX as usize * 32;
const MAX_VERTICES: usize = (MAX_INDICES / 6) * 4;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

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

/// Low-level GPU renderer built on `wgpu`
///
/// Handles rendering pipelines, surface configuration, resources (textures, buffers), & drawing  
/// Used internally by [`Graphics`](crate::Graphics) to render 2D primitives
///
/// Most users shouldn't interact with this directly unless doing advanced rendering or hooking into the pipeline
pub struct Renderer {
    gpu: Gpu,
    target: RenderTarget,
    pipeline: RenderPipeline,
    clear_color: Color,
    bind_group_layout: BindGroupLayout,
    camera_bind_group: BindGroup,
    camera_buffer: Buffer,
    textures: Vec<Texture>,
    default_texture: Texture,
    pub(crate) text: TextRenderer,
}

impl Renderer {
    /// Creates a new `Renderer` with a configured surface, pipeline & default resources
    ///
    /// Initializes `wgpu`, sets up a basic alpha-blended render pipeline, default texture,
    /// camera uniform, internal text renderer & more
    pub async fn create_graphics<'w>(
        inner_width: u32,
        inner_height: u32,
        window: impl Into<SurfaceTarget<'static>> + WindowHandle + 'w,
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

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[CameraUniform {
                view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            }]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(include_wgsl!("../shader.wgsl"));
        let bind_group_layout = Texture::create_bind_group_layout(&device);
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout, &camera_bind_group_layout],
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
                targets: &[Some(ColorTargetState {
                    format: surface_cfg.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        let default_texture = Texture::create_default(&device, &queue, &bind_group_layout);
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
            pipeline,
            clear_color: Color::BLACK,
            bind_group_layout,
            camera_bind_group,
            camera_buffer,
            textures: Vec::new(),
            default_texture,
            text,
        }
    }

    /// Renders a frame using the given geometry batches grouped by texture ID
    ///
    /// Each `(usize, GeometryBatch)` tuple represents a texture index & associated geometry  
    /// Text is rendered afterward automatically
    pub fn render_frame(&mut self, geometry: Vec<(usize, GeometryBatch)>) {
        let frame = self.target.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());

        self.text.prepare(
            &self.gpu.device,
            &self.gpu.queue,
            self.target.config.width,
            self.target.config.height,
        );

        {
            let mut r_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.clear_color.into()),
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            r_pass.set_pipeline(&self.pipeline);
            r_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            for (tex_id, batch) in geometry {
                if batch.vertices.is_empty() || batch.indices.is_empty() {
                    continue;
                }

                let texture = self.textures.get(tex_id).unwrap_or(&self.default_texture);
                texture.bind(&mut r_pass, 0);

                let vertex_buffer =
                    self.gpu
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&batch.vertices),
                            usage: BufferUsages::VERTEX,
                        });

                let mut index_data = bytemuck::cast_slice(&batch.indices).to_vec();
                index_data.resize((index_data.len() + 3) & !3, 0);

                let index_buffer =
                    self.gpu
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: &index_data,
                            usage: BufferUsages::INDEX,
                        });

                r_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                r_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
                r_pass.draw_indexed(0..batch.indices.len() as u32, 0, 0..1);
            }

            self.text.render(&mut r_pass);
        }

        self.gpu.queue.submit(Some(encoder.finish()));
        frame.present();
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

    /// Sets the color used to clear the screen before drawing
    pub fn clear(&mut self, color: Color) {
        self.clear_color = color;
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
    ///
    /// This id is used in drawing primitives (via `Graphics::rect().texture(id)`)
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
            &self.bind_group_layout,
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
            &self.bind_group_layout,
            data,
            w,
            h,
        );
        self.textures[index] = tex;
    }
}
