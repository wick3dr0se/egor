pub mod app;
mod render;

pub use wgpu::Color;
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;
#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;

pub struct InitContext {
    window: Rc<Window>,
}

impl InitContext {
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }
}
