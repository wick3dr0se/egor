use std::sync::Arc;

use crate::{graphics::Graphics, text::TextRenderer};

#[cfg(feature = "ui")]
use crate::ui::EguiRenderer;

use egor_app::{
    AppConfig, AppHandler, AppRunner, Window, WindowEvent, input::Input, time::FrameTimer,
};
use egor_render::Renderer;

type UpdateFn = dyn FnMut(&mut FrameContext);

pub struct FrameContext<'a> {
    pub events: Vec<WindowEvent>,
    pub gfx: Graphics<'a>,
    pub input: &'a Input,
    pub timer: &'a FrameTimer,
    #[cfg(feature = "ui")]
    pub egui_ctx: &'a egui::Context,
}

pub struct App {
    events: Vec<WindowEvent>,
    update: Option<Box<UpdateFn>>,
    config: Option<AppConfig>,
    vsync: bool,
    text_renderer: Option<TextRenderer>,
    #[cfg(feature = "ui")]
    egui: Option<EguiRenderer>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new [`App`]
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            update: None,
            config: Some(AppConfig::default()),
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
    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.width = Some(width);
            c.height = Some(height);
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
    pub fn run(mut self, #[allow(unused_mut)] mut update: impl FnMut(&mut FrameContext) + 'static) {
        #[cfg(all(feature = "hot_reload", not(target_arch = "wasm32")))]
        let update = {
            dioxus_devtools::connect_subsecond();

            move |ctx: &mut FrameContext| {
                dioxus_devtools::subsecond::call(|| update(ctx));
            }
        };
        self.update = Some(Box::new(update));

        let config = self.config.take().unwrap();
        AppRunner::new(self, config).run();
    }
}

impl AppHandler<Renderer> for App {
    fn on_window_event(&mut self, _window: &Window, event: &WindowEvent) {
        #[cfg(feature = "ui")]
        if let Some(egui) = self.egui.as_mut() {
            egui.handle_event(_window, event);
        }

        self.events.push(event.clone());
    }

    async fn with_resource(&mut self, window: Arc<Window>) -> Renderer {
        // WebGPU throws error 'size is zero' if not set
        let size = window.inner_size();
        let (w, h) = (
            if size.width == 0 { 800 } else { size.width },
            if size.height == 0 { 600 } else { size.height },
        );
        Renderer::new(w, h, window).await
    }

    fn on_ready(&mut self, window: &Window, renderer: &mut Renderer) {
        renderer.set_vsync(self.vsync);

        let (device, format) = (renderer.device(), renderer.surface_format());
        self.text_renderer = Some(TextRenderer::new(device, renderer.queue(), format));
        #[cfg(feature = "ui")]
        {
            self.egui = Some(EguiRenderer::new(device, format, window));
        }

        self.resize(
            window.inner_size().width,
            window.inner_size().height,
            renderer,
        );
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
        let Some(mut frame) = renderer.begin_frame() else {
            return;
        };

        let (width, height) = (
            renderer.surface_config().width,
            renderer.surface_config().height,
        );
        let (device, queue) = (renderer.device().clone(), renderer.queue().clone());

        let text_renderer = self.text_renderer.as_mut().unwrap();

        #[cfg(feature = "ui")]
        let egui_ctx = self.egui.as_mut().unwrap().begin_frame(_window);
        let mut ctx = FrameContext {
            gfx: Graphics::new(renderer, text_renderer),
            input,
            timer,
            #[cfg(feature = "ui")]
            egui_ctx,
            events: std::mem::take(&mut self.events),
        };
        update(&mut ctx);

        let mut geometry = ctx.gfx.flush();

        text_renderer.prepare(&device, &queue, width, height);

        {
            let mut r_pass = renderer.begin_render_pass(&mut frame.encoder, &frame.view);

            for (tex_id, batch) in &mut geometry {
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
        renderer.resize(width, height);
        self.text_renderer
            .as_mut()
            .unwrap()
            .resize(width, height, renderer.queue());
    }
}
