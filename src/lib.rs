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
//! use egor::{app::{App, FrameContext}, render::Graphics};
//! App::new().run(|FrameContext { gfx, .. }| {
//!     // start building a rectangle with some defaults
//!     // draws automatically on `Drop` without an explicit `build()`
//!     gfx.rect();
//! });
//! ```
//!
//! ## Crate Layout
//! `egor` is the API crate: drawing, camera, text and the frame loop live here.
//! - [`egor_render`] - WGPU-based 2D rendering
//! - [`egor_app`] - windowing, input, & event loop
//!
//! `egor_glue` was folded into this crate and stops at 0.10.0

//! ## Cargo Features
//! Feature | Description | Default
//! ---|---|---
//! `log` | Enable logging via `egor_app/log` | opt-in
//! `hot_reload` | Live code patching through subsecond and `dx serve` | opt-in
//! `ui`         | Enable egui integration via `ui` | opt-in
//! `webgl`      | WebGL backend for `egor_render` | opt-in
//! `angle`      | ANGLE backend for `egor_render` | opt-in
//! `gles`       | OpenGL ES backend for `egor_render` | opt-in
//! `vulkan`     | Vulkan backend for `egor_render` | Linux default/opt-in
//!
//! Notes:
//! - Windows builds use DX12 by default, Linux builds use Vulkan by default, etc
//! - Optional backends can be enabled to override defaults or for cross-platform targeting

mod camera;
mod color;
mod graphics;
mod primitives;
mod text;

#[cfg(feature = "ui")]
mod ui;

pub mod app;
pub mod math;

#[cfg(all(feature = "hot_reload", not(target_arch = "wasm32")))]
pub mod hot {
    //! Primitives for keeping state across hot patches, so a changed struct is not
    //! read with its old layout. Diff [`HotFnPtr`] from [`HotFn::ptr_address`] each
    //! frame and rebuild when it changes.
    pub use dioxus_devtools::subsecond::{HotFn, HotFnPtr};
}

pub mod input {
    pub use egor_app::input::{Input, KeyCode, MouseButton};
}

pub mod time {
    pub use egor_app::time::FrameTimer;
}

pub mod render {
    pub use crate::{
        camera::Camera,
        color::Color,
        graphics::Graphics,
        primitives::{BorderRadii, Pivot},
        text::Align,
    };
    pub use egor_render::{
        ClipRect, MemoryHints, Space, Texture,
        target::{OffscreenTarget, RenderTarget},
    };
}

#[macro_export]
/// Invoke this by passing your main function as an argument.
/// Ensures unusual platforms like android get initialized properly.
macro_rules! main {
    ($main:expr) => {
        #[cfg(target_os = "android")]
        #[unsafe(no_mangle)]
        fn android_main(app: ::egor::app::AndroidApp) {
            let _ = ::egor::app::ANDROID_APP.set(app);
            ($main)();
        }
    };
}
