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
