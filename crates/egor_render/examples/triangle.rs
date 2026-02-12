use std::error::Error;
use std::sync::Arc;

use egor_render::{Backbuffer, RenderTarget};
use egor_render::{GeometryBatch, Renderer, vertex::Vertex};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = MinimalApp::default();
    Ok(event_loop.run_app(&mut app)?)
}

#[derive(Default)]
struct MinimalApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    batch: GeometryBatch,
    backbuffer: Option<Backbuffer>,
}

impl ApplicationHandler for MinimalApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Default::default()).unwrap());
        let size = window.inner_size();

        let renderer = pollster::block_on(Renderer::new(window.clone()));
        let backbuffer = Backbuffer::new(
            renderer.instance(),
            renderer.adapter(),
            renderer.device(),
            window.clone(),
            size.width,
            size.height,
        );

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.backbuffer = Some(backbuffer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(r) = self.renderer.as_mut() {
            match event {
                WindowEvent::RedrawRequested => {
                    let Some(backbuffer) = &mut self.backbuffer else {
                        return;
                    };
                    let Some(mut frame) = r.begin_frame(backbuffer) else {
                        return;
                    };

                    let vertices = [
                        Vertex::new([0.0, 0.5], [1.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
                        Vertex::new([-0.5, -0.5], [0.0, 1.0, 0.0, 1.0], [0.0, 0.0]),
                        Vertex::new([0.5, -0.5], [0.0, 0.0, 1.0, 1.0], [0.0, 0.0]),
                    ];
                    let indices = [0, 1, 2];

                    if let Some((batch_verts, batch_indices, base)) =
                        self.batch.allocate(vertices.len(), indices.len())
                    {
                        batch_verts.copy_from_slice(&vertices);
                        batch_indices.copy_from_slice(&indices.map(|i| i + base));
                    }

                    {
                        let mut r_pass = r.begin_render_pass(&mut frame.encoder, &frame.view);
                        r.draw_batch(&mut r_pass, &mut self.batch, 0);
                    }
                    r.end_frame(frame);
                }
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(size) => {
                    if let Some(backbuffer) = &mut self.backbuffer {
                        backbuffer.resize(r.device(), size.width, size.height);
                    }
                }
                _ => {}
            }
        }
    }
}
