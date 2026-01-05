use bytemuck::{Pod, Zeroable};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

/// A single vertex used in rendering 2D primitives
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
    tex_coords: [f32; 2],
}

impl Vertex {
    /// Creates a new vertex with position, color, & texture coordinates
    ///
    /// - `position`: `[x, y]` in world space
    /// - `color`: RGBA color
    /// - `tex_coords`: `[u, v]` in normalized (0–1) texture space
    pub fn new(position: [f32; 2], color: [f32; 4], tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            color,
            tex_coords,
        }
    }

    /// Returns the vertex buffer layout
    ///
    /// This must match the vertex shader input layout:
    /// - location 0: `vec2<f32>` (position)
    /// - location 1: `vec4<f32>` (color)
    /// - location 2: `vec2<f32>` (texture coordinates)
    pub fn desc() -> VertexBufferLayout<'static> {
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
