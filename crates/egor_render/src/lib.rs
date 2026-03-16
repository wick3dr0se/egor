pub mod batch;
pub mod frame;
pub mod instance;
mod pipeline;
pub mod target;
mod texture;
mod uniforms;
pub mod vertex;

pub use wgpu::{Device, Queue, RenderPass, TextureFormat};

use wgpu::{
    Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, Color,
    CommandEncoder, DeviceDescriptor, Instance, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, StoreOp, SurfaceTarget, TextureView, WindowHandle,
    util::{BufferInitDescriptor, DeviceExt, new_instance_with_webgpu_detection},
};

/// Controls the memory/performance tradeoff for `wgpu` device allocation and egor's batch vertex buffers.
///
/// Roughly based on <https://wgpu.rs/doc/wgpu/enum.MemoryHints.html> with an
/// extra knob for egor's batch vertex memory usage.
#[derive(Clone)]
pub enum MemoryHints {
    /// Uses higher memory (larger buffers) for better performance; default
    Performance,
    /// Uses less memory (smaller buffers) for better compatibility on low-end
    /// devices. Good for small UI libraries or low-res games
    MemoryUsage,
    /// Manual control over both wgpu sub-allocation block sizes and egor batch
    /// vertex limits
    Manual {
        max_verticies_per_batch: usize,
        max_indices_per_batch: Option<usize>,
        suballocated_device_memory_block_size: Option<std::ops::Range<u64>>,
    },
}

impl MemoryHints {
    pub fn max_verticies_per_batch(&self) -> usize {
        match self {
            MemoryHints::Performance => GeometryBatch::DEFAULT_MAX_VERTICES,
            MemoryHints::MemoryUsage => GeometryBatch::DEFAULT_MAX_VERTICES / 4,
            MemoryHints::Manual {
                max_verticies_per_batch,
                ..
            } => *max_verticies_per_batch,
        }
    }

    pub fn max_indices_per_batch(&self) -> usize {
        match self {
            MemoryHints::Performance => GeometryBatch::DEFAULT_MAX_INDICES,
            MemoryHints::MemoryUsage => GeometryBatch::DEFAULT_MAX_INDICES / 4,
            MemoryHints::Manual {
                max_verticies_per_batch,
                max_indices_per_batch,
                ..
            } => max_indices_per_batch
                .unwrap_or(max_verticies_per_batch * GeometryBatch::DEFAULT_INDICES_PER_VERT),
        }
    }
}

impl From<&MemoryHints> for wgpu::MemoryHints {
    fn from(hints: &MemoryHints) -> Self {
        match hints {
            MemoryHints::Performance => wgpu::MemoryHints::Performance,
            MemoryHints::MemoryUsage => wgpu::MemoryHints::MemoryUsage,
            MemoryHints::Manual {
                suballocated_device_memory_block_size,
                ..
            } => {
                let mb = 1024 * 1024;

                // Same as wgpu defaults, but exposed here for manual configuration or reference
                let perf_cfg = (128 * mb)..(512 * mb);
                let _mem_usage_cfg = (8 * mb)..(64 * mb);

                let suballocated_device_memory_block_size = suballocated_device_memory_block_size
                    .as_ref()
                    .unwrap_or(&perf_cfg)
                    .clone();

                wgpu::MemoryHints::Manual {
                    suballocated_device_memory_block_size,
                }
            }
        }
    }
}

use crate::{
    batch::GeometryBatch,
    frame::Frame,
    pipeline::Pipelines,
    target::{OffscreenTarget, RenderTarget},
    texture::Textures,
    uniforms::{CameraUniform, Uniforms},
    vertex::{QUAD_INDICES, QUAD_VERTICES},
};

pub(crate) struct Gpu {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

/// Low-level GPU renderer built on `wgpu`
///
/// Handles rendering pipelines, surface configuration, resources (textures, buffers), & drawing
pub struct Renderer {
    gpu: Gpu,
    pipelines: Pipelines,
    quad_vertex_buffer: Buffer,
    quad_index_buffer: Buffer,
    dummy_instance_buffer: Buffer,
    camera_bind_group: BindGroup,
    camera_buffer: Buffer,
    surface_format: TextureFormat,
    uniforms: Uniforms,
    textures: Textures,
    clear_color: Color,
    memory_hints: MemoryHints,
}

impl Renderer {
    /// Creates a renderer & initializes GPU state using the window's surface
    ///
    /// Sets up wgpu, pipelines, default texture & camera resources
    pub async fn new(
        window: impl Into<SurfaceTarget<'static>> + WindowHandle,
        memory_hints: MemoryHints,
    ) -> Self {
        let instance = new_instance_with_webgpu_detection(&Default::default()).await;
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                // Required for WebGL to prevent selecting a non-presentable device
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                #[cfg(target_arch = "wasm32")]
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                memory_hints: wgpu::MemoryHints::from(&memory_hints),
                ..Default::default()
            })
            .await
            .unwrap();

        let surface_config = surface.get_default_config(&adapter, 1, 1).unwrap();
        let surface_format = surface_config.format;
        let pipelines = Pipelines::new(&device, surface_format);

