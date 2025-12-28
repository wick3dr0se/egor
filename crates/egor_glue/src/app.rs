use std::sync::Arc;

use crate::graphics::Graphics;

#[cfg(feature = "ui")]
use crate::ui::EguiRenderer;

use egor_app::{
    AppConfig, AppHandler, AppRunner, Window, WindowEvent, input::Input, time::FrameTimer,
};
use egor_render::Renderer;

#[cfg(not(feature = "ui"))]
pub trait UpdateCallback: FnMut(&mut Graphics, &Input, &FrameTimer) + 'static {}
#[cfg(not(feature = "ui"))]
impl<F: FnMut(&mut Graphics, &Input, &FrameTimer) + 'static> UpdateCallback for F {}
#[cfg(feature = "ui")]
pub trait UpdateCallback:
    FnMut(&mut Graphics, &Input, &FrameTimer, &egui::Context) + 'static
{
}
#[cfg(feature = "ui")]
impl<F: FnMut(&mut Graphics, &Input, &FrameTimer, &egui::Context) + 'static> UpdateCallback for F {}

pub struct App {
    update: Option<Box<dyn UpdateCallback>>,
    config: Option<AppConfig>,
    on_quit: Option<Box<dyn FnMut()>>,
    vsync: bool,
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

    /// Enable or disable vsync
    pub fn vsync(mut self, enabled: bool) -> Self {
        self.vsync = enabled;
        self
    }

    /// Run the app with a per-frame update closure
    pub fn run(mut self, update: impl UpdateCallback) {
        #[allow(unused_mut)]
        let mut update: Box<dyn UpdateCallback> = Box::new(update);
        #[cfg(all(not(target_arch = "wasm32"), feature = "hot_reload"))]
        {
            dioxus_devtools::connect_subsecond();

            update = Box::new({
                #[cfg(not(feature = "ui"))]
                {
                    move |g: &mut Graphics, i: &Input, t: &FrameTimer| {
                        dioxus_devtools::subsecond::call(|| update(g, i, t))
                    }
                }

                #[cfg(feature = "ui")]
                {
                    move |g: &mut Graphics, i: &Input, t: &FrameTimer, ui: &egui::Context| {
                        dioxus_devtools::subsecond::call(|| update(g, i, t, ui))
                    }
                }
            });
        }
        self.update = Some(update);

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

    fn on_ready(&mut self, _window: &Window, renderer: &mut Renderer) {
        renderer.set_vsync(self.vsync);

        #[cfg(feature = "ui")]
        {
            let (device, format) = (renderer.device(), renderer.surface_format());
            self.egui = Some(EguiRenderer::new(device, format, _window));
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

        let (width, height) = (
            renderer.surface_config().width,
            renderer.surface_config().height,
        );
        let (device, queue) = (renderer.device().clone(), renderer.queue().clone());

        let mut frame = renderer.begin_frame().unwrap();
        renderer.text.prepare(&device, &queue, width, height);

        let mut graphics = Graphics::new(renderer);

        #[cfg(not(feature = "ui"))]
        update(&mut graphics, input, timer);
        #[cfg(feature = "ui")]
        {
            let egui_ctx = self.egui.as_mut().unwrap().begin_frame(_window);
            update(&mut graphics, input, timer, egui_ctx);
        }

        let geometry = graphics.flush();

        {
            let mut r_pass = renderer.begin_render_pass(&mut frame.encoder, &frame.view);

            for (tex_id, batch) in &geometry {
                renderer.draw_batch(&mut r_pass, batch, *tex_id);
            }

            renderer.text.render(&mut r_pass);
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
