use crate::{
    InitContext, Rc,
    render::{Renderer, context::GraphicsContext},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

pub trait InitFn: FnOnce(&mut InitContext) + 'static {}
impl<F: FnOnce(&mut InitContext) + 'static> InitFn for F {}
pub trait UpdateFn: FnMut(&mut GraphicsContext) + 'static {}
impl<F: FnMut(&mut GraphicsContext) + 'static> UpdateFn for F {}

pub struct App<I, U> {
    window: Option<Rc<Window>>,
    proxy: Option<EventLoopProxy<Renderer>>,
    renderer: Option<Renderer>,
    init: Option<I>,
    update: Option<U>,
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

            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(Renderer::create_graphics(window, proxy));
            #[cfg(not(target_arch = "wasm32"))]
            pollster::block_on(Renderer::create_graphics(window, proxy));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.renderer.as_mut().map(|r| {
                    self.update.as_mut().unwrap()(&mut GraphicsContext { renderer: r });
                    r.render_frame();
                });

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(size) => {
                self.renderer
                    .as_mut()
                    .map(|r| r.resize(size.width, size.height));
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, mut renderer: Renderer) {
        if let Some(init) = self.init.take() {
            init(&mut InitContext {
                window: self.window.as_ref().unwrap().clone(),
                render: &mut renderer,
            });
            self.renderer = Some(renderer);
        }
    }
}

impl<I: InitFn, U: UpdateFn> App<I, U> {
    pub fn init(init: I) -> Self {
        Self {
            window: None,
            proxy: None,
            renderer: None,
            init: Some(init),
            update: None,
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
}
