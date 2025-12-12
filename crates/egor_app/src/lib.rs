pub mod input;
pub mod time;

use std::ops::Deref;

pub use winit::window::Window;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::WindowId,
};

use crate::{
    input::{Input, InputInternal},
    time::{FrameTimer, FrameTimerInternal},
};

#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;
#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;
pub struct WindowHandle(Rc<Window>);

impl WindowHandle {
    /// Get inner Rc/Arc
    pub fn inner(self) -> Rc<Window> {
        self.0
    }
}

// Make it act like a &Window automatically
impl Deref for WindowHandle {
    type Target = Window;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Trait defining application behavior
///
/// Implement this for your app logic. Hooks are called during window creation,
/// every frame, on resize, and before quitting
#[allow(async_fn_in_trait)]
pub trait AppHandler<R> {
    /// Called once the window exists; should create & return the resource
    async fn with_resource(&mut self, window: WindowHandle) -> R;
    /// Called after the resource is initialized and window is ready
    fn on_ready(&mut self, _window: &Window, _state: &mut R) {}
    /// Called every frame
    fn frame(&mut self, _state: &mut R, _input: &Input, _timer: &FrameTimer) {}
    /// Called on window resize
    fn resize(&mut self, _w: u32, _h: u32, _state: &mut R) {}
    fn on_quit(&mut self) {}
}

/// Generic application entry point
///
/// Manages window creation, input, event loop, and delegating hooks
/// to your `AppHandler`
/// Use `AppRunner::new()` to construct it, then call `.run(...)` to start the loop
pub struct AppRunner<R: 'static, H: AppHandler<R> + 'static> {
    handler: Option<H>,
    resource: Option<R>,
    window: Option<Rc<Window>>,
    proxy: Option<EventLoopProxy<(R, H)>>,
    input: Input,
    timer: FrameTimer,
    title: String,
}

#[doc(hidden)]
impl<R, H: AppHandler<R> + 'static> ApplicationHandler<(R, H)> for AppRunner<R, H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Called when window is ready; initializes the resource async (wasm) or sync (native)
        if let Some(proxy) = self.proxy.take() {
            let win_attrs = {
                #[cfg(target_arch = "wasm32")]
                {
                    use winit::platform::web::WindowAttributesExtWebSys;
                    Window::default_attributes()
                        .with_title(&self.title)
                        .with_append(true)
                }
                #[cfg(not(target_arch = "wasm32"))]
                Window::default_attributes().with_title(&self.title)
            };
            let window = Rc::new(event_loop.create_window(win_attrs).unwrap());
            self.window = Some(window.clone());
            let mut handler = self.handler.take().unwrap();

            #[cfg(target_arch = "wasm32")]
            {
                wasm_bindgen_futures::spawn_local(async move {
                    let resource = handler.with_resource(WindowHandle(window)).await;
                    _ = proxy.send_event((resource, handler));
                });
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let resource = pollster::block_on(handler.with_resource(WindowHandle(window)));
                _ = proxy.send_event((resource, handler));
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                if let Some(handler) = self.handler.as_mut() {
                    handler.on_quit();
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(r), Some(handler)) = (self.resource.as_mut(), self.handler.as_mut()) {
                    handler.frame(r, &self.input, &self.timer);
                    self.timer.update();
                    self.input.end_frame();
                }
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
            WindowEvent::Resized(size) => {
                if let (Some(r), Some(handler)) = (self.resource.as_mut(), self.handler.as_mut()) {
                    handler.resize(size.width, size.height, r);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => self.input.keyboard(event),
            WindowEvent::MouseInput { button, state, .. } => self.input.mouse(button, state),
            WindowEvent::CursorMoved { position, .. } => self.input.cursor(position),
            _ => {}
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, (resource, handler): (R, H)) {
        self.resource = Some(resource);
        self.handler = Some(handler);

        if let (Some(r), Some(h), Some(window)) =
            (&mut self.resource, &mut self.handler, &self.window)
        {
            h.on_ready(window, r);
        }
    }
}

impl<R, H: AppHandler<R> + 'static> AppRunner<R, H> {
    /// Creates a new `AppRunner` with the given handler
    pub fn new(handler: H) -> Self {
        Self {
            handler: Some(handler),
            resource: None,
            window: None,
            proxy: None,
            input: Input::default(),
            timer: FrameTimer::default(),
            title: "egor app".to_string(),
        }
    }

    /// Sets the window title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Starts the app and runs the event loop
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
