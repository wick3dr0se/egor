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

pub trait InitFn<S>: FnOnce(&mut S, &mut InitContext) + 'static {}
impl<S, F: FnOnce(&mut S, &mut InitContext) + 'static> InitFn<S> for F {}

type OnQuit<S> = Box<dyn FnMut(&mut S)>;

/// Entry point for `egor` apps
///
/// Manages windowing, input, rendering, event loop, & plugin system
///
/// Use `App::init(...)` to construct it, then call `.run(...)` to start the loop
/// Add optional plugins or shutdown logic via `.plugin(...)` & `.on_quit(...)`
pub struct App<S, I> {
    state: S,
    init: Option<I>,
    on_quit: Option<OnQuit<S>>,
    plugins: Vec<Box<dyn Plugin<S>>>,
    window: Option<Rc<Window>>,
    proxy: Option<EventLoopProxy<Renderer>>,
    renderer: Option<Renderer>,
    input: Input,
    timer: FrameTimer,
}

#[doc(hidden)]
impl<S, I: InitFn<S>> ApplicationHandler<Renderer> for App<S, I> {
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
                    on_quit(&mut self.state);
                }
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.timer.update();
                if let Some(r) = self.renderer.as_mut() {
                    let Self {
                        state,
                        input,
                        timer,
                        ..
                    } = self;

                    let mut graphics = Graphics::new(r);
                    let mut cx = Context {
                        graphics: &mut graphics,
                        input,
                        timer,
                    };
                    for p in &mut self.plugins {
                        p.update(state, &mut cx);
                    }
                    for p in &mut self.plugins {
                        p.render(state, &mut cx);
                    }

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
            let Self { state, window, .. } = self;
            let mut ctx = InitContext {
                window: window.as_ref().unwrap().clone(),
                render: &mut renderer,
            };

            for plugin in &mut self.plugins {
                plugin.init(state, &mut ctx);
            }
            init(state, &mut ctx);
            self.renderer = Some(renderer);
        }
    }
}

impl<S: 'static, I: InitFn<S>> App<S, I> {
    /// Creates a new `App` with defined state & init logic
    pub fn init(state: S, init: I) -> Self {
        Self {
            state,
            init: Some(init),
            on_quit: None,
            plugins: Vec::new(),
            window: None,
            proxy: None,
            renderer: None,
            input: Input::default(),
            timer: FrameTimer::default(),
        }
    }

    /// Starts the app & runs the event loop
    pub fn run(mut self, update: impl FnMut(&mut S, &mut Context) + 'static) {
        let event_loop = EventLoop::<Renderer>::with_user_event().build().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        self.proxy = Some(event_loop.create_proxy());
        self.plugins.insert(0, Box::new(update));

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
    pub fn on_quit<F: FnMut(&mut S) + 'static>(&mut self, f: F) {
        self.on_quit = Some(Box::new(f));
    }

    /// Adds a plugin that receives `init()` & `update()` hooks
    pub fn plugin<P: Plugin<S> + 'static>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
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

/// Simple plugin trait to hook into `App` lifecycle
pub trait Plugin<S> {
    fn init(&mut self, state: &mut S, ctx: &mut InitContext);
    fn update(&mut self, state: &mut S, ctx: &mut Context);
    /// Called *after* all `update()`s, but *before* the batch is flushed.  
    fn render(&mut self, _state: &S, _ctx: &mut Context) {}
}

impl<T, S> Plugin<S> for T
where
    T: FnMut(&mut S, &mut Context),
{
    fn init(&mut self, _state: &mut S, _ctx: &mut InitContext) {}
    fn update(&mut self, state: &mut S, ctx: &mut Context) {
        self(state, ctx);
    }
    // `render` falls back to default no-op
}
