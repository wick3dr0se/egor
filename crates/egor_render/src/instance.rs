use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

/// Per-instance data for 2D instanced drawing (56 bytes)
///
/// Uses a compact 2D affine representation instead of a full `mat4x4`:
/// - `affine`: column-major 2×2 rotation+scale matrix `[col0.x, col0.y, col1.x, col1.y]`
/// - `translate`: world-space translation `[x, y]`
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub affine: [f32; 4],
    pub translate: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 4],
}

impl Instance {
    pub fn desc() -> VertexBufferLayout<'static> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                // affine (2×2 matrix)
                VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: VertexFormat::Float32x4,
                },
                // translate
                VertexAttribute {
                    offset: 16,
                    shader_location: 4,
                    format: VertexFormat::Float32x2,
                },
                // color
                VertexAttribute {
                    offset: 24,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
                // uv rect
                VertexAttribute {
                    offset: 40,
                    shader_location: 6,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }

    pub fn identity() -> Self {
        Self {
            affine: [1.0, 0.0, 0.0, 1.0],
            translate: [0.0, 0.0],
            color: [1.0; 4],
            uv: [0.0, 0.0, 1.0, 1.0],
        }
    }
}
