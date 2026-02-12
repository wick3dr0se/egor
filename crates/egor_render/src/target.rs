use wgpu::{
    Adapter, Device, Instance, PresentMode, Surface, SurfaceConfiguration, SurfaceTarget,
    TextureFormat, TextureView, WindowHandle,
};

use crate::frame::Presentable;

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
