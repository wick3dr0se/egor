use crate::graphics::{Graphics, GraphicsInternal};

use egor_app::{AppHandler, AppRunner, Window, WindowHandle, input::Input, time::FrameTimer};
use egor_render::Renderer;

pub struct App {
    title: String,
    vsync: bool,
    on_quit: Option<Box<dyn FnMut()>>,
    hot_reload: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            title: "Egor App".to_string(),
            vsync: true,
            on_quit: None,
            hot_reload: false,
        }
    }

    #[cfg(feature = "hot_reload")]
    pub fn with_hot_reload(mut self) -> Self {
        self.hot_reload = true;
        self
    }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }

    pub fn vsync(mut self, v: bool) -> Self {
        self.vsync = v;
        self
    }

    pub fn on_quit(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_quit = Some(Box::new(f));
        self
    }

    pub fn run<F: FnMut(&mut Graphics, &Input, &FrameTimer) + 'static>(self, update: F) {
        // Wrap the update closure in subsecond call if hot_reload is enabled
        let update = {
            let mut f = update;
            move |g: &mut Graphics, i: &Input, t: &FrameTimer| {
                if self.hot_reload {
                    #[cfg(feature = "hot_reload")]
                    {
                        use dioxus_devtools::{connect_subsecond, subsecond};
                        connect_subsecond();
                        subsecond::call(|| f(g, i, t))
                    }
                } else {
                    f(g, i, t)
                }
            }
        };

        struct Handler<F> {
            update: F,
            vsync: bool,
            on_quit: Option<Box<dyn FnMut()>>,
        }

        impl<F: FnMut(&mut Graphics, &Input, &FrameTimer)> AppHandler<Renderer> for Handler<F> {
            async fn with_resource(&mut self, window: WindowHandle) -> Renderer {
                let (w, h) = (window.inner_size().width, window.inner_size().height);
                Renderer::new(w, h, window.inner()).await
            }

            fn on_ready(&mut self, _window: &Window, r: &mut Renderer) {
                r.set_vsync(self.vsync);
            }

            fn frame(&mut self, r: &mut Renderer, i: &Input, t: &FrameTimer) {
                let mut g = Graphics::new(r);
                (self.update)(&mut g, i, t);
                let geometry = g.flush();
                r.render_frame(geometry);
            }

            fn resize(&mut self, w: u32, h: u32, r: &mut Renderer) {
                r.resize(w, h)
            }

            fn on_quit(&mut self) {
                if let Some(quit) = &mut self.on_quit {
                    quit();
                }
            }
        }

        AppRunner::new(Handler {
            update,
            vsync: self.vsync,
            on_quit: self.on_quit,
        })
        .title(self.title)
        .run();
    }
}
