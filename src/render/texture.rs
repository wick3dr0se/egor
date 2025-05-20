use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Device, Extent3d, Origin3d, Queue,
    RenderPass, Sampler, SamplerBindingType, ShaderStages, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture as WgpuTexture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDimension,
};

#[derive(Debug)]
pub struct Texture {
    pub texture: WgpuTexture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub bind_group: BindGroup,
}

impl Texture {
    pub fn from_bytes(
        device: &Device,
        queue: &Queue,
        bind_group_layout: &BindGroupLayout,
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
        let sampler = device.create_sampler(&Default::default());
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
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture,
            view,
            sampler,
            bind_group,
        }
    }

    pub fn create_default(device: &Device, queue: &Queue, layout: &BindGroupLayout) -> Self {
        let white_pixel = [255u8, 255, 255, 255];
        Self::from_bytes(device, queue, layout, &white_pixel, 1, 1)
    }

    pub fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
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

    pub fn bind<'a>(&'a self, pass: &mut RenderPass<'a>, index: u32) {
        pass.set_bind_group(index, &self.bind_group, &[]);
    }
}
