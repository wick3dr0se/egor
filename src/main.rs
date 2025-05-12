mod render;

use render::{Renderer, context::GraphicsContext};
use wgpu::Color;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

#[cfg(target_arch = "wasm32")]
pub type Rc<T> = std::rc::Rc<T>;
#[cfg(not(target_arch = "wasm32"))]
pub type Rc<T> = std::sync::Arc<T>;

trait UpdateFn: FnMut(&mut GraphicsContext) + 'static {}
impl<F: FnMut(&mut GraphicsContext) + 'static> UpdateFn for F {}

struct App<U> {
    window: Option<Rc<Window>>,
    proxy: Option<EventLoopProxy<Renderer>>,
    renderer: Option<Renderer>,
    update: Option<U>,
}

impl<U: UpdateFn> ApplicationHandler<Renderer> for App<U> {
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
                if let Some(renderer) = self.renderer.as_mut() {
                    self.update.as_mut().unwrap()(&mut GraphicsContext { renderer });
                }

                self.renderer.as_mut().map(|r| r.render_frame());
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

    fn user_event(&mut self, _: &ActiveEventLoop, renderer: Renderer) {
        self.renderer = Some(renderer);
    }
}

impl<U: UpdateFn> App<U> {
    fn new() -> Self {
        Self {
            window: None,
            proxy: None,
            renderer: None,
            update: None,
        }
    }

    fn run(mut self, update: U) {
        let event_loop = EventLoop::<Renderer>::with_user_event().build().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        let proxy = event_loop.create_proxy();
        self.proxy = Some(proxy);
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

fn main() {
    App::new().run(|g| {
        let [cx, cy] = [g.screen_size()[0] / 2.0, g.screen_size()[1] / 2.0];
        let size = 128.0;
        let half = size / 2.0;

        g.clear(Color::BLACK);

        g.tri().at(cx - half, cy - half).color(Color::GREEN);
        g.tri().at(cx + half, cy - half);
        g.tri().at(cx + half, cy + half).color(Color::GREEN);
        g.tri().at(cx - half, cy + half);
        g.rect().at(cx, cy).size(size, size);
    });
}
