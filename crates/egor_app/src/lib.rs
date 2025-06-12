pub mod time;
pub use winit::keyboard::KeyCode;

pub mod input;

#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;

#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

use egor_render::{Graphics, Renderer};

use crate::{input::Input, time::FrameTimer};

pub trait InitFn: FnOnce(&mut InitContext) + 'static {}
impl<F: FnOnce(&mut InitContext) + 'static> InitFn for F {}
pub trait UpdateFn: FnMut(&FrameTimer, &mut Graphics, &mut Input) + 'static {}
impl<F: FnMut(&FrameTimer, &mut Graphics, &mut Input) + 'static> UpdateFn for F {}

pub struct App<I, U> {
    window: Option<Rc<Window>>,
    proxy: Option<EventLoopProxy<Renderer>>,
    init: Option<I>,
    update: Option<U>,
    timer: FrameTimer,
    renderer: Option<Renderer>,
    input: Input,
    #[cfg(target_arch = "wasm32")]
    plugins: Vec<Box<dyn Plugin>>,
    #[cfg(not(target_arch = "wasm32"))]
    plugins: Vec<Box<dyn Plugin>>,
}

impl<I: InitFn, U: UpdateFn> ApplicationHandler<Renderer> for App<I, U> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.timer.update();
                if let Some(r) = self.renderer.as_mut() {
                    let mut graphics = Graphics::new(r);

                    let mut cx = Context {
                        timer: &self.timer,
                        graphics: &mut graphics,
                        input: &mut self.input,
                    };
                    for plugin in self.plugins.iter_mut() {
                        plugin.update(&mut cx);
                    }
                    r.render_frame();
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
        if let Some(init) = self.init.take() {
            let mut ctx = InitContext {
                window: self.window.as_ref().unwrap().clone(),
                render: &mut renderer,
            };

            for plugin in &mut self.plugins {
                #[cfg(target_arch = "wasm32")]
                plugin.init(&mut ctx);
                #[cfg(not(target_arch = "wasm32"))]
                plugin.init(&mut ctx);
            }
            init(&mut ctx);
            self.renderer = Some(renderer);
        }
    }
}

impl<I: InitFn, U: UpdateFn> App<I, U> {
    pub fn init(init: I) -> Self {
        Self {
            window: None,
            proxy: None,
            init: Some(init),
            update: None,
            timer: FrameTimer::new(),
            renderer: None,
            input: Input::default(),
            plugins: Vec::new(),
        }
    }

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

    pub fn plugin<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        #[cfg(target_arch = "wasm32")]
        self.plugins.push(Box::new(plugin));
        #[cfg(not(target_arch = "wasm32"))]
        self.plugins.push(Box::new(plugin));
        self
    }
}

pub struct InitContext<'a> {
    window: Rc<Window>,
    render: &'a mut Renderer,
}

impl InitContext<'_> {
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    pub fn load_texture(&mut self, data: &[u8]) -> usize {
        self.render.add_texture(data)
    }
}

pub struct Context<'a> {
    pub timer: &'a FrameTimer,
    pub graphics: &'a mut Graphics<'a>,
    pub input: &'a mut Input,
}

pub trait Plugin {
    fn init(&mut self, ctx: &mut InitContext);
    fn update(&mut self, ctx: &'_ mut Context<'_>);
}
