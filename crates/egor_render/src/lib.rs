pub mod batch;
pub mod frame;
pub mod instance;
mod pipeline;
pub mod target;
mod texture;
mod uniforms;
pub mod vertex;

pub use wgpu::{Device, MemoryHints, Queue, RenderPass, TextureFormat};

use std::cell::RefCell;
use std::rc::Rc;
use wgpu::{
    Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor,
    BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT, Color, CommandEncoder, CommandEncoderDescriptor,
    DeviceDescriptor, Extent3d, Instance, InstanceDescriptor, LoadOp, MapMode, Operations,
    Origin3d, PollType, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions,
    StoreOp, SurfaceTarget, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo,
    TextureAspect, TextureView, WindowHandle,
    util::{BufferInitDescriptor, DeviceExt, new_instance_with_webgpu_detection},
};

pub use texture::{Texture, TextureId};

use crate::{
    batch::{BatchKey, GeometryBatch},
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

/// Axis-aligned clip rectangle in framebuffer pixels, with a top-left origin.
///
/// Used to restrict a draw call to a sub-region of the target. Coordinates match
/// Egor's screen space (y grows downward from the top-left corner), so a value
/// derived from a screen-space [`crate::Renderer`] draw maps directly.
/// Pixels to write into an existing texture
#[derive(Clone, Copy)]
pub struct TextureRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Axis-aligned clip rectangle in framebuffer pixels, with a top-left origin.
///
/// Used to restrict a draw call to a sub-region of the target. Coordinates match
/// Egor's screen space (y grows downward from the top-left corner), so a value
/// derived from a screen-space [`crate::Renderer`] draw maps directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClipRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ClipRect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// The coordinate space a batch is drawn in.
///
/// - [`Space::World`] applies the frame's camera view-projection, so geometry
///   participates in camera panning & zooming.
/// - [`Space::Screen`] ignores the camera & maps pixels directly to the target,
///   which is what UI and overlays need so they stay put while the world moves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Space {
    #[default]
    World,
    Screen,
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
    screen_bind_group: BindGroup,
    screen_buffer: Buffer,
    target_size: std::cell::Cell<(u32, u32)>,
    surface_format: TextureFormat,
    uniforms: Uniforms,
    textures: Textures,
    freed: Rc<RefCell<Vec<TextureId>>>,
    clear_color: Color,
}

impl Renderer {
    /// Creates a renderer & initializes GPU state using the window's surface
    ///
    /// Sets up wgpu, pipelines, default texture & camera resources
    pub async fn new(
        window: impl Into<SurfaceTarget<'static>> + WindowHandle,
        memory_hints: &MemoryHints,
    ) -> Self {
        let instance = new_instance_with_webgpu_detection(
            InstanceDescriptor::new_without_display_handle_from_env(),
        )
        .await;
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
                memory_hints: memory_hints.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let surface_config = surface.get_default_config(&adapter, 1, 1).unwrap();
        Self::assemble(
            Gpu {
                instance,
                adapter,
                device,
                queue,
            },
            surface_config.format,
        )
    }

    /// Builds a renderer that never opens a window, for tests and headless hosts
    ///
    /// Frames render into an [`OffscreenTarget`]
    /// and come back through [`Self::read_target`].
    pub fn headless(width: u32, height: u32, memory_hints: &MemoryHints) -> Self {
        pollster::block_on(async {
            let instance = new_instance_with_webgpu_detection(
                InstanceDescriptor::new_without_display_handle_from_env(),
            )
            .await;
            let adapter = instance
                .request_adapter(&RequestAdapterOptions::default())
                .await
                .unwrap();

            let (device, queue) = adapter
                .request_device(&DeviceDescriptor {
                    #[cfg(target_arch = "wasm32")]
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    memory_hints: memory_hints.clone(),
                    ..Default::default()
                })
                .await
                .unwrap();

            let renderer = Self::assemble(
                Gpu {
                    instance,
                    adapter,
                    device,
                    queue,
                },
                TextureFormat::Rgba8UnormSrgb,
            );
            renderer.set_target_size(width, height);
            renderer
        })
    }

