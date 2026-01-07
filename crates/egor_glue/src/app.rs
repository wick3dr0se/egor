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

/// Application builder for creating and configuring a windowed application.
///
/// This struct provides a fluent builder API for configuring window properties
/// and running the application's main loop.
///
/// # Example
///
/// ```
/// use egor::{app::App, render::Graphics};
///
/// App::new()
///     .title("My Game")
///     .screen_size(800, 600)
///     .vsync(true)
///     .run(|gfx: &mut Graphics, input, timer| {
///         // Game loop
///     });
/// ```
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
    /// Create a new [`App`] instance with default configuration.
    ///
    /// # Returns
    ///
    /// A new `App` builder with default settings:
    /// - VSync enabled
    /// - No window size set (will use default)
    /// - Window is resizable by default
    ///
    /// # Example
    ///
    /// ```
    /// use egor::app::App;
    /// let app = App::new();
    /// ```
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

    /// Set the application window title.
    ///
    /// # Arguments
    ///
    /// * `title` - The text to display in the window title bar
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// App::new().title("My Game");
    /// ```
    pub fn title(mut self, title: &str) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.title = title.into();
        }
        self
    }

    /// Set the window size in pixels.
    ///
    /// # Arguments
    ///
    /// * `width` - Window width in pixels
    /// * `height` - Window height in pixels
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining.
    ///
    /// # Note
    ///
    /// By default, windows spawn at a platform-dependent position (typically the top-left corner
    /// or the last known position). If you want the window to be centered on the primary monitor,
    /// use [`screen_size_centered`](Self::screen_size_centered) instead.
    ///
    /// # Example
    ///
    /// ```
    /// App::new().screen_size(800, 600);
    /// ```
    pub fn screen_size(mut self, width: u32, height: u32) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.width = Some(width);
            c.height = Some(height);
        }
        self
    }

    /// Set the window size in pixels and center it on the primary monitor.
    ///
    /// # Arguments
    ///
    /// * `width` - Window width in pixels
    /// * `height` - Window height in pixels
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining.
    ///
    /// # Note
    ///
    /// This method sets the window size and automatically centers it on the primary monitor.
    /// The centering calculation accounts for the monitor's position offset (important for
    /// multi-monitor setups) and uses floating-point precision to ensure accurate positioning.
    ///
    /// Windows spawn at platform-dependent positions by default, which can be inconsistent
    /// across different systems or window managers. This method provides a consistent,
    /// user-friendly default by centering the window on the primary display.
    ///
    /// # Example
    ///
    /// ```
    /// App::new().screen_size_centered(800, 600);
    /// ```
    pub fn screen_size_centered(mut self, width: u32, height: u32) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.width = Some(width);
            c.height = Some(height);
        }

        // Set up centering calculation
        if let Some(c) = self.config.as_mut()
            && let (Some(window_width), Some(window_height)) = (c.width, c.height)
        {
            c.position = Some(Box::new(move |monitor_width: u32, monitor_height: u32| {
                // Center calculation using floating point for precision, then round
                // Formula: (monitor_size - window_size) / 2
                // This calculates the position relative to the monitor, which will be
                // adjusted by the monitor's position offset in the window creation code
                let x = ((monitor_width as f32 - window_width as f32) / 2.0).round() as i32;
                let y = ((monitor_height as f32 - window_height as f32) / 2.0).round() as i32;
                (x, y)
            }));
        }

        self
    }

    /// Enable or disable window resizing by the user.
    ///
    /// # Arguments
    ///
    /// * `resizable` - `true` to allow the user to resize the window, `false` to disable resizing
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// // Create a fixed-size window
    /// App::new().screen_size(640, 480).resizable(false);
    /// ```
    pub fn resizable(mut self, resizable: bool) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.resizable = Some(resizable);
        }
        self
    }

    /// Set fullscreen mode for the window.
    ///
    /// # Arguments
    ///
    /// * `fullscreen` - `true` to enable fullscreen mode, `false` for windowed mode
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining.
    ///
    /// # Note
    ///
    /// When `true`, uses exclusive fullscreen mode which changes the display resolution.
    /// The window will take over the entire screen.
    ///
    /// # Example
    ///
    /// ```
    /// App::new().fullscreen(true);
    /// ```
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.fullscreen = Some(fullscreen);
        }
        self
    }

    /// Set whether the window should start maximized.
    ///
    /// # Arguments
    ///
    /// * `maximized` - `true` to start the window maximized, `false` for normal size
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// App::new().maximized(true);
    /// ```
    pub fn maximized(mut self, maximized: bool) -> Self {
        if let Some(c) = self.config.as_mut() {
            c.maximized = Some(maximized);
        }
        self
    }

    /// Set the window position using a closure that calculates the position.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that receives the monitor width and height, and returns the window position `(x, y)`
    ///   - `screen_w` - Monitor width in pixels
    ///   - `screen_h` - Monitor height in pixels
    ///   - Returns: `(x, y)` - Window position in pixels (can be negative for multi-monitor setups)
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining.
    ///
    /// # Note
    ///
    /// The position closure only receives monitor dimensions, not window dimensions. If you need
    /// to calculate positions relative to the window size (e.g., for centering), you'll need to
    /// know the window dimensions from calling [`screen_size`](Self::screen_size) or
    /// [`screen_size_centered`](Self::screen_size_centered) first, and hardcode or capture those
    /// values in your closure.
    ///
    /// # Example
    ///
    /// ```
    /// // Center the window on screen (requires knowing window size)
    /// App::new()
    ///     .screen_size(640, 480)
    ///     .position(|screen_w, screen_h| {
    ///         let x = (screen_w.saturating_sub(640) / 2) as i32;
    ///         let y = (screen_h.saturating_sub(480) / 2) as i32;
    ///         (x, y)
    ///     });
    ///
    /// // Or use absolute positioning (doesn't require window size)
    /// App::new()
    ///     .position(|_screen_w, _screen_h| (100, 100));
    /// ```
    pub fn position<F>(mut self, f: F) -> Self
    where
        F: FnOnce(u32, u32) -> (i32, i32) + 'static,
    {
        if let Some(c) = self.config.as_mut() {
            c.position = Some(Box::new(f));
        }
        self
    }

    /// Enable or disable VSync (vertical synchronization).
    ///
    /// # Arguments
    ///
    /// * `enabled` - `true` to enable VSync (caps framerate to monitor refresh rate), `false` to disable
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining.
    ///
    /// # Note
    ///
    /// VSync helps prevent screen tearing but may introduce input lag.
    /// Disabled by default for lower latency, but can be enabled for smoother visuals.
    ///
    /// # Example
    ///
    /// ```
    /// App::new().vsync(true);
    /// ```
    pub fn vsync(mut self, enabled: bool) -> Self {
        self.vsync = enabled;
        self
    }

    /// Run the application with a per-frame update closure.
    ///
    /// # Arguments
    ///
    /// * `update` - A closure that is called every frame with:
    ///   - `gfx` - Mutable reference to [`Graphics`] for drawing
    ///   - `input` - Reference to [`Input`] for keyboard/mouse state
    ///   - `timer` - Reference to [`FrameTimer`] for timing information
    ///
    /// # Note
    ///
    /// This method consumes the `App` and starts the event loop.
    /// The closure will be called repeatedly until the window is closed.
    ///
    /// # Example
    ///
    /// ```
    /// use egor::{app::App, render::Graphics};
    /// App::new().run(|gfx: &mut Graphics, input, timer| {
    ///     // Your game loop code here
    /// });
    /// ```
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
    /// Run the application with a per-frame update closure (with egui support).
    ///
    /// # Arguments
    ///
    /// * `update` - A closure that is called every frame with:
    ///   - `gfx` - Mutable reference to [`Graphics`] for drawing
    ///   - `input` - Reference to [`Input`] for keyboard/mouse state
    ///   - `timer` - Reference to [`FrameTimer`] for timing information
    ///   - `ui` - Reference to [`egui::Context`] for UI rendering
    ///
    /// # Note
    ///
    /// This variant is only available when the `ui` feature is enabled.
    /// This method consumes the `App` and starts the event loop.
    ///
    /// # Example
    ///
    /// ```
    /// use egor::{app::App, render::Graphics};
    /// App::new().run(|gfx, input, timer, ui| {
    ///     // Your game loop with UI code here
    /// });
    /// ```
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

    /// Set a closure to be called when the application is quitting.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that will be called when the window is closed or the app is requested to quit
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// App::new().on_quit(|| {
    ///     println!("Goodbye!");
    /// });
    /// ```
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

        self.text_renderer = Some(TextRenderer::new(
            renderer.device(),
            renderer.queue(),
            renderer.surface_format(),
        ));

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
