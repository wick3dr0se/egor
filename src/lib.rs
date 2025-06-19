#[cfg(feature = "app")]
pub mod app {
    pub use egor_app::{App, Context, InitContext, Plugin};
}

#[cfg(feature = "app")]
pub mod input {
    pub use egor_app::input::{Input, KeyCode, MouseButton};
}

#[cfg(all(feature = "rand", feature = "app"))]
pub use rand;

#[cfg(feature = "render")]
pub mod render {
    pub use egor_render::{Graphics, Renderer, color::Color, primitives::Anchor};
}

#[cfg(feature = "render")]
pub mod math {
    pub use egor_render::math::{Rect, Vec2, vec2};
}
