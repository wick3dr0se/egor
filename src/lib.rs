pub mod app;
mod camera;
pub mod input;
mod render;

use camera::Camera;
pub use wgpu::Color;
pub use winit::keyboard::KeyCode;

use render::{
    Renderer,
    primitives::{RectangleBuilder, TriangleBuilder},
};
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;
#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;

pub struct InitContext<'a> {
    window: Rc<Window>,
    render: &'a mut Renderer,
}

impl<'a> InitContext<'a> {
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    pub fn load_texture(&mut self, data: &[u8]) -> usize {
        self.render.add_texture(data)
    }
}

pub struct GraphicsContext<'a> {
    renderer: &'a mut Renderer,
    camera: &'a mut Camera,
}

impl<'a> GraphicsContext<'a> {
    pub fn tri(&mut self) -> TriangleBuilder {
        TriangleBuilder::new(self.renderer, &self.camera)
    }

    pub fn rect(&mut self) -> RectangleBuilder {
        RectangleBuilder::new(self.renderer, &self.camera)
    }

    pub fn clear(&mut self, color: Color) {
        self.renderer.clear(color);
    }

    pub fn screen_size(&self) -> [f32; 2] {
        [
            self.renderer.screen_width() as f32,
            self.renderer.screen_height() as f32,
        ]
    }

    pub fn camera(&mut self) -> &mut Camera {
        self.camera
    }
}
