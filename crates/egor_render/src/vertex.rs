use bytemuck::{Pod, Zeroable};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

/// A single vertex used in rendering 2D primitives
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    /// Creates a new vertex with position, color, & texture coordinates
    ///
    /// - `position`: `[x, y]` in world space
    /// - `color`: RGBA color
    /// - `tex_coords`: `[u, v]` in normalized (0–1) texture space
    pub const fn new(position: [f32; 2], color: [f32; 4], tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            color,
            tex_coords,
        }
    }

    pub(crate) fn zeroed() -> Self {
        Zeroable::zeroed()
    }

    /// Returns the vertex buffer layout
    ///
    /// This must match the vertex shader input layout:
    /// - location 0: `vec2<f32>` (position)
    /// - location 1: `vec4<f32>` (color)
    /// - location 2: `vec2<f32>` (texture coordinates)
    pub(crate) fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: 32,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub(crate) const QUAD_VERTICES: [Vertex; 4] = [
    Vertex::new([-0.5, -0.5], [1.0; 4], [0.0, 0.0]),
    Vertex::new([0.5, -0.5], [1.0; 4], [1.0, 0.0]),
    Vertex::new([0.5, 0.5], [1.0; 4], [1.0, 1.0]),
    Vertex::new([-0.5, 0.5], [1.0; 4], [0.0, 1.0]),
];
pub(crate) const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
