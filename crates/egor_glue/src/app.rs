use std::sync::Arc;

use crate::{graphics::Graphics, text::TextRenderer};

#[cfg(feature = "ui")]
use crate::ui::EguiRenderer;

use egor_app::{
    AppConfig, AppHandler, AppRunner, Window, WindowEvent, input::Input, time::FrameTimer,
};
use egor_render::Renderer;

#[cfg(not(feature = "ui"))]
type UpdateFn = dyn FnMut(&mut Graphics, &Input, &FrameTimer);
#[cfg(feature = "ui")]
type UpdateFn = dyn FnMut(&mut Graphics, &Input, &FrameTimer, &egui::Context);

pub struct App {
    update: Option<Box<UpdateFn>>,
    config: Option<AppConfig>,
    on_quit: Option<Box<dyn FnMut()>>,
    vsync: bool,
    text_renderer: Option<TextRenderer>,
    #[cfg(feature = "ui")]
    egui: Option<EguiRenderer>,
}

impl App {
    /// Create a new [`App`]
    pub fn new() -> Self {
        Self {
            update: None,
            config: Some(AppConfig::default()),
            on_quit: None,
            vsync: true,
            text_renderer: None,
            #[cfg(feature = "ui")]
            egui: None,
        }
    }

    /// Set application title
    pub fn title(mut self, title: &str) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.title = title.into();
        }
        self
    }

    /// Set window size (width, height in pixels)
    pub fn screen_size(mut self, width: u32, height: u32) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.width = width;
            c.height = height;
        }
        self
    }

    /// Enable or disable window resizing (defaults to true)
    pub fn resizable(mut self, resizable: bool) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.resizable = resizable;
        }
        self
    }

    /// Enable or disable vsync
    pub fn vsync(mut self, enabled: bool) -> Self {
        self.vsync = enabled;
        self
    }

    /// Run the app with a per-frame update closure
    #[cfg(not(feature = "ui"))]
    pub fn run(
        mut self,
        #[allow(unused_mut)] mut update: impl FnMut(&mut Graphics, &Input, &FrameTimer) + 'static,
    ) {
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
    #[cfg(feature = "ui")]
    pub fn run(
        mut self,
        #[allow(unused_mut)] mut update: impl FnMut(&mut Graphics, &Input, &FrameTimer, &egui::Context)
        + 'static,
    ) {
        #[cfg(feature = "hot_reload")]
        let update = {
            dioxus_devtools::connect_subsecond();

            move |g: &mut Graphics, i: &Input, t: &FrameTimer, ui: &egui::Context| {
                dioxus_devtools::subsecond::call(|| update(g, i, t, ui))
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
    fn on_window_event(&mut self, _window: &Window, _event: &WindowEvent) {
        #[cfg(feature = "ui")]
        if let Some(egui) = self.egui.as_mut() {
            egui.handle_event(_window, _event);
        }
    }

    async fn with_resource(&mut self, window: Arc<Window>) -> Renderer {
        let (w, h) = (window.inner_size().width, window.inner_size().height);
        Renderer::new(w, h, window).await
    }

    fn on_ready(&mut self, window: &Window, renderer: &mut Renderer) {
        renderer.set_vsync(self.vsync);

        #[cfg(target_arch = "wasm32")]
        renderer.resize(window.inner_size().width, window.inner_size().height);

        self.text_renderer = Some(TextRenderer::new(
            renderer.device(),
            renderer.queue(),
            renderer.surface_format(),
        ));

        #[cfg(feature = "ui")]
        {
            let (device, format) = (renderer.device(), renderer.surface_format());
            self.egui = Some(EguiRenderer::new(device, format, window));
        }
    }

    fn frame(
        &mut self,
        _window: &Window,
        renderer: &mut Renderer,
        input: &Input,
        timer: &FrameTimer,
    ) {
        let Some(update) = &mut self.update else {
            return;
        };
        let text_renderer = self.text_renderer.as_mut().unwrap();

        let (width, height) = (
            renderer.surface_config().width,
            renderer.surface_config().height,
        );
        let (device, queue) = (renderer.device().clone(), renderer.queue().clone());
        let mut frame = renderer.begin_frame().unwrap();
        let mut graphics = Graphics::new(renderer, text_renderer);

        #[cfg(not(feature = "ui"))]
        update(&mut graphics, input, timer);
        #[cfg(feature = "ui")]
        {
            let egui_ctx = self.egui.as_mut().unwrap().begin_frame(_window);
            update(&mut graphics, input, timer, egui_ctx);
        }

        let geometry = graphics.flush();
        text_renderer.prepare(&device, &queue, width, height);

        {
            let mut r_pass = renderer.begin_render_pass(&mut frame.encoder, &frame.view);

            for (tex_id, batch) in &geometry {
                renderer.draw_batch(&mut r_pass, batch, *tex_id);
            }

            text_renderer.render(&mut r_pass);
        }

        #[cfg(feature = "ui")]
        {
            let render_data = self.egui.as_mut().unwrap().end_frame(_window);
            self.egui.as_mut().unwrap().render(
                &device,
                &queue,
                &mut frame.encoder,
                &frame.view,
                width,
                height,
                render_data,
            );
        }

        renderer.end_frame(frame);
    }

    fn resize(&mut self, width: u32, height: u32, renderer: &mut Renderer) {
        renderer.resize(width, height)
    }

    fn on_quit(&mut self) {
        if let Some(f) = &mut self.on_quit {
            f();
        }
    }
}
