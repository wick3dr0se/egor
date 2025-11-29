pub mod input;
pub mod time;

pub use egor_render::Graphics;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

use egor_render::{GraphicsInternal, renderer::Renderer};

use crate::{
    input::{Input, InputInternal},
    time::{FrameTimer, FrameTimerInternal},
};

#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;

#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;

pub trait UpdateFn: FnMut(&mut Graphics, &Input, &FrameTimer) + 'static {}
impl<F: FnMut(&mut Graphics, &Input, &FrameTimer) + 'static> UpdateFn for F {}

type OnQuit = Box<dyn FnMut()>;

/// Entry point for `egor` apps
///
/// Manages windowing, input, rendering, & event loop
///
/// Use `App::new()` to construct it, then call `.run(...)` to start the loop
pub struct App<U> {
    update: Option<U>,
    on_quit: Option<OnQuit>,
    window: Option<Rc<Window>>,
    proxy: Option<EventLoopProxy<Renderer>>,
    renderer: Option<Renderer>,
    input: Input,
    timer: FrameTimer,
    title: String,
    vsync: bool,
}

#[doc(hidden)]
impl<U: UpdateFn> ApplicationHandler<Renderer> for App<U> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Called when window is ready; initializes the renderer async (wasm) or sync (native)
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
            let (width, height) = (window.inner_size().width, window.inner_size().height);

            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(async move {
                let renderer = Renderer::create_graphics(width, height, window).await;
                _ = proxy.send_event(renderer);
            });
            #[cfg(not(target_arch = "wasm32"))]
            {
                let renderer = pollster::block_on(Renderer::create_graphics(width, height, window));
                _ = proxy.send_event(renderer);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                if let Some(on_quit) = self.on_quit.as_mut() {
                    on_quit();
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(r) = self.renderer.as_mut() {
                    if let Some(update) = self.update.as_mut() {
                        let mut graphics = Graphics::new(r);
                        update(&mut graphics, &self.input, &self.timer);
                        let geometry = graphics.flush();
                        r.render_frame(geometry);
                    }

                    self.timer.update();
                    self.input.end_frame();
                }

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(size) => {
                if let Some(r) = self.renderer.as_mut() {
                    r.resize(size.width, size.height)
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input.keyboard(event);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                self.input.mouse(button, state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input.cursor(position);
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, mut renderer: Renderer) {
        // Renderer initialized, apply config
        renderer.set_vsync(self.vsync);
        self.renderer = Some(renderer);
    }
}

impl<U: UpdateFn> App<U> {
    /// Creates a new `App`
    pub fn new() -> Self {
        Self {
            update: None,
            on_quit: None,
            window: None,
            proxy: None,
            renderer: None,
            input: Input::default(),
            timer: FrameTimer::default(),
            title: "egor app".to_string(),
            vsync: true,
        }
    }

    /// Sets the window title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Enables/disables V-Sync (default: true)
    pub fn vsync(mut self, on: bool) -> Self {
        self.vsync = on;
        self
    }

    /// Runs the provided closure before quitting
    pub fn on_quit(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_quit = Some(Box::new(f));
        self
    }

    /// Starts the app & runs the event loop
    ///
    /// The closure receives graphics, input, and timer every frame.
    /// Use `timer.frame` to detect the first frame for initialization:
    ///
    /// ```rust
    /// let mut state = GameState::new();
    ///
    /// App::new()
    ///     .title("My Game")
    ///     .vsync(false)
    ///     .run(|graphics, input, timer| {
    ///         // First frame - initialization
    ///         if timer.frame == 0 {
    ///             state.texture = graphics.load_texture(data);
    ///         }
    ///         
    ///         // Every frame - update & render
    ///         if input.key_pressed(KeyCode::Space) {
    ///             state.player.jump();
    ///         }
    ///         
    ///         graphics.clear(Color::BLACK);
    ///         graphics.rect().at(state.player.pos);
    ///     });
    /// ```
    pub fn run(mut self, update: U) {
        let event_loop = EventLoop::<Renderer>::with_user_event().build().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        self.proxy = Some(event_loop.create_proxy());
        self.update = Some(update);

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
