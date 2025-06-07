#[cfg(feature = "windowing")]
pub mod app;
#[cfg(feature = "windowing")]
pub use app::InitContext;
#[cfg(feature = "windowing")]
pub mod input;
#[cfg(feature = "windowing")]
pub use winit::keyboard::KeyCode;

pub mod render;
mod time;
pub use wgpu::Color;

#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;

#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;
