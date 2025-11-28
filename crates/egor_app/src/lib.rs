pub mod input;
pub mod time;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

use egor_render::{Graphics, GraphicsInternal, renderer::Renderer};

use crate::{
    input::{Input, InputInternal},
    time::{FrameTimer, FrameTimerInternal},
};

#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;

#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;

pub trait InitFn: FnOnce(&mut InitContext) + 'static {}
impl<F: FnOnce(&mut InitContext) + 'static> InitFn for F {}

pub trait UpdateFn: FnMut(&FrameTimer, &mut Graphics, &mut Input) + 'static {}
impl<F: FnMut(&FrameTimer, &mut Graphics, &mut Input) + 'static> UpdateFn for F {}

type OnQuit = Box<dyn FnMut()>;

/// Entry point for `egor` apps
///
/// Manages windowing, input, rendering, event loop, & plugin system
///
/// Use `App::init(...)` to construct it, then call `.run(...)` to start the loop
/// Add optional plugins or shutdown logic via `.plugin(...)` & `.on_quit(...)`
pub struct App<I, U> {
    init: Option<I>,
    update: Option<U>,
    on_quit: Option<OnQuit>,
    window: Option<Rc<Window>>,
    proxy: Option<EventLoopProxy<Renderer>>,
    renderer: Option<Renderer>,
    input: Input,
    timer: FrameTimer,
}

#[doc(hidden)]
impl<I: InitFn, U: UpdateFn> ApplicationHandler<Renderer> for App<I, U> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Called when window is ready; initializes the renderer async (wasm) or sync (native)
        if let Some(proxy) = self.proxy.take() {
            let win_attrs = {
                #[cfg(target_arch = "wasm32")]
                {
                    use winit::platform::web::WindowAttributesExtWebSys;
                    Window::default_attributes().with_append(true)
                }
                #[cfg(not(target_arch = "wasm32"))]
                Window::default_attributes()
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
                self.timer.update();
                if let Some(r) = self.renderer.as_mut() {
                    let mut graphics = Graphics::new(r);
                    let geometry = graphics.flush();
                    r.render_frame(geometry);
                }

                self.input.end_frame();
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
        // Called once when the renderer finishes initializing
        if let Some(init) = self.init.take() {
            //let Self { window, .. } = self;
            let mut ctx = InitContext {
                window: self.window.as_ref().unwrap().clone(),
                render: &mut renderer,
            };

            init(&mut ctx);
            self.renderer = Some(renderer);
        }
    }
}

impl<I: InitFn, U: UpdateFn> App<I, U> {
    /// Creates a new `App` with defined state & init logic
    pub fn init(init: I) -> Self {
        Self {
            init: Some(init),
            update: None,
            on_quit: None,
            window: None,
            proxy: None,
            renderer: None,
            input: Input::default(),
            timer: FrameTimer::default(),
        }
    }

    /// Starts the app & runs the event loop
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

    /// Runs the provided closure before quitting
    pub fn on_quit<F: FnMut() + 'static>(&mut self, f: F) {
        self.on_quit = Some(Box::new(f));
    }
}

/// Passed into `init()`
pub struct InitContext<'a> {
    window: Rc<Window>,
    render: &'a mut Renderer,
}

impl InitContext<'_> {
    /// Sets the window title
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Enables/disables V-Sync
    pub fn set_vsync(&mut self, on: bool) {
        self.render.set_vsync(on);
    }

    /// Loads a texture from raw image data (e.g., PNG)
    pub fn load_texture(&mut self, data: &[u8]) -> usize {
        self.render.add_texture(data)
    }
}

/// Frame-local context passed into `update()` & plugins
pub struct Context<'a, 'b> {
    pub graphics: &'a mut Graphics<'b>,
    pub input: &'a mut Input,
    pub timer: &'a FrameTimer,
}
