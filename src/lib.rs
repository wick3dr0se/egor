#[cfg(feature = "app")]
pub mod app {
    pub use egor_app::{App, Context, InitContext, Plugin};
}

#[cfg(feature = "app")]
pub mod input {
    pub use egor_app::input::{Input, KeyCode, MouseButton};
}

#[cfg(feature = "render")]
pub mod render {
    pub use egor_render::{Graphics, Renderer, color::Color, primitives::Anchor};

    #[cfg(not(feature = "app"))]
    pub use egor_render::GraphicsInternal;
}

#[cfg(feature = "render")]
pub mod math {
    pub use egor_render::math::{Rect, Vec2, vec2};
}
