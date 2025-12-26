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
//! - [`egor_app`] — windowing, input, & event loop
//! - [`egor_glue`] - high-level wrappers over egor crates

pub mod app {
    pub use egor_app::AppConfig;
    pub use egor_glue::app::App;
    pub use egor_glue::ui::egui;
}

pub mod input {
    pub use egor_app::input::{Input, InputInternal, KeyCode, MouseButton};
}

pub mod time {
    pub use egor_app::time::{FrameTimer, FrameTimerInternal};
}

pub mod render {
    pub use egor_glue::{
        camera::CameraInternal, graphics::Graphics, graphics::GraphicsInternal, primitives::Anchor,
    };
    pub use egor_render::{Renderer, color::Color};
}

pub mod math {
    pub use egor_render::math::{Rect, Vec2, vec2};
}
