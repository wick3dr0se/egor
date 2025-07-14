use wgpu::{
    Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferUsages, ColorTargetState, ColorWrites, CompositeAlphaMode, Device,
    DeviceDescriptor, Extent3d, FragmentState, IndexFormat, Instance, Limits, LoadOp, Operations,
    PipelineLayoutDescriptor, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler, ShaderModule,
    ShaderStages, StoreOp, Surface, SurfaceConfiguration, SurfaceTarget, SurfaceTexture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDimension, VertexState, WindowHandle, include_wgsl, util::DeviceExt,
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

#[derive(Clone, Copy)]
pub struct RenderTargetId(pub usize);
#[derive(Clone, Copy)]
pub struct RenderNodeId(pub usize);

pub struct RenderNode {
    pub pipeline: RenderPipeline,
    pub bind_group: BindGroup,
    pub target: RenderTargetId,
}

enum RenderTargetKind {
    Surface {
        surface: Surface<'static>,
    },
    Offscreen {
        texture: wgpu::Texture,
        view: TextureView,
        sampler: Sampler,
        bind_group: BindGroup,
    },
}

pub struct RenderTarget {
    kind: RenderTargetKind,
    config: SurfaceConfiguration,
}

impl RenderTarget {
    fn from_surface(
        instance: Instance,
        window: impl Into<SurfaceTarget<'static>> + WindowHandle + 'static,
        adapter: &Adapter,
        device: &Device,
        width: u32,
        height: u32,
    ) -> Self {
        let surface = instance.create_surface(window).unwrap();
        let config = surface.get_default_config(adapter, width, height).unwrap();
        surface.configure(device, &config);

        Self {
            kind: RenderTargetKind::Surface { surface },
            config,
        }
    }

    fn from_offscreen(
        device: &Device,
        bind_group_layout: &BindGroupLayout,
        format: TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Offscreen Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&Default::default());
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Offscreen Bind Group"),
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: PresentMode::AutoVsync, // not presented
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        Self {
            kind: RenderTargetKind::Offscreen {
                texture,
                view,
                sampler,
                bind_group,
            },
            config,
        }
    }

    fn resize(
        &mut self,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) {
        match &mut self.kind {
            RenderTargetKind::Surface { surface } => {
                self.config.width = width;
                self.config.height = height;
                surface.configure(device, &self.config);
            }
            RenderTargetKind::Offscreen { .. } => {
                *self = Self::from_offscreen(
                    device,
                    bind_group_layout,
                    self.config.format,
                    width,
                    height,
                );
            }
        }
    }

    fn get_view(&self) -> wgpu::TextureView {
        match &self.kind {
            RenderTargetKind::Surface { surface } => {
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to get surface texture");
                frame.texture.create_view(&Default::default())
            }
            RenderTargetKind::Offscreen { view, .. } => view.clone(),
        }
    }

    pub fn bind_group(&self) -> Option<&wgpu::BindGroup> {
        match &self.kind {
            RenderTargetKind::Offscreen { bind_group, .. } => Some(bind_group),
            _ => None,
        }
    }

    fn surface(&self) -> Option<&Surface<'static>> {
        match &self.kind {
            RenderTargetKind::Surface { surface } => Some(surface),
            _ => None,
        }
    }

    pub fn acquire_frame(&self) -> Option<wgpu::SurfaceTexture> {
        match &self.kind {
            RenderTargetKind::Surface { surface } => surface.get_current_texture().ok(),
            _ => None,
        }
    }
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
    post_targets: Vec<RenderTarget>,
    render_nodes: Vec<RenderNode>,
    render_order: Vec<RenderNodeId>,
    pipeline: RenderPipeline,
    clear_color: Color,
    texture_bind_group_layout: BindGroupLayout,
    camera_bind_group: BindGroup,
    camera_buffer: Buffer,
    textures: Vec<(Texture, BindGroup)>,
    default_texture: (Texture, BindGroup),
    sampler: Sampler,
    fullscreen_vertex_buffer: Buffer,
    fullscreen_index_buffer: Buffer,
    fullscreen_index_count: u32,
    pub(crate) text: TextRenderer,
}

