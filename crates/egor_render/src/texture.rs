use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Device,
    Extent3d, Origin3d, Queue, RenderPass, SamplerBindingType, ShaderStages, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDimension,
};

/// A GPU texture that can be bound in shaders for rendering
pub struct Texture {
    texture: wgpu::Texture,
    pub view: TextureView,
}

impl Texture {
    /// Creates a new texture from raw RGBA image data,
    /// uploads the data, & builds the bind group using the layout
    ///
    /// - `data`: Must be in tightly packed 8-bit RGBA format
    /// - `width`, `height`: Dimensions of the image in pixels
    pub fn from_bytes(
        device: &Device,
        queue: &Queue,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&Default::default());

        Self { texture, view }
    }

    /// Creates a 1×1 white fallback texture
    ///
    /// Used when no valid texture is provided for a draw call
    pub fn create_default(device: &Device, queue: &Queue) -> Self {
        Self::from_bytes(device, queue, &[255, 255, 255, 255], 1, 1)
    }
}
