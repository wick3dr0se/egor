#[cfg(feature = "windowing")]
pub mod app;
#[cfg(feature = "windowing")]
pub mod input;

mod render;
mod time;

pub use wgpu::Color;

#[cfg(feature = "windowing")]
pub use winit::keyboard::KeyCode;

pub use render::{Graphics, Renderer, camera::*, primitives::*};

#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;

#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;
