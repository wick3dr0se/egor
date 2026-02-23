use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device,
    Extent3d, Origin3d, Queue, RenderPass, Sampler, TexelCopyBufferLayout, TexelCopyTextureInfo,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
};

/// A GPU texture that can be bound in shaders for rendering
///
/// Wraps a `wgpu::Texture`, its view, sampler, & bind group
pub struct Texture {
    bind_group: BindGroup,
}

impl Texture {
    /// Creates a new texture from raw RGBA image data,
    /// uploads the data, & builds the bind group using the layout and shared sampler
    ///
    /// - `data`: Must be in tightly packed 8-bit RGBA format
    /// - `width`, `height`: Dimensions of the image in pixels
    pub fn from_bytes(
        device: &Device,
        queue: &Queue,
        bind_group_layout: &BindGroupLayout,
        sampler: &Sampler,
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
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        });

        Self { bind_group }
    }

    /// Creates a bindable texture from an existing GPU texture view.
    ///
    /// This does not allocate or upload image data.
    /// It wraps a view produced elsewhere (an offscreen render target)
    /// and builds the bind group required for sampling in shaders
    pub fn from_view(
        view: &TextureView,
        device: &Device,
        layout: &BindGroupLayout,
        sampler: &Sampler,
    ) -> Self {
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        });

        Self { bind_group }
    }

    /// Creates a 1×1 white fallback texture
    ///
    /// Used when no valid texture is provided for a draw call
    pub fn create_default(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        sampler: &Sampler,
    ) -> Self {
        Self::from_bytes(
            device,
            queue,
            layout,
            sampler,
            &[255u8, 255, 255, 255],
            1,
            1,
        )
    }

    /// Binds this texture at the given index in the render pass
    ///
    /// - `index` must match the bind group index used in the pipeline layout
    pub fn bind(&self, pass: &mut RenderPass, index: u32) {
        pass.set_bind_group(index, &self.bind_group, &[]);
    }
}
