//! # Egor
//! A dead simple cross-platform 2D graphics engine
//!
//! ## Why Egor?
//! Egor is dead simple, lightweight and cross-platform.
//! The same code runs on native and web (WASM) with minimal boilerplate.
//! It’s built from small, composable crates on top of modern graphics
//! and windowing abstractions
//!
//! Egor gives you the essentials for 2D apps and games:
//! - Efficient 2D rendering (shapes, textures, text)
//! - Keyboard & mouse input
//! - Camera & world-space transforms
//! - Optional egui integration for tools and UIs
//! - Optional hot-reload during development
//!
//! ## Start Here
//! These are the main types Egor users interact with:
//! - [`app::App`] - application lifecycle and main loop
//! - [`render::Graphics`] - high-level 2D drawing API
//! - [`time::FrameTimer`] - frame timing and delta time
//! - [`input::Input`] - keyboard and mouse state
//!
//! ## Minimal Example: Draw a Rectangle
//! ```no_run
//! use egor::{app::App, render::Graphics};
//! App::new().run(|gfx: &mut Graphics, _input, _timer| {
//!     // start building a rectangle with some defaults
//!     // draws automatically on `Drop` without an explicit `build()`
//!     gfx.rect();
//! });
//! ```
//!
//! ## Crate Layout
//! `egor` is a meta crate that re-exports `egor_*` crates for convenience:
//! - [`egor_render`] - WGPU-based 2D rendering
//! - [`egor_app`] - windowing, input, & event loop
//! - [`egor_glue`] - opinionated layer over egor crates

//! ## Cargo Features
//! Feature | Description | Default
//! ---|---|---
//! `log` | Enable logging via `egor_app/log` | opt-in
//! `hot_reload` | Hot-reload support via `egor_glue/hot_reload` | opt-in
//! `ui`         | Enable egui integration via `egor_glue/ui` | opt-in
//! `webgl`      | WebGL backend for `egor_render` | opt-in
//! `angle`      | ANGLE backend for `egor_render` | opt-in
//! `gles`       | OpenGL ES backend for `egor_render` | opt-in
//! `vulkan`     | Vulkan backend for `egor_render` | Linux default/opt-in
//!
//! Notes:
//! - Windows builds use DX12 by default, Linux builds use Vulkan by default, etc
//! - Optional backends can be enabled to override defaults or for cross-platform targeting

pub mod app {
    pub use egor_glue::app::{App, FrameContext};
    #[cfg(feature = "ui")]
    pub use egor_glue::ui::egui;
}

pub mod input {
    pub use egor_app::input::{Input, KeyCode, MouseButton};
}

pub mod time {
    pub use egor_app::time::FrameTimer;
}

pub mod render {
    pub use egor_glue::{color::Color, graphics::Graphics, primitives::Anchor};
}

pub mod math {
    pub use egor_render::math::{IVec2, Rect, Vec2, ivec2, vec2};
}
