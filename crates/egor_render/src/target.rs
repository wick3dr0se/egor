use wgpu::{
    Adapter, BindGroupLayout, Device, Extent3d, Instance, PresentMode, Surface,
    SurfaceConfiguration, SurfaceTarget, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, WindowHandle,
};

use crate::{frame::Presentable, texture::Texture};

/// Trait for render targets (backbuffers, offscreen textures, etc.)
pub trait RenderTarget {
    fn format(&self) -> TextureFormat;
    fn size(&self) -> (u32, u32);
    /// Returns the view and optionally something that must be presented (swapchain)
    fn acquire(&mut self) -> Option<(TextureView, Option<Box<dyn Presentable>>)>;
    fn resize(&mut self, device: &Device, w: u32, h: u32);
    /// Only useful for backbuffer targets
    fn set_vsync(&mut self, _device: &Device, _on: bool) {}
}

/// Renders to the window's backbuffer (swapchain)
pub struct Backbuffer {
    surface: Surface<'static>,
    config: SurfaceConfiguration,
}

impl Backbuffer {
    pub fn new(
        instance: &Instance,
        adapter: &Adapter,
        device: &Device,
        window: impl Into<SurfaceTarget<'static>> + WindowHandle,
        w: u32,
        h: u32,
    ) -> Self {
        let surface = instance.create_surface(window).unwrap();
        let mut config = surface.get_default_config(adapter, w, h).unwrap();
        config.present_mode = PresentMode::AutoVsync;
        surface.configure(device, &config);
        Self { surface, config }
    }
}

impl RenderTarget for Backbuffer {
    fn format(&self) -> TextureFormat {
        self.config.format
    }

    fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    fn acquire(&mut self) -> Option<(TextureView, Option<Box<dyn Presentable>>)> {
        let surface_texture = self.surface.get_current_texture().ok()?;
        let view = surface_texture.texture.create_view(&Default::default());
        Some((view, Some(Box::new(surface_texture))))
    }

    fn resize(&mut self, device: &Device, w: u32, h: u32) {
        (self.config.width, self.config.height) = (w, h);
        self.surface.configure(device, &self.config);
    }

    fn set_vsync(&mut self, device: &Device, on: bool) {
        self.config.present_mode = if on {
            PresentMode::Fifo
        } else {
            PresentMode::AutoNoVsync
        };
        self.surface.configure(device, &self.config);
    }
}

/// Renders to an offscreen texture that can be read back or used as a texture
pub struct OffscreenTarget {
    texture: wgpu::Texture,
    view: TextureView,
    format: TextureFormat,
    width: u32,
    height: u32,
}

impl OffscreenTarget {
    pub fn new(device: &Device, width: u32, height: u32, format: TextureFormat) -> Self {
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
            format,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC, // for reading back pixels if needed
            view_formats: &[],
        });

        let view = texture.create_view(&Default::default());

        Self {
            texture,
            view,
            format,
            width,
            height,
        }
    }

    pub fn as_texture(&self, device: &Device, layout: &BindGroupLayout) -> Texture {
        Texture::from_view(&self.view, device, layout)
    }

    /// Get the underlying texture for binding as a shader resource
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// Get a view to the texture
    pub fn view(&self) -> &TextureView {
        &self.view
    }
}

impl RenderTarget for OffscreenTarget {
    fn format(&self) -> TextureFormat {
        self.format
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn acquire(&mut self) -> Option<(TextureView, Option<Box<dyn Presentable>>)> {
        // no presentation needed for offscreen targets
        Some((self.view.clone(), None))
    }

    fn resize(&mut self, device: &Device, w: u32, h: u32) {
        if self.width == w && self.height == h {
            return;
        }
        // recreate the texture with new dimensions
        *self = Self::new(device, w, h, self.format);
    }
}