impl Renderer {
    /// Creates a new `Renderer` with a configured surface, pipeline & default resources
    ///
    /// Initializes `wgpu`, sets up a basic alpha-blended render pipeline, default texture,
    /// camera uniform, internal text renderer & more
    pub async fn create_graphics(
        inner_width: u32,
        inner_height: u32,
        window: impl Into<SurfaceTarget<'static>> + WindowHandle + 'static + Clone,
    ) -> Self {
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

        // WebGPU throws error 'size is zero' if not set
        let (w, h) = (inner_width.max(1), inner_height.max(1));

        let mut surface_cfg = surface.get_default_config(&adapter, w, h).unwrap();
        surface_cfg.present_mode = PresentMode::AutoVsync;
        surface.configure(&device, &surface_cfg);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let sampler = device.create_sampler(&Default::default());

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

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
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

        let default_texture = Texture::create_default(&device, &queue);
        let default_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Default Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&default_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let text = TextRenderer::new(&device, &queue, surface_cfg.format);

        let fullscreen_vertices = [
            Vertex::new([-1.0, -1.0], Color::WHITE, [0.0, 1.0]),
            Vertex::new([1.0, -1.0], Color::WHITE, [1.0, 1.0]),
            Vertex::new([1.0, 1.0], Color::WHITE, [1.0, 0.0]),
            Vertex::new([-1.0, 1.0], Color::WHITE, [0.0, 0.0]),
        ];

        let fullscreen_indices: &[u16] = &[0, 1, 2, 2, 3, 0];

        let fullscreen_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Fullscreen Quad Vertex Buffer"),
                contents: bytemuck::cast_slice(&fullscreen_vertices),
                usage: BufferUsages::VERTEX,
            });

        let fullscreen_index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Fullscreen Quad Index Buffer"),
                contents: bytemuck::cast_slice(fullscreen_indices),
                usage: BufferUsages::INDEX,
            });

        let fullscreen_index_count = fullscreen_indices.len() as u32;

        Self {
            gpu: Gpu {
                device: device.clone(),
                queue,
            },
            target: RenderTarget::from_surface(instance, window, &adapter, &device, w, h),
            post_targets: Vec::new(),
            render_nodes: Vec::new(),
            render_order: Vec::new(),
            pipeline,
            clear_color: Color::BLACK,
            texture_bind_group_layout,
            camera_bind_group,
            camera_buffer,
            textures: Vec::new(),
            sampler,
            default_texture: (default_texture, default_bind_group),
            fullscreen_vertex_buffer,
            fullscreen_index_buffer,
            fullscreen_index_count,
            text,
        }
    }

    pub fn get_bind_group_for_target(&self, id: RenderTargetId) -> Option<&BindGroup> {
        self.post_targets.get(id.0)?.bind_group()
    }

    pub fn create_post_pipeline(
        &self,
        shader: &ShaderModule,
        entry: &str,
        format: TextureFormat,
    ) -> RenderPipeline {
        let layout = self
            .gpu
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Post-Process Pipeline Layout"),
                bind_group_layouts: &[&self.texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        self.gpu
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Post-Process Pipeline"),
                layout: Some(&layout),
                vertex: VertexState {
                    module: shader,
                    entry_point: Some(entry),
                    buffers: &[Vertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(FragmentState {
                    module: shader,
                    entry_point: Some(entry),
                    targets: &[Some(ColorTargetState {
                        format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
    }

    /// Get the TextureView for a given RenderTargetId
    pub fn get_render_target_view(&self, id: RenderTargetId) -> TextureView {
        self.post_targets[id.0].get_view()
    }

    /// create & add an offscreen target, return its id
    pub fn add_offscreen_target(&mut self, width: u32, height: u32) -> RenderTargetId {
        let target = RenderTarget::from_offscreen(
            &self.gpu.device,
            &self.texture_bind_group_layout,
            self.target.config.format,
            width,
            height,
        );
        let id = RenderTargetId(self.post_targets.len());
        self.post_targets.push(target);
        id
    }

    /// add a render node (post-processing pass) using a pipeline, bind group & target
    pub fn add_render_node(
        &mut self,
        pipeline: RenderPipeline,
        bind_group: BindGroup,
        target: RenderTargetId,
    ) -> RenderNodeId {
        let id = RenderNodeId(self.render_nodes.len());
        self.render_nodes.push(RenderNode {
            pipeline,
            bind_group,
            target,
        });
        id
    }

    /// Set the order to execute render nodes (simple linear DAG)
    pub fn set_render_order(&mut self, order: Vec<RenderNodeId>) {
        self.render_order = order;
    }

    /// execute render nodes in order, rendering fullscreen quads to their targets
    pub fn execute_render_dag(&mut self) {
        for &node_id in &self.render_order {
            let node = &self.render_nodes[node_id.0];
            let target = &self.post_targets[node.target.0];
            let view = target.get_view();

            let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
            {
                let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Post-process Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(self.clear_color.into()),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                rpass.set_pipeline(&node.pipeline);
                rpass.set_bind_group(0, &node.bind_group, &[]);
                rpass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
                rpass.set_index_buffer(self.fullscreen_index_buffer.slice(..), IndexFormat::Uint16);
                rpass.draw_indexed(0..self.fullscreen_index_count, 0, 0..1);
            }
            self.gpu.queue.submit(Some(encoder.finish()));
        }
    }

    pub fn render_frame_to_view(
        &mut self,
        view: &TextureView,
        geometry: Vec<(usize, GeometryBatch)>,
    ) {
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
                    view,
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

                let bind_group = self
                    .textures
                    .get(tex_id)
                    .map(|(_, bg)| bg)
                    .unwrap_or(&self.default_texture.1);
                r_pass.set_bind_group(0, bind_group, &[]);

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
    }

    /// Renders a frame using the given geometry batches grouped by texture ID
    ///
    /// Each `(usize, GeometryBatch)` tuple represents a texture index & associated geometry  
    /// Text is rendered afterward automatically  
    /// Rendered directly to the surface target
    pub fn render_frame(&mut self, geometry: Vec<(usize, GeometryBatch)>) {
        let frame = self.target.acquire_frame().unwrap();
        let view = frame.texture.create_view(&Default::default());

        self.render_frame_to_view(&view, geometry);

        frame.present();
    }

    /// Resizes the surface & updates internal render targets
    pub fn resize(&mut self, w: u32, h: u32) {
        (self.target.config.width, self.target.config.height) = (w, h);

        if let Some(surface) = self.target.surface() {
            surface.configure(&self.gpu.device, &self.target.config);
        }

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

        if let Some(surface) = self.target.surface() {
            surface.configure(&self.gpu.device, &self.target.config);
        }
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
        let tex = Texture::from_bytes(&self.gpu.device, &self.gpu.queue, data, w, h);
        let bind_group = self.gpu.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&tex.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        let texture_idx = self.textures.len();
        self.textures.push((tex, bind_group));
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
        let tex = Texture::from_bytes(&self.gpu.device, &self.gpu.queue, data, w, h);
        let bind_group = self.gpu.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Updated Texture Bind Group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&tex.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        self.textures[index] = (tex, bind_group);
    }
}
