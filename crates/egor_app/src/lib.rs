pub mod input;
pub mod time;

use crate::{input::Input, time::FrameTimer};
use std::sync::Arc;
pub use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ControlFlow,
    window::{Fullscreen, Window},
};

#[cfg(target_os = "android")]
use std::sync::OnceLock;
#[cfg(target_os = "android")]
pub use winit::platform::android::activity::AndroidApp;
#[cfg(target_os = "android")]
pub static ANDROID_APP: OnceLock<AndroidApp> = OnceLock::new();

use winit::{
    application::ApplicationHandler,
    event::MouseScrollDelta,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::WindowId,
};

pub struct AppConfig {
    pub control_flow: ControlFlow,
    pub title: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: bool,
    pub maximized: bool,
    pub fullscreen: bool,
    pub decorations: bool,
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            control_flow: ControlFlow::Poll,
            title: "Egor App".to_string(),
            width: None,
            height: None,
            resizable: true,
            maximized: false,
            fullscreen: false,
            decorations: true,
            min_size: None,
            max_size: None,
        }
    }
}

/// Trait defining application behavior
///
/// Implement this for your app logic. Hooks are called during window creation,
/// every frame, on resize, & before quitting
#[allow(async_fn_in_trait)]
pub trait AppHandler<R> {
    /// Called when app is resumed
    fn resumed(&mut self, _window: Arc<Window>, _resource: &mut R) {}
    /// Called when app is suspended (happens for Android in background)
    fn suspended(&mut self) {}
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
}

#[doc(hidden)]
impl<R, H: AppHandler<R> + 'static> ApplicationHandler<(R, H)> for AppRunner<R, H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let (Some(window), Some(resource), Some(handler)) = (
            self.window.clone(),
            self.resource.as_mut(),
            self.handler.as_mut(),
        ) {
            handler.resumed(window, resource);
        }

        // Called when window is ready; initializes the resource async (wasm) or sync (native)
        let Some(proxy) = self.proxy.take() else {
            return;
        };

        let fullscreen = match self.config.fullscreen {
            true => Some(Fullscreen::Borderless(None)),
            false => None,
        };

        let mut win_attrs = Window::default_attributes()
            .with_visible(false)
            .with_title(&self.config.title)
            .with_resizable(self.config.resizable)
            .with_maximized(self.config.maximized)
            .with_fullscreen(fullscreen)
            .with_decorations(self.config.decorations);

        if let (Some(w), Some(h)) = (self.config.width, self.config.height) {
            win_attrs = win_attrs.with_inner_size(PhysicalSize::new(w, h));
        }
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowAttributesExtWebSys;
            win_attrs = win_attrs.with_append(true);
        }

        let window = Arc::new(event_loop.create_window(win_attrs).unwrap());
        self.window = Some(window.clone());

        if let Some((w, h)) = self.config.min_size {
            window.set_min_inner_size(Some(PhysicalSize::new(w, h)));
        }
        if let Some((w, h)) = self.config.max_size {
            window.set_max_inner_size(Some(PhysicalSize::new(w, h)));
        }

        let mut handler = self.handler.take().unwrap();
        #[cfg(target_arch = "wasm32")]
        {
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

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(handler) = self.handler.as_mut() {
            handler.suspended();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Some(handler) = &mut self.handler {
            handler.on_window_event(self.window.as_ref().unwrap(), &event);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let Some(window) = &self.window else { return };
                let (Some(resource), Some(handler)) = (&mut self.resource, &mut self.handler)
                else {
                    return;
                };

                self.timer.update();
                handler.frame(window, resource, &self.input, &self.timer);
                self.input.end_frame();

                if self.config.control_flow == ControlFlow::Poll {
                    window.request_redraw();
                }
            }
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return;
                }

                if let (Some(resource), Some(handler)) =
                    (self.resource.as_mut(), self.handler.as_mut())
                {
                    handler.resize(size.width, size.height, resource);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => self.input.update_key(event),
            WindowEvent::MouseInput { button, state, .. } => {
                self.input.update_mouse_button(button, state)
            }
            WindowEvent::CursorMoved { position, .. } => self.input.update_cursor(position),
            WindowEvent::MouseWheel { delta, .. } => {
                let wheel_delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                self.input.update_scroll(wheel_delta);
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, (mut resource, mut handler): (R, H)) {
        let Some(window) = &self.window else { return };

        handler.on_ready(window, &mut resource);
        handler.frame(window, &mut resource, &self.input, &self.timer);

        window.set_visible(true);
        window.request_redraw();

        self.resource = Some(resource);
        self.handler = Some(handler);
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
        }
    }

    /// Runs the app’s event loop on the current platform
    ///
    /// Handles Android, WASM and native setups, plus logging and user events
    pub fn run(mut self) {
        let mut event_loop_builder = EventLoop::<(R, H)>::with_user_event();
        #[cfg(target_os = "android")]
        {
            #[cfg(feature = "log")]
            android_logger::init_once(Default::default().with_max_level(log::LevelFilter::Info));

            use winit::platform::android::EventLoopBuilderExtAndroid;
            let android_app = ANDROID_APP.get().unwrap().clone();
            event_loop_builder.with_android_app(android_app);
        }

        let event_loop = event_loop_builder.build().unwrap();
        event_loop.set_control_flow(self.config.control_flow);
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
            #[cfg(all(feature = "log", not(target_os = "android")))]
            env_logger::init_from_env(env_logger::Env::default().default_filter_or("error"));

            event_loop.run_app(&mut self).unwrap();
        }
    }
}
