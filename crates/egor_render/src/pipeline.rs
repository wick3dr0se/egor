use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
    ColorTargetState, ColorWrites, Device, FragmentState, PipelineLayoutDescriptor, RenderPipeline,
    RenderPipelineDescriptor, SamplerBindingType, ShaderStages, TextureFormat, TextureSampleType,
    TextureViewDimension, VertexState, include_wgsl,
};

use crate::vertex::Vertex;

/// Contains all render pipelines and bind group layouts for [`Renderer`]
///
/// Centralizes GPU pipeline configuration, including:
/// - The main primitive rendering pipeline (textured quads, sprites, shapes)
/// - Texture bind group layout (for sampling textures in shaders)
/// - Camera bind group layout (for view/projection transforms)
pub struct Pipelines {
    pub primitive: RenderPipeline,
    pub texture_layout: BindGroupLayout,
    pub camera_layout: BindGroupLayout,
}

impl Pipelines {
    /// Creates all pipelines and bind group layouts for the given device and surface format
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        let texture_layout = create_texture_bind_group_layout(device);
        let camera_layout = create_camera_bind_group_layout(device);

        let primitive =
            create_primitive_pipeline(device, surface_format, &texture_layout, &camera_layout);

        Self {
            primitive,
            texture_layout,
            camera_layout,
        }
    }
}

/// Creates the bind group layout for texture sampling
///
/// Defines two bindings:
/// - Binding 0: 2D texture (fragment shader)
/// - Binding 1: Sampler (fragment shader)
fn create_texture_bind_group_layout(device: &Device) -> BindGroupLayout {
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
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// Creates the bind group layout for camera uniforms
///
/// Defines a single binding:
/// - Binding 0: Uniform buffer containing view-projection matrix (vertex shader)
fn create_camera_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
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
    })
}

/// Creates the main rendering pipeline for 2D primitives
///
/// Configured with:
/// - Alpha blending for transparency
/// - Vertex shader transforms using camera uniform
/// - Fragment shader samples from texture
/// - `Vertex` buffer layout from vertex module
fn create_primitive_pipeline(
    device: &Device,
    surface_format: TextureFormat,
    texture_layout: &BindGroupLayout,
    camera_layout: &BindGroupLayout,
) -> RenderPipeline {
    let shader = device.create_shader_module(include_wgsl!("../shader.wgsl"));

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Primitive Pipeline Layout"),
        bind_group_layouts: &[texture_layout, camera_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Primitive Pipeline"),
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
                format: surface_format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview: None,
        cache: None,
    })
}
