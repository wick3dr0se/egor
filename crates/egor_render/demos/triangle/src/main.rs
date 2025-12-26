use std::error::Error;
use std::sync::Arc;

use egor_render::{GeometryBatch, Renderer, color::Color, vertex::Vertex};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes};

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = MinimalApp::new();
    Ok(event_loop.run_app(&mut app)?)
}

struct MinimalApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    batch: Option<GeometryBatch>,
}

impl MinimalApp {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            batch: None,
        }
    }
}

impl ApplicationHandler for MinimalApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default())
                .unwrap(),
        );
        let size = window.inner_size();

        // Init renderer
        let renderer = pollster::block_on(Renderer::new(size.width, size.height, window.clone()));

        // Minimal triangle
        let mut batch = GeometryBatch::default();
        let vertices = [
            Vertex::new([0.0, 0.5], Color::RED, [0.0, 0.0]),
            Vertex::new([-0.5, -0.5], Color::GREEN, [0.0, 0.0]),
            Vertex::new([0.5, -0.5], Color::BLUE, [0.0, 0.0]),
        ];
        let indices = [0, 1, 2];
        batch.push(&vertices, &indices);

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.batch = Some(batch);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let (Some(r), Some(g)) = (self.renderer.as_mut(), self.batch.as_ref()) {
            match event {
                WindowEvent::RedrawRequested => {
                    r.clear_color = Color::BLACK;
                    r.render_frame(vec![(0, g.clone())]);
                }
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(size) => {
                    r.resize(size.width, size.height);
                }
                _ => {}
            }
        }
    }
}