    fn assemble(gpu: Gpu, surface_format: TextureFormat) -> Self {
        #[cfg(feature = "log")]
        {
            let info = gpu.adapter.get_info();
            log::info!(
                "adapter: {} ({:?}, {:?}, driver {})",
                info.name,
                info.backend,
                info.device_type,
                info.driver
            );
        }

        let device = &gpu.device;
        let queue = &gpu.queue;
        let pipelines = Pipelines::new(device, surface_format);

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

        // Screen-space counterpart to the world camera. Holds an identity-space
        // orthographic matrix (re-uploaded on resize) so UI/overlay batches can
        // draw in raw pixels while world batches keep the camera transform.
        let screen_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Screen Space Uniform"),
            contents: bytemuck::bytes_of(&CameraUniform::default()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let screen_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Screen Space Bind Group"),
            layout: &pipelines.camera_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });

        let uniforms = Uniforms::new(device);
        let textures = Textures::new(device, queue);

        Renderer {
            gpu,
            pipelines,
            quad_vertex_buffer,
            quad_index_buffer,
            dummy_instance_buffer,
            camera_bind_group,
            camera_buffer,
            screen_bind_group,
            screen_buffer,
            target_size: std::cell::Cell::new((1, 1)),
            surface_format,
            uniforms,
            textures,
            freed: Rc::new(RefCell::new(Vec::new())),
            clear_color: Color::BLACK,
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

        // drops land after the submit, so a texture used by this frame's batches is
        // only released once they have been drawn
        let freed: Vec<TextureId> = self.freed.borrow_mut().drain(..).collect();
        for id in freed {
            self.textures.free(id);
        }
    }

