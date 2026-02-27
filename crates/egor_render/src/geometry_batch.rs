use wgpu::{Buffer, BufferUsages, Device, IndexFormat, Queue, RenderPass};

use crate::{instance::Instance, vertex::Vertex};

/// A batch of geometry (vertices + indices) that can be drawn in a single GPU call
///
/// Tracks CPU vertex/index data, lazily uploads GPU buffers and prevents overflowing `u16` indices.
/// Supports two draw paths:
/// - Baked geometry (vertices + indices) for paths, polygons, arbitrary meshes
/// - Instanced drawing (instance buffer) for quads/rects/sprites via a static unit quad
pub struct GeometryBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    vertices_dirty: bool,
    indices_dirty: bool,
    instances: Vec<Instance>,
    instance_buffer: Option<Buffer>,
    instances_dirty: bool,
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
            instances: Vec::new(),
            instance_buffer: None,
            instances_dirty: false,
        }
    }
}

impl GeometryBatch {
    const MAX_VERTICES: usize = u16::MAX as usize;
    const MAX_INDICES: usize = Self::MAX_VERTICES * 6;
    const MAX_INSTANCES: usize = 10_000;

    // Returns true if adding verts/indices would exceed max allowed
    pub fn would_overflow(&self, vert_count: usize, idx_count: usize) -> bool {
        self.vertices.len() + vert_count > Self::MAX_VERTICES
            || self.indices.len() + idx_count > Self::MAX_INDICES
    }

    // Returns true if an instance is full
    pub fn instances_full(&self) -> bool {
        self.instances.len() >= Self::MAX_INSTANCES
    }

    /// Reserves space for `vert_count` + `idx_count`
    ///
    /// Returns mutable slices to the new ranges and the base vertex offset.
    /// Returns `None` if this would exceed `u16` limits.
    /// Marks buffers dirty
    pub fn try_allocate(
        &mut self,
        vert_count: usize,
        idx_count: usize,
    ) -> Option<(&mut [Vertex], &mut [u16], u16)> {
        if self.would_overflow(vert_count, idx_count) {
            return None;
        }

        let v_start = self.vertices.len();
        let i_start = self.indices.len();

        self.vertices.resize(v_start + vert_count, Vertex::zeroed());
        self.indices.resize(i_start + idx_count, 0);

        self.vertices_dirty = true;
        self.indices_dirty = true;

        Some((
            &mut self.vertices[v_start..],
            &mut self.indices[i_start..],
            v_start as u16,
        ))
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

    /// Pushes an instance for instanced drawing
    pub fn push_instance(&mut self, instance: Instance) {
        if self.instances.len() < Self::MAX_INSTANCES {
            self.instances.push(instance);
            self.instances_dirty = true;
        }
    }

    /// Returns true if there is nothing to draw in either path
    pub(crate) fn is_empty(&self) -> bool {
        self.indices.is_empty() && self.instances.is_empty()
    }

    /// Clears CPU-side geometry and instances, keeps buffer allocations for reuse
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.instances.clear();
        self.vertices_dirty = true;
        self.indices_dirty = true;
        self.instances_dirty = true;
    }

    // Uploads buffers to GPU only if needed
    pub(crate) fn upload(&mut self, device: &Device, queue: &Queue) {
        if !self.vertices_dirty && !self.indices_dirty && !self.instances_dirty {
            return;
        }

        if self.vertices_dirty && !self.vertices.is_empty() {
            if self.vertex_buffer.is_none() {
                self.vertex_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("GeometryBatch Vertex Buffer"),
                    size: (Self::MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64,
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
            }
            queue.write_buffer(
                self.vertex_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&self.vertices),
            );
            self.vertices_dirty = false;
        }

        if self.indices_dirty && !self.indices.is_empty() {
            if self.index_buffer.is_none() {
                self.index_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("GeometryBatch Index Buffer"),
                    size: (Self::MAX_INDICES * std::mem::size_of::<u16>()) as u64,
                    usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
            }

            let mut indices_bytes = bytemuck::cast_slice(&self.indices).to_vec();
            let remainder = indices_bytes.len() % wgpu::COPY_BUFFER_ALIGNMENT as usize;
            if remainder != 0 {
                indices_bytes.extend_from_slice(&vec![
                    0u8;
                    wgpu::COPY_BUFFER_ALIGNMENT as usize
                        - remainder
                ]);
            }
            queue.write_buffer(self.index_buffer.as_ref().unwrap(), 0, &indices_bytes);
            self.indices_dirty = false;
        }

        if self.instances_dirty && !self.instances.is_empty() {
            if self.instance_buffer.is_none() {
                self.instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("GeometryBatch Instance Buffer"),
                    size: (Self::MAX_INSTANCES * std::mem::size_of::<Instance>()) as u64,
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
            }
            queue.write_buffer(
                self.instance_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&self.instances),
            );
            self.instances_dirty = false;
        }
    }

    /// Draws baked geometry and/or instanced quads as separate draw calls
    pub(crate) fn draw(
        &self,
        r_pass: &mut RenderPass,
        quad_vb: &Buffer,
        quad_ib: &Buffer,
        dummy_instance: &Buffer,
    ) {
        if !self.instances.is_empty() {
            if let Some(instance_buf) = &self.instance_buffer {
                r_pass.set_vertex_buffer(0, quad_vb.slice(..));
                r_pass.set_vertex_buffer(1, instance_buf.slice(..));
                r_pass.set_index_buffer(quad_ib.slice(..), IndexFormat::Uint16);
                r_pass.draw_indexed(0..6, 0, 0..self.instances.len() as u32);
            }
        }

        if !self.indices.is_empty() {
            if let (Some(vb), Some(ib)) = (&self.vertex_buffer, &self.index_buffer) {
                r_pass.set_vertex_buffer(0, vb.slice(..));
                r_pass.set_vertex_buffer(1, dummy_instance.slice(..));
                r_pass.set_index_buffer(ib.slice(..), IndexFormat::Uint16);
                r_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
            }
        }
    }
}