        let quad_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Static Unit Quad VB"),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let quad_index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Static Unit Quad IB"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: BufferUsages::INDEX,
        });
        let dummy_instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Dummy Instance Buffer"),
            contents: bytemuck::bytes_of(&instance::Instance::identity()),
            usage: BufferUsages::VERTEX,
        });
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&CameraUniform::default()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipelines.camera_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let uniforms = Uniforms::new(&device);
        let textures = Textures::new(&device, &queue);

        Renderer {
            gpu: Gpu {
                instance,
                adapter,
                device,
                queue,
            },
            pipelines,
            quad_vertex_buffer,
            quad_index_buffer,
            dummy_instance_buffer,
            camera_bind_group,
            camera_buffer,
            surface_format,
            uniforms,
            textures,
            clear_color: Color::BLACK,
            memory_hints,
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

    /// Sets the clear color for future render passes
    pub fn set_clear_color(&mut self, color: [f64; 4]) {
        self.clear_color = Color {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        };
    }

    /// Begins a frame with the given render target
    pub fn begin_frame(&mut self, target: &mut dyn RenderTarget) -> Option<Frame> {
        let (view, presentable) = target.acquire(&self.gpu.device)?;
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
        texture_id: Option<usize>,
        shader_id: Option<usize>,
    ) {
        if batch.is_empty() {
            return;
        }

        batch.upload(&self.gpu.device, &self.gpu.queue);

        let texture = self.textures.get(texture_id);
        texture.bind(r_pass, 0);

        let (pipeline, uniform_ids) = self.pipelines.resolve(shader_id);

        r_pass.set_pipeline(pipeline);
        r_pass.set_bind_group(1, &self.camera_bind_group, &[]);

        for (i, &uid) in uniform_ids.iter().enumerate() {
            r_pass.set_bind_group((2 + i) as u32, self.uniforms.bind_group(uid), &[]);
        }

        batch.draw(
            r_pass,
            &self.quad_vertex_buffer,
            &self.quad_index_buffer,
            &self.dummy_instance_buffer,
        );
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

    /// Create an offscreen render target
    pub fn create_offscreen_target(
        &self,
        width: u32,
        height: u32,
        format: TextureFormat,
        memory_hints: Option<MemoryHints>,
    ) -> OffscreenTarget {
        let memory_hints = memory_hints.unwrap_or(self.memory_hints.clone());
        OffscreenTarget::new(&self.gpu.device, width, height, format, memory_hints)
    }

    /// Adds an offscreen target texture & returns its id
    pub fn add_offscreen_texture(&mut self, offscreen: &mut OffscreenTarget) -> usize {
        self.textures.insert_offscreen(&self.gpu.device, offscreen)
    }

    /// Adds a new texture from image bytes & returns its id
    pub fn add_texture(&mut self, data: &[u8]) -> usize {
        self.textures
            .insert(&self.gpu.device, &self.gpu.queue, data)
    }

    /// Adds a texture from raw RGBA bytes & returns its id
    pub fn add_texture_raw(&mut self, w: u32, h: u32, data: &[u8]) -> usize {
        self.textures
            .insert_raw(&self.gpu.device, &self.gpu.queue, w, h, data)
    }

    /// Replaces an existing texture with new image data
    pub fn update_texture(&mut self, index: usize, data: &[u8]) {
        self.textures
            .replace(&self.gpu.device, &self.gpu.queue, index, data);
    }

    /// Replaces an existing texture with raw RGBA bytes
    pub fn update_texture_raw(&mut self, index: usize, w: u32, h: u32, data: &[u8]) {
        self.textures
            .replace_raw(&self.gpu.device, &self.gpu.queue, index, w, h, data);
    }

    /// Creates a uniform buffer and returns its id
    pub fn add_uniform(&mut self, data: &[u8]) -> usize {
        self.uniforms.insert(&self.gpu.device, data)
    }

    /// Updates an existing uniform buffer with new data
    pub fn update_uniform(&mut self, id: usize, data: &[u8]) {
        self.uniforms.write(&self.gpu.queue, id, data);
    }

    /// Creates a custom shader pipeline from WGSL source code
    /// Returns the pipeline index for use in draw calls
    pub fn add_shader(&mut self, wgsl_source: &str) -> usize {
        self.pipelines
            .add_custom(&self.gpu.device, self.surface_format, wgsl_source, &[], &[])
    }

    /// Creates a custom shader pipeline with associated uniform buffers
    ///
    /// `uniform_ids` specify which renderer uniform buffers should be bound
    /// after the built-in texture and camera bind groups when this shader is used
    ///
    /// Returns the pipeline index for use in draw calls
    pub fn add_shader_with_uniforms(&mut self, wgsl_source: &str, uniform_ids: &[usize]) -> usize {
        let layouts = vec![self.uniforms.layout(); uniform_ids.len()];
        self.pipelines.add_custom(
            &self.gpu.device,
            self.surface_format,
            wgsl_source,
            &layouts,
            uniform_ids,
        )
    }
}