    /// Copies an offscreen target back as RGBA8 rows, top row first
    ///
    /// Each row is tightly packed, so the returned length is `width * height * 4`
    /// whatever the copy alignment was.
    pub fn read_target(&self, target: &OffscreenTarget) -> Vec<u8> {
        let (width, height) = target.size();
        let unpadded = width * 4;
        let padded = unpadded.div_ceil(COPY_BYTES_PER_ROW_ALIGNMENT) * COPY_BYTES_PER_ROW_ALIGNMENT;

        let buffer = self.gpu.device.create_buffer(&BufferDescriptor {
            label: Some("Target Readback"),
            size: u64::from(padded) * u64::from(height),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture: target.attachment(),
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: &buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.gpu.queue.submit(Some(encoder.finish()));

        let slice = buffer.slice(..);
        slice.map_async(MapMode::Read, |_| {});
        self.gpu
            .device
            .poll(PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .unwrap();

        let mapped = slice.get_mapped_range().unwrap();
        let mut pixels = Vec::with_capacity((unpadded * height) as usize);
        for row in 0..height {
            let start = (row * padded) as usize;
            pixels.extend_from_slice(&mapped[start..start + unpadded as usize]);
        }
        drop(mapped);
        buffer.unmap();

        pixels
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
                depth_slice: None,
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
    ///
    /// `clip` restricts drawing to a sub-region of the target (scissor test); pass
    /// `None` to draw to the full target. `space` selects the transform the batch
    /// is drawn with (world camera vs. raw screen pixels).
    pub fn draw_batch(
        &self,
        r_pass: &mut RenderPass<'_>,
        batch: &mut GeometryBatch,
        key: BatchKey,
    ) {
        if batch.is_empty() {
            return;
        }

        batch.upload(&self.gpu.device, &self.gpu.queue);

        let texture = self.textures.get(key.texture);
        texture.bind(r_pass, 0);

        let (pipeline, uniform_ids) = self.pipelines.resolve(key.shader);

        r_pass.set_pipeline(pipeline);

        let camera = match key.space {
            Space::World => &self.camera_bind_group,
            Space::Screen => &self.screen_bind_group,
        };
        r_pass.set_bind_group(1, camera, &[]);

        for (i, &uid) in uniform_ids.iter().enumerate() {
            r_pass.set_bind_group((2 + i) as u32, self.uniforms.bind_group(uid), &[]);
        }

        self.set_scissor(r_pass, key.clip);

        batch.draw(
            r_pass,
            &self.quad_vertex_buffer,
            &self.quad_index_buffer,
            &self.dummy_instance_buffer,
        );
        batch.clear();
    }

    /// Sets the scissor rect for subsequent draws in the pass.
    ///
    /// `None` resets it to the full target size (tracked via [`Self::set_target_size`]).
    pub fn set_scissor(&self, r_pass: &mut RenderPass<'_>, clip: Option<ClipRect>) {
        let (tw, th) = self.target_size.get();
        let rect = clip.unwrap_or(ClipRect::new(0, 0, tw.max(1), th.max(1)));
        // Clamp into the target; a zero-area scissor is invalid so keep at least 1px
        let x = rect.x.min(tw);
        let y = rect.y.min(th);
        let width = rect.width.min(tw.saturating_sub(x)).max(1);
        let height = rect.height.min(th.saturating_sub(y)).max(1);
        r_pass.set_scissor_rect(x, y, width, height);
    }

    /// Records the current target size so [`Self::set_scissor`] can reset to full
    pub fn set_target_size(&self, width: u32, height: u32) {
        self.target_size.set((width, height));
    }

    /// Uploads the given view-projection matrix to the GPU for use in vertex transforms
    pub fn upload_camera_matrix(&mut self, view_proj: [[f32; 4]; 4]) {
        self.gpu.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&CameraUniform { view_proj }),
        );
    }

    /// Uploads the screen-space (pixel-aligned) matrix used by [`Space::Screen`] batches
    pub fn upload_screen_matrix(&mut self, view_proj: [[f32; 4]; 4]) {
        self.gpu.queue.write_buffer(
            &self.screen_buffer,
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
    ) -> OffscreenTarget {
        OffscreenTarget::new(&self.gpu.device, width, height, format)
    }

    /// Adds an offscreen target texture, freed when the handle drops
    pub fn add_offscreen_texture(&mut self, offscreen: &mut OffscreenTarget) -> Texture {
        let id = self.textures.insert_offscreen(&self.gpu.device, offscreen);
        Texture::new(id, &self.freed)
    }

    /// Adds a new texture from image bytes, freed when the handle drops
    pub fn add_texture(&mut self, data: &[u8]) -> Texture {
        let id = self
            .textures
            .insert(&self.gpu.device, &self.gpu.queue, data);
        Texture::new(id, &self.freed)
    }

    /// Adds a texture from raw RGBA bytes, freed when the handle drops
    pub fn add_texture_raw(&mut self, w: u32, h: u32, data: &[u8]) -> Texture {
        let id = self
            .textures
            .insert_raw(&self.gpu.device, &self.gpu.queue, w, h, data);
        Texture::new(id, &self.freed)
    }

    /// Write a region of an existing texture, rows packed
    pub fn write_texture_region(&self, id: TextureId, region: TextureRegion, data: &[u8]) {
        self.textures
            .write_region(&self.gpu.queue, id, region, data);
    }

    /// Replaces an existing texture with new image data
    pub fn update_texture(&mut self, id: TextureId, data: &[u8]) {
        self.textures
            .replace(&self.gpu.device, &self.gpu.queue, id, data);
    }

    /// Replaces an existing texture with raw RGBA bytes
    pub fn update_texture_raw(&mut self, id: TextureId, w: u32, h: u32, data: &[u8]) {
        self.textures
            .replace_raw(&self.gpu.device, &self.gpu.queue, id, w, h, data);
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
