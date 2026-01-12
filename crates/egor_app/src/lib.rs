mod coordinate_converter;
pub mod input;
pub mod time;

#[allow(unused)]
use crate::coordinate_converter::{CoordinateConverter, create_desktop_converter};
use crate::{input::Input, time::FrameTimer};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::WindowId,
};
pub use winit::{event::WindowEvent, window::Window};

pub struct AppConfig {
    pub title: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "Egor App".to_string(),
            width: None,
            height: None,
            resizable: true,
        }
    }
}

/// Trait defining application behavior
///
/// Implement this for your app logic. Hooks are called during window creation,
/// every frame, on resize, & before quitting
#[allow(async_fn_in_trait)]
pub trait AppHandler<R> {
    /// Called for every WindowEvent before default input handling
    fn on_window_event(&mut self, _window: &Window, _event: &WindowEvent) {}
    /// Called once the window exists; should create & return the resource
    async fn with_resource(&mut self, _window: Arc<Window>) -> R;
    /// Called after the resource is initialized & window is ready
    fn on_ready(&mut self, _window: &Window, _resource: &mut R) {}
    /// Called every frame
    fn frame(&mut self, _window: &Window, _resource: &mut R, _input: &Input, _timer: &FrameTimer) {}
    /// Called on window resize
    fn resize(&mut self, _w: u32, _h: u32, _resource: &mut R) {}
    /// Called when the window is requested to close
    fn on_quit(&mut self) {}
}

/// Generic application entry point
///
/// Manages window creation, input, event loop, & delegating hooks
/// to your `AppHandler`
/// Use `AppRunner::new()` to construct it, then call `.run(...)` to start the loop
pub struct AppRunner<R: 'static, H: AppHandler<R> + 'static> {
    handler: Option<H>,
    resource: Option<R>,
    window: Option<Arc<Window>>,
    proxy: Option<EventLoopProxy<(R, H)>>,
    input: Input,
    timer: FrameTimer,
    config: AppConfig,
    coordinate_converter: Option<CoordinateConverter>,
}

