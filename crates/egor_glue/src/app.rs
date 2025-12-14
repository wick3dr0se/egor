use crate::graphics::{Graphics, GraphicsInternal};

use egor_app::{
    AppConfig, AppHandler, AppRunner, Window, WindowHandle, input::Input, time::FrameTimer,
};
use egor_render::Renderer;

type UpdateCallback = dyn FnMut(&mut Graphics, &Input, &FrameTimer);

pub struct App {
    update: Option<Box<UpdateCallback>>,
    config: Option<AppConfig>,
    on_quit: Option<Box<dyn FnMut()>>,
    vsync: bool,
}

impl App {
    /// Create a new [`App`]
    pub fn new() -> Self {
        Self {
            update: None,
            config: Some(AppConfig::default()),
            on_quit: None,
            vsync: true,
        }
    }

    /// Set application title
    pub fn title(mut self, title: &str) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.title = title.into();
        }
        self
    }

    /// Enable or disable vsync
    pub fn vsync(mut self, enabled: bool) -> Self {
        self.vsync = enabled;
        self
    }

    /// Run the app with a per-frame update closure
    #[allow(unused_mut)]
    pub fn run(mut self, mut update: impl FnMut(&mut Graphics, &Input, &FrameTimer) + 'static) {
        #[cfg(feature = "hot_reload")]
        let update = {
            dioxus_devtools::connect_subsecond();
            move |g: &mut Graphics, i: &Input, t: &FrameTimer| {
                dioxus_devtools::subsecond::call(|| update(g, i, t))
            }
        };
        self.update = Some(Box::new(update));

        let config = self.config.take().unwrap();
        AppRunner::new(self, config).run();
    }

    /// Sets a closure to call when the app is quitting
    pub fn on_quit(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_quit = Some(Box::new(f));
        self
    }
}

impl AppHandler<Renderer> for App {
    async fn with_resource(&mut self, window: WindowHandle) -> Renderer {
        let (w, h) = (window.inner_size().width, window.inner_size().height);
        Renderer::new(w, h, window.to_owned()).await
    }

    fn on_ready(&mut self, _window: &Window, r: &mut Renderer) {
        r.set_vsync(self.vsync)
    }

    fn frame(&mut self, r: &mut Renderer, i: &Input, t: &FrameTimer) {
        if let Some(update) = &mut self.update {
            let mut g = Graphics::new(r);
            update(&mut g, i, t);
            let geometry = g.flush();
            r.render_frame(geometry);
        }
    }

    fn resize(&mut self, w: u32, h: u32, r: &mut Renderer) {
        r.resize(w, h)
    }

    fn on_quit(&mut self) {
        if let Some(f) = &mut self.on_quit {
            f();
        }
    }
}
