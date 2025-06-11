#[cfg(feature = "app")]
pub mod app {
    pub use egor_app::App;
}

#[cfg(feature = "app")]
pub mod input {
    pub use egor_app::input::{KeyCode, MouseButton};
}

#[cfg(feature = "render")]
pub mod render {
    pub use egor_render::{Anchor, Color, Graphics, Renderer};
}