#[doc(hidden)]
impl<R, H: AppHandler<R> + 'static> ApplicationHandler<(R, H)> for AppRunner<R, H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Called when window is ready; initializes the resource async (wasm) or sync (native)
        if let Some(proxy) = self.proxy.take() {
            let win_attrs = {
                use winit::dpi::PhysicalSize;

                #[allow(unused_mut)]
                let mut attrs = Window::default_attributes()
                    .with_title(&self.config.title)
                    .with_resizable(self.config.resizable);

                if let (Some(width), Some(height)) = (self.config.width, self.config.height) {
                    attrs = attrs.with_inner_size(PhysicalSize::new(width, height));
                }

                #[cfg(target_arch = "wasm32")]
                {
                    use winit::platform::web::WindowAttributesExtWebSys;
                    attrs = attrs.with_append(true);
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    attrs = attrs.with_visible(false);
                }

                attrs
            };
            let window = Arc::new(event_loop.create_window(win_attrs).unwrap());
            self.window = Some(window.clone());
            let mut handler = self.handler.take().unwrap();

            #[cfg(target_arch = "wasm32")]
            {
                // Wait for DOM and canvas to be ready before initializing
                wasm_bindgen_futures::spawn_local(async move {
                    let resource = handler.with_resource(window).await;
                    _ = proxy.send_event((resource, handler));
                });
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let resource = pollster::block_on(handler.with_resource(window));
                _ = proxy.send_event((resource, handler));
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Some(handler) = &mut self.handler {
            handler.on_window_event(self.window.as_ref().unwrap(), &event);
        }

        match event {
            WindowEvent::CloseRequested => {
                if let Some(handler) = &mut self.handler {
                    handler.on_quit();
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(w), Some(r), Some(handler)) = (
                    self.window.as_ref(),
                    self.resource.as_mut(),
                    self.handler.as_mut(),
                ) {
                    handler.frame(w, r, &self.input, &self.timer);
                    self.timer.update();
                    self.input.end_frame();
                }
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
            WindowEvent::Resized(size) => {
                // Recalculate coordinate converter on resize (for WASM and desktop touch support)
                #[cfg(target_arch = "wasm32")]
                {
                    if let Some(web_window) = web_sys::window() {
                        self.coordinate_converter =
                            Some(wasm_helpers::create_coordinate_converter(&web_window));
                    }
                }

                // no way to test this so I'm guessing it works.
                // worse case scenario touch is off and we get a bug
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(window) = self.window.as_ref() {
                        let physical_size = (size.width as f32, size.height as f32);
                        let scale_factor = window.scale_factor() as f32;
                        self.coordinate_converter =
                            Some(create_desktop_converter(physical_size, scale_factor));
                    } else {
                        self.coordinate_converter = Some(CoordinateConverter::default());
                    }
                }

                if let (Some(r), Some(handler)) = (self.resource.as_mut(), self.handler.as_mut()) {
                    handler.resize(size.width, size.height, r);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => self.input.keyboard(event),
            WindowEvent::MouseInput { button, state, .. } => self.input.mouse(button, state),
            WindowEvent::CursorMoved { position, .. } => self.input.cursor(position),
            WindowEvent::Touch(touch) => {
                if let Some(converter) = self.coordinate_converter {
                    self.input.touch(touch, converter);
                }
            }
            WindowEvent::ScaleFactorChanged {
                #[allow(unused)]
                scale_factor,
                ..
            } => {
                // Recalculate coordinate converter when DPI changes (e.g., window moved to different monitor)
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(window) = self.window.as_ref() {
                        let size = window.inner_size();
                        let physical_size = (size.width as f32, size.height as f32);
                        let scale_factor_f32 = scale_factor as f32;
                        self.coordinate_converter =
                            Some(create_desktop_converter(physical_size, scale_factor_f32));
                    }
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, (resource, handler): (R, H)) {
        self.resource = Some(resource);
        self.handler = Some(handler);

        if let (Some(r), Some(h), Some(w)) = (&mut self.resource, &mut self.handler, &self.window) {
            h.on_ready(w, r);

            #[cfg(not(target_arch = "wasm32"))]
            {
                h.frame(w, r, &self.input, &self.timer);
                w.set_visible(true);
            }
        }
    }
}

impl<R, H: AppHandler<R> + 'static> AppRunner<R, H> {
    /// Creates a new runner with the given handler & configuration
    pub fn new(handler: H, config: AppConfig) -> Self {
        Self {
            handler: Some(handler),
            resource: None,
            window: None,
            proxy: None,
            input: Input::default(),
            timer: FrameTimer::default(),
            config,
            coordinate_converter: None,
        }
    }

    /// Starts the app & runs the event loop
    pub fn run(mut self) {
        let event_loop = EventLoop::<(R, H)>::with_user_event().build().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        self.proxy = Some(event_loop.create_proxy());

        #[cfg(target_arch = "wasm32")]
        {
            #[cfg(feature = "log")]
            {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Error).unwrap();
            }

            use winit::platform::web::EventLoopExtWebSys;
            wasm_bindgen_futures::spawn_local(async move {
                event_loop.spawn_app(self);
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            #[cfg(feature = "log")]
            env_logger::init_from_env(env_logger::Env::default().default_filter_or("error"));

            event_loop.run_app(&mut self).unwrap();
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_helpers {
    use super::coordinate_converter::{CoordinateConverter, DisplayInfo};
    use wasm_bindgen::JsCast;
    use web_sys::{HtmlCanvasElement, Window};

    /// Extract canvas information from the DOM
    pub fn get_canvas_info(window: &Window) -> Option<DisplayInfo> {
        let document = window.document()?;
        let canvas = document.query_selector("canvas").ok()??;
        let canvas: HtmlCanvasElement = canvas.dyn_into().ok()?;

        // Get logical dimensions (CSS/client dimensions as rendered on screen)
        let logical_width = canvas.client_width() as f32;
        let logical_height = canvas.client_height() as f32;

        // Get buffer dimensions (actual pixel buffer)
        let buffer_width = canvas.width() as f32;
        let buffer_height = canvas.height() as f32;

        Some(DisplayInfo {
            logical_width,
            logical_height,
            buffer_width,
            buffer_height,
        })
    }

    /// Create a CoordinateConverter by inspecting the DOM
    pub fn create_coordinate_converter(window: &Window) -> CoordinateConverter {
        let display_info = get_canvas_info(window);
        let scale_factor = window.device_pixel_ratio() as f32;

        if let Some(info) = display_info {
            CoordinateConverter::new(info, scale_factor)
        } else {
            // Fallback to default if canvas not found
            CoordinateConverter::default()
        }
    }
}
