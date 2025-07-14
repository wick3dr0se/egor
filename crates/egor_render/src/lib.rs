pub mod camera;
pub mod color;
pub mod math;
pub mod primitives;
pub mod renderer;
pub mod text;
pub mod texture;
pub mod vertex;

use glam::Vec2;

use crate::{
    camera::{Camera, CameraInternal},
    color::Color,
    primitives::RectangleBuilder,
    renderer::{GeometryBatch, RenderNodeId, RenderTargetId, Renderer},
    text::TextBuilder,
    vertex::Vertex,
};

#[derive(Clone, Copy)]
pub enum Target {
    Surface,
    Offscreen(RenderTargetId),
}

#[derive(Default)]
struct PrimitiveBatch {
    geometry: Vec<(usize, GeometryBatch)>,
}

impl PrimitiveBatch {
    // Add verts & indices to batch with matching texture_id or create a new batch
    fn push(&mut self, verts: &[Vertex], indices: &[u16], texture_id: usize) {
        if let Some((_, batch)) = self.geometry.iter_mut().find(|(id, _)| *id == texture_id) {
            batch.push(verts, indices);
        } else {
            let mut batch = GeometryBatch::default();
            batch.push(verts, indices);
            self.geometry.push((texture_id, batch));
        }
    }

    fn take(&mut self) -> Vec<(usize, GeometryBatch)> {
        std::mem::take(&mut self.geometry)
    }
}

/// High-level 2D drawing interface that simplifies the [`Renderer`]
pub struct Graphics<'a> {
    renderer: &'a mut Renderer,
    batch: PrimitiveBatch,
    camera: Camera,
    active_target: Target,
    active_target_id: Option<RenderTargetId>,
}

impl<'a> Graphics<'a> {
    /// Create new `Graphics` tied to [`Renderer`]
    pub fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            renderer,
            batch: PrimitiveBatch::default(),
            camera: Camera::default(),
            active_target: Target::Surface,
            active_target_id: None,
        }
    }

    /// Start building a rectangle primitive
    pub fn rect(&mut self) -> RectangleBuilder<'_> {
        RectangleBuilder::new(&mut self.batch)
    }

    /// Clear the screen to a color
    pub fn clear(&mut self, color: Color) {
        self.renderer.clear(color);
    }

    /// Get current surface size in pixels
    pub fn screen_size(&self) -> Vec2 {
        self.renderer.surface_size().into()
    }

    /// Mutable access to [`Camera`]
    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Draw a line of text
    pub fn text(&mut self, text: &str) -> TextBuilder<'_> {
        TextBuilder::new(&mut self.renderer.text, text.to_string())
    }

    /// Update texture data by index
    pub fn update_texture(&mut self, index: usize, data: &[u8]) {
        self.renderer.update_texture(index, data);
    }

    /// Update texture data by index with raw width/height
    pub fn update_texture_raw(&mut self, index: usize, w: u32, h: u32, data: &[u8]) {
        self.renderer.update_texture_raw(index, w, h, data);
    }

    /// Begin building a post-processing chain
    pub fn post_process(&mut self) -> PostProcessBuilder<'_> {
        PostProcessBuilder::new(self.renderer)
    }

    pub fn create_offscreen_target(&mut self, width: u32, height: u32) -> RenderTargetId {
        self.renderer.add_offscreen_target(width, height)
    }

    pub fn post_bind_group(&self, id: RenderTargetId) -> &wgpu::BindGroup {
        self.renderer.get_bind_group_for_target(id).unwrap()
    }

    pub fn create_post_pipeline(
        &self,
        shader: &wgpu::ShaderModule,
        entry: &str,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        self.renderer.create_post_pipeline(shader, entry, format)
    }

    pub fn set_post_process_order(&mut self, order: Vec<RenderNodeId>) {
        self.renderer.set_render_order(order);
    }

    pub fn run_post_processes(&mut self) {
        self.renderer.execute_render_dag();
    }

    pub fn set_active_target(&mut self, target: Target) {
        self.active_target = target;
        self.active_target_id = match target {
            Target::Surface => None,
            Target::Offscreen(id) => Some(id),
        };
    }

    pub fn active_target(&self) -> &Target {
        &self.active_target
    }
}

/// Internal trait exposing egor’s core graphics operations  
/// Allows flushing batched geometry, uploading camera matrix, etc  
/// For advanced users or `egor_render` integration; not part of public API
pub trait GraphicsInternal {
    /// Upload camera matrix & extract batched geometry for [`Renderer::render_frame()`]
    fn flush(&mut self) -> Vec<(usize, GeometryBatch)>;
}

impl GraphicsInternal for Graphics<'_> {
    fn flush(&mut self) -> Vec<(usize, GeometryBatch)> {
        self.renderer
            .upload_camera_matrix(self.camera.view_proj(self.renderer.surface_size().into()));
        self.batch.take()
    }
}
pub struct PostProcessBuilder<'a> {
    renderer: &'a mut Renderer,
    nodes: Vec<RenderNodeId>,
}

impl<'a> PostProcessBuilder<'a> {
    pub fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            renderer,
            nodes: Vec::new(),
        }
    }

    pub fn node(
        &mut self,
        pipeline: wgpu::RenderPipeline,
        bind_group: wgpu::BindGroup,
        target: RenderTargetId,
    ) -> &mut Self {
        let id = self.renderer.add_render_node(pipeline, bind_group, target);
        self.nodes.push(id);
        self
    }
}

impl Drop for PostProcessBuilder<'_> {
    fn drop(&mut self) {
        self.renderer.set_render_order(self.nodes.clone());
        self.renderer.execute_render_dag();
        self.nodes.clear();
    }
}
