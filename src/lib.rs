mod renderer;

use renderer::Renderer;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

pub use wgpu::Color;

#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;
#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;

pub trait InitFn: FnOnce(Context) + 'static {}
impl<T: FnOnce(Context) + 'static> InitFn for T {}
pub trait UpdateFn: FnMut(&mut Context) + 'static {}
impl<T: FnMut(&mut Context) + 'static> UpdateFn for T {}

pub struct Context<'a> {
    pub window: Rc<Window>,
    pub renderer: &'a mut Renderer,
}

pub struct App<I, U> {
    init: Option<I>,
    update: Option<U>,
    window: Option<Rc<Window>>,
    proxy: Option<EventLoopProxy<Renderer>>,
    renderer: Option<Renderer>,
}

impl<I: InitFn, U: UpdateFn> ApplicationHandler<Renderer> for App<I, U> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(proxy) = self.proxy.take() {
            let win_attrs = {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Window::default_attributes()
                }
                #[cfg(target_arch = "wasm32")]
                {
                    use winit::platform::web::WindowAttributesExtWebSys;
                    Window::default_attributes().with_append(true)
                }
            };

            let window = Rc::new(event_loop.create_window(win_attrs).unwrap());
            let gfx = Renderer::init(window.clone(), proxy);

            #[cfg(not(target_arch = "wasm32"))]
            pollster::block_on(gfx);
            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(gfx);

            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let (Some(update), Some(window), Some(renderer)) = (
                    self.update.as_mut(),
                    self.window.as_ref(),
                    self.renderer.as_mut(),
                ) {
                    update(&mut Context {
                        window: window.clone(),
                        renderer,
                    });

                    renderer.render_frame();
                    window.request_redraw();
                }
            }
            WindowEvent::Resized(size) => {
                self.renderer
                    .as_mut()
                    .map(|gfx| gfx.resize(size.width, size.height));
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, mut renderer: Renderer) {
        if let Some(init) = self.init.take() {
            init(Context {
                window: self.window.as_ref().unwrap().clone(),
                renderer: &mut renderer,
            });
        }

        self.renderer = Some(renderer);
        self.window.as_ref().map(|w| w.request_redraw());
    }
}

impl<I: InitFn, U: UpdateFn> App<I, U> {
    pub fn new(init: I) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Error)
                .init();
        }
        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Error).expect("Couldn't initialize logger");
        }

        Self {
            init: Some(init),
            update: None,
            window: None,
            proxy: None,
            renderer: None,
        }
    }

    pub fn run(mut self, update: U) {
        let event_loop = EventLoop::<Renderer>::with_user_event().build().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let proxy = event_loop.create_proxy();

        self.proxy = Some(proxy);
        self.update = Some(update);

        #[cfg(not(target_arch = "wasm32"))]
        event_loop.run_app(&mut self).unwrap();
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::EventLoopExtWebSys;
            wasm_bindgen_futures::spawn_local(async move {
                event_loop.spawn_app(self);
            });
        }
    }
}
