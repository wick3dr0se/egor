use std::sync::Arc;

use pollster::block_on;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::renderer::{Renderer, graphics::Graphics};

pub struct App<F> {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    update: Option<F>,
    pre_cached_textures: Vec<Vec<u8>>,
}

impl<F: FnMut(&mut Graphics)> App<F> {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            update: None,
            pre_cached_textures: Vec::new(),
        }
    }

    pub fn with_texture(mut self, data: &[u8]) -> Self {
        self.pre_cached_textures.push(data.into());
        self
    }

    pub fn run(self, update: F) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop
            .run_app(&mut Self {
                update: Some(update),
                ..self
            })
            .unwrap();
    }
}

impl<F: FnMut(&mut Graphics)> ApplicationHandler for App<F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let mut renderer = block_on(Renderer::new(window.clone()));

        for data in &self.pre_cached_textures {
            renderer.add_texture(data);
        }

        window.request_redraw();

        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.renderer.as_mut().map(|r| {
                    self.update.as_mut().unwrap()(&mut r.graphics());

                    r.render();
                    self.window.as_ref().unwrap().request_redraw();
                });
            }
            _ => {}
        }
    }
}
