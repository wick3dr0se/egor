mod render;

use render::{Renderer, vertex::Vertex};
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

trait UpdateFn: FnMut(&mut Context) + 'static {}
impl<F: FnMut(&mut Context) + 'static> UpdateFn for F {}

struct Context<'a> {
    render: &'a mut Renderer,
}

impl<'a> Context<'a> {
    fn draw_triangle(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let vertices = [
            Vertex::new([-0.5 * w + x, -0.5 * h + y], [1.0, 0.0, 0.0, 1.0]),
            Vertex::new([0.5 * w + x, -0.5 * h + y], [0.0, 1.0, 0.0, 1.0]),
            Vertex::new([x, 0.5 * h + y], [0.0, 0.0, 1.0, 1.0]),
        ];
        let indices = [0, 1, 2];
        self.render.submit_geometry(&vertices, &indices);
    }

    fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let vertices = [
            Vertex::new([x, y], [1.0, 0.0, 0.0, 1.0]),
            Vertex::new([x + w, y], [0.0, 1.0, 0.0, 1.0]),
            Vertex::new([x + w, y + h], [0.0, 0.0, 1.0, 1.0]),
            Vertex::new([x, y + h], [1.0, 1.0, 0.0, 1.0]),
        ];
        let indices = [0, 1, 2, 2, 3, 0];
        self.render.submit_geometry(&vertices, &indices);
    }
}

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
                    self.update.as_mut().unwrap()(&mut Context { render: renderer });
                }

                self.renderer.as_ref().map(|r| r.render_frame());
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
    App::new().run(|ctx| {
        ctx.draw_triangle(-0.5, 0.5, 0.5, 0.5);
        ctx.draw_triangle(0.5, -0.5, 0.5, 0.5);
        ctx.draw_triangle(-0.5, -0.5, 0.5, 0.5);
        ctx.draw_triangle(0.5, 0.5, 0.5, 0.5);

        ctx.draw_rect(-0.5, -0.5, 1.0, 1.0);
    });
}
