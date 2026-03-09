use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    Extent3d, FilterMode, Origin3d, Queue, RenderPass, Sampler, SamplerBindingType,
    SamplerDescriptor, ShaderStages, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDimension,
};

use crate::target::OffscreenTarget;

/// A GPU texture that can be bound in shaders for rendering
///
/// Wraps a `wgpu::Texture`, its view, sampler, & bind group
pub(crate) struct Texture {
    bind_group: BindGroup,
}

impl Texture {
    fn create_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        view: &TextureView,
        sampler: &Sampler,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
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
        })
    }

    /// Creates a new texture from raw RGBA image data,
    /// uploads the data, & builds the bind group using the layout and shared sampler
    ///
    /// - `data`: Must be in tightly packed 8-bit RGBA format
    /// - `width`, `height`: Dimensions of the image in pixels
    fn from_bytes(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
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

        Self {
            bind_group: Self::create_bind_group(device, layout, &view, sampler),
        }
    }

    /// Creates a bindable texture from an existing GPU texture view.
    ///
    /// This does not allocate or upload image data.
    /// It wraps a view produced elsewhere (an offscreen render target)
    /// and builds the bind group required for sampling in shaders
    fn from_view(
        view: &TextureView,
        device: &Device,
        layout: &BindGroupLayout,
        sampler: &Sampler,
    ) -> Self {
        Self {
            bind_group: Self::create_bind_group(device, layout, view, sampler),
        }
    }

    /// Creates a 1×1 white fallback texture
    ///
    /// Used when no valid texture is provided for a draw call
    fn create_default(
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

pub(crate) struct Textures {
    layout: BindGroupLayout,
    default_sampler: Sampler,
    linear_clamp_sampler: Sampler,
    default_texture: Texture,
    store: Vec<Texture>,
}

impl Textures {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
        });

        let default_sampler = device.create_sampler(&Default::default());

        let linear_clamp_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        let default_texture = Texture::create_default(device, queue, &layout, &default_sampler);

        Self {
            layout,
            default_sampler,
            linear_clamp_sampler,
            default_texture,
            store: Vec::new(),
        }
    }

    fn decode_rgba(data: &[u8]) -> (u32, u32, image::RgbaImage) {
        let img = image::load_from_memory(data).unwrap().to_rgba8();
        let (w, h) = img.dimensions();
        (w, h, img)
    }

    pub fn get(&self, id: Option<usize>) -> &Texture {
        id.and_then(|i| self.store.get(i))
            .unwrap_or(&self.default_texture)
    }

    pub fn insert(&mut self, device: &Device, queue: &Queue, data: &[u8]) -> usize {
        let (w, h, img) = Self::decode_rgba(data);
        self.insert_raw(device, queue, w, h, &img)
    }

    pub fn insert_raw(
        &mut self,
        device: &Device,
        queue: &Queue,
        w: u32,
        h: u32,
        data: &[u8],
    ) -> usize {
        let id = self.store.len();
        self.store.push(Texture::from_bytes(
            device,
            queue,
            &self.layout,
            &self.default_sampler,
            data,
            w,
            h,
        ));
        id
    }

    pub fn replace(&mut self, device: &Device, queue: &Queue, id: usize, data: &[u8]) {
        let (w, h, img) = Self::decode_rgba(data);
        self.replace_raw(device, queue, id, w, h, &img);
    }

    pub fn replace_raw(
        &mut self,
        device: &Device,
        queue: &Queue,
        id: usize,
        w: u32,
        h: u32,
        data: &[u8],
    ) {
        self.store[id] = Texture::from_bytes(
            device,
            queue,
            &self.layout,
            &self.default_sampler,
            data,
            w,
            h,
        );
    }

    pub fn insert_offscreen(&mut self, device: &Device, offscreen: &OffscreenTarget) -> usize {
        let id = self.store.len();
        self.store.push(Texture::from_view(
            offscreen.view(),
            device,
            &self.layout,
            &self.linear_clamp_sampler,
        ));
        id
    }
}
