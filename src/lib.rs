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
    pub use egor_render::{Anchor, Color, Graphics, Renderer};
}

#[cfg(feature = "render")]
pub mod math {
    pub use egor_render::{Rect, Vec2, vec2};
}
