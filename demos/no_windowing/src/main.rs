use std::error::Error;
use std::sync::Arc;

use egor::Color;
use egor::render::primitives::Anchor;
use egor::render::{Graphics, Renderer};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    let mut app = Application::new();
    Ok(event_loop.run_app(&mut app)?)
}

struct Application {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    mouse_position: (f32, f32),
    dpi: f64,
}

impl Application {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            mouse_position: (0.0, 0.0),
            dpi: 1.0,
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let inner_width = window.inner_size().width;
        let inner_height = window.inner_size().height;
        let renderer = pollster::block_on(Renderer::create_graphics(
            inner_width,
            inner_height,
            window.clone(),
        ));
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Self {
            window: Some(window),
            renderer: Some(renderer),
            mouse_position,
            dpi,
        } = self
        else {
            return;
        };
        match event {
            WindowEvent::RedrawRequested => {
                window.pre_present_notify();
                let width = renderer.screen_width();
                let height = renderer.screen_height();
                let mut g = Graphics::new(renderer);
                g.clear(Color::BLACK);
                g.rect().anchor(Anchor::Center).at(
                    mouse_position.0 - (width * 0.5),
                    mouse_position.1 - (height * 0.5),
                );
                renderer.render_frame();
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.mouse_position = position.into();
                window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                *dpi = scale_factor;
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}
