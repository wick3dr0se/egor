//! # Egor
//!
//! A dead simple cross-platform 2D graphics engine
//!
//! `egor` is a minimal, modular toolkit for building 2D applications, games, or tools   
//! It avoids heavy abstractions & gives you direct, high-level control over rendering,
//! input, timing, the event loop & more
//!
//! `egor` is a meta crate that re-exports commonly used `egor_*` crates for convenience:
//! - [`egor_render`] — WGPU-based 2D rendering
//! - [`egor_app`] — windowing, input, & main loop
#[cfg(feature = "app")]
pub mod app {
    pub use egor_app::{App, Context, InitContext, Plugin};
}

#[cfg(feature = "app")]
pub mod input {
    pub use egor_app::input::{Input, KeyCode, MouseButton};

    #[cfg(not(feature = "render"))]
    pub use egor_app::{input::InputInternal, time::FrameTimerInternal};
}

#[cfg(feature = "render")]
pub mod render {
    pub use egor_render::{Graphics, color::Color, primitives::Anchor, renderer::Renderer};

    #[cfg(not(feature = "app"))]
    pub use egor_render::{GraphicsInternal, camera::CameraInternal};
}

#[cfg(feature = "render")]
pub mod math {
    pub use egor_render::math::{Rect, Vec2, vec2};
}
