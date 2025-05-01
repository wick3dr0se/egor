pub mod app;
mod renderer;
mod time;

pub use app::App;
pub use wgpu::Color;

use renderer::Renderer;
use winit::{dpi::LogicalSize, event_loop::ActiveEventLoop, window::{Window, WindowAttributes}};

#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;
#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;

pub trait InitFn: FnOnce(&mut InitContext) + 'static {}
impl<F: FnOnce(&mut InitContext) + 'static> InitFn for F {}
pub trait UpdateFn: FnMut(&mut Context) + 'static {}
impl<F: FnMut(&mut Context) + 'static> UpdateFn for F {}

pub struct WindowBuilder {
    title: String,
    size: Option<(f32, f32)>,
}

impl WindowBuilder {
    pub fn new() -> Self {
        Self {
            title: "Egor".to_string(),
            size: None,
        }
    }

    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_string();
        self
    }

    pub fn size(&mut self, w: f32, h: f32) -> &mut Self {
        self.size = Some((w, h));
        self
    }

    pub(crate) fn build(self, event_loop: &ActiveEventLoop) -> Rc<Window> {
        let mut attrs = WindowAttributes::default()
            .with_title(self.title);
            
        if let Some((w, h)) = self.size {
            attrs = attrs.with_inner_size(LogicalSize::new(w, h));
        }            

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowAttributesExtWebSys;
            attrs = attrs.with_append(true);
        }

        Rc::new(event_loop.create_window(attrs).unwrap())
    }
}

pub struct InitContext {
    window_builder: Option<WindowBuilder>,
}

impl InitContext {
    pub fn window(&mut self) -> &mut WindowBuilder {
        self.window_builder.get_or_insert_with(WindowBuilder::new)
    }
}

pub struct Context<'a> {
    pub renderer: &'a mut Renderer,
}

impl Context<'_> {
    pub fn clear(&mut self, color: Color) {
        self.renderer.clear_color = color;
    }

    pub fn fps(&self) -> u32 {
        self.renderer.frame_timer.fps
    }
}
