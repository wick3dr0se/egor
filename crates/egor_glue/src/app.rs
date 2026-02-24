use std::sync::Arc;

use crate::{graphics::Graphics, primitives::PrimitiveBatch, text::TextRenderer};

#[cfg(feature = "ui")]
use crate::ui::EguiRenderer;

use egor_app::{
    AppConfig, AppHandler, AppRunner, ControlFlow, Fullscreen, PhysicalSize, Window, WindowEvent,
    input::Input, time::FrameTimer,
};
use egor_render::{Backbuffer, RenderTarget, Renderer};

type UpdateFn = dyn FnMut(&mut FrameContext);

pub struct AppControl<'a> {
    window: &'a Window,
    requested_size: Option<(u32, u32)>,
    requested_vsync: Option<bool>,
}

impl<'a> AppControl<'a> {
    /// Request the window to redraw its contents on the next frame
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// Set the inner size of the window in physical pixels
    /// Returns the new size depending on platform
    pub fn set_size(&mut self, w: u32, h: u32) {
        let _ = self.window.request_inner_size(PhysicalSize::new(w, h));
        self.requested_size = Some((w, h));
    }

    /// Enable or disable borderless fullscreen mode
    pub fn set_fullscreen(&self, enabled: bool) {
        self.window
            .set_fullscreen(enabled.then(|| Fullscreen::Borderless(None)));
    }

    /// Enable or disable vertical sync
    /// When enabled, frame presentation is synchronized to the display's refresh
    /// rate, preventing screen tearing
    pub fn set_vsync(&mut self, on: bool) {
        self.requested_vsync = Some(on);
    }
}

pub struct FrameContext<'a> {
    pub events: Vec<WindowEvent>,
    pub app: AppControl<'a>,
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
    backbuffer: Option<Backbuffer>,
    primitive_batch: PrimitiveBatch,
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
            backbuffer: None,
            primitive_batch: PrimitiveBatch::default(),
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

    /// Enable or disable window maximized (defaults to false)
    pub fn maximized(mut self, maximized: bool) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.maximized = maximized;
        }
        self
    }

    /// Enable or disable fullscreen (defaults to false)
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.fullscreen = fullscreen;
        }
        self
    }

    /// Enable or disable window decorations (defaults to true)
    pub fn decorations(mut self, decorations: bool) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.decorations = decorations;
        }
        self
    }

    /// Enable or disable vsync
    pub fn vsync(mut self, enabled: bool) -> Self {
        self.vsync = enabled;
        self
    }

    /// Set the event loop control flow (defaults to [`ControlFlow::Poll`])
    ///
    /// - `ControlFlow::Poll`: continuously redraws (game-style loop)
    /// - `ControlFlow::Wait`: no frames are produced unless
    ///   [`AppControl::request_redraw()`] is called
    ///
    /// When using `Wait`, you are responsible for requesting redraws
    pub fn control_flow(mut self, control_flow: ControlFlow) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.control_flow = control_flow;
        }
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
        let renderer = Renderer::new(window.clone()).await;
        self.backbuffer = Some(Backbuffer::new(
            renderer.instance(),
            renderer.adapter(),
            renderer.device(),
            window,
            w,
            h,
        ));
        renderer
    }

    fn on_ready(&mut self, window: &Window, renderer: &mut Renderer) {
        let (device, format) = (
            renderer.device(),
            self.backbuffer.as_ref().unwrap().format(),
        );
        self.backbuffer
            .as_mut()
            .unwrap()
            .set_vsync(device, self.vsync);
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
        let Some(backbuffer) = &mut self.backbuffer else {
            return;
        };
        let Some(mut frame) = renderer.begin_frame(backbuffer) else {
            return;
        };

        let (w, h) = backbuffer.size();
        let (device, queue) = (renderer.device().clone(), renderer.queue().clone());
        let format = backbuffer.format();
        let text_renderer = self.text_renderer.as_mut().unwrap();

        #[cfg(feature = "ui")]
        let egui_ctx = self.egui.as_mut().unwrap().begin_frame(_window);
        let mut ctx = FrameContext {
            events: std::mem::take(&mut self.events),
            app: AppControl {
                window: _window,
                requested_size: None,
                requested_vsync: None,
            },
            gfx: Graphics::new(
                renderer,
                &mut self.primitive_batch,
                text_renderer,
                format,
                w,
                h,
            ),
            input,
            timer,
            #[cfg(feature = "ui")]
            egui_ctx,
        };
        update(&mut ctx);

        let requested_size = ctx.app.requested_size;
        let requested_vsync = ctx.app.requested_vsync;
        if let Some((pw, ph)) = requested_size {
            ctx.gfx.set_target_size(pw, ph);
        }

        ctx.gfx.upload_camera();

        text_renderer.prepare(&device, &queue, w, h);

        {
            let mut r_pass = renderer.begin_render_pass(&mut frame.encoder, &frame.view);

            for (tex_id, shader_id, batch) in self.primitive_batch.iter_mut() {
                renderer.draw_batch(&mut r_pass, batch, tex_id, shader_id);
            }

            text_renderer.render(&mut r_pass);
        }

        self.primitive_batch.reset();

        #[cfg(feature = "ui")]
        {
            let render_data = self.egui.as_mut().unwrap().end_frame(_window);
            self.egui.as_mut().unwrap().render(
                &device,
                &queue,
                &mut frame.encoder,
                &frame.view,
                w,
                h,
                render_data,
            );
        }

        renderer.end_frame(frame);

        if let Some((rw, rh)) = requested_size {
            self.backbuffer.as_mut().unwrap().resize(&device, rw, rh);
        }
        if let Some(vsync) = requested_vsync {
            self.backbuffer.as_mut().unwrap().set_vsync(&device, vsync);
            self.vsync = vsync;
        }
    }

    fn resize(&mut self, w: u32, h: u32, renderer: &mut Renderer) {
        self.backbuffer
            .as_mut()
            .unwrap()
            .resize(renderer.device(), w, h);
        self.text_renderer
            .as_mut()
            .unwrap()
            .resize(w, h, renderer.queue());
    }

    fn suspended(&mut self) {
        self.backbuffer = None;
    }

    fn resumed(&mut self, window: Arc<Window>, renderer: &mut Renderer) {
        let size = window.inner_size();
        let device = renderer.device();
        let mut backbuffer = Backbuffer::new(
            renderer.instance(),
            renderer.adapter(),
            device,
            window,
            size.width,
            size.height,
        );
        backbuffer.set_vsync(device, self.vsync);
        self.backbuffer = Some(backbuffer);
    }
}
