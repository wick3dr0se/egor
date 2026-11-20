pub use egui;

use egui::{ClippedPrimitive, Context, TexturesDelta};
use egui_wgpu::ScreenDescriptor;
use egui_wgpu::wgpu::{
    CommandEncoder, Device, LoadOp, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};
use egui_winit::State;
use egui_winit::winit::{event::WindowEvent, window::Window};

pub struct EguiFrame {
    pub clipped_primitives: Vec<ClippedPrimitive>,
    pub textures_delta: TexturesDelta,
    pub pixels_per_point: f32,
}

pub struct EguiRenderer {
    pub ctx: Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

impl EguiRenderer {
    pub fn new(device: &Device, surface_format: TextureFormat, window: &Window) -> Self {
        let ctx = Context::default();
        let viewport_id = ctx.viewport_id();
        let state = State::new(ctx.clone(), viewport_id, window, None, None, None);
        let renderer =
            egui_wgpu::Renderer::new(device, surface_format, Default::default(), 1, false);

        Self {
            ctx,
            state,
            renderer,
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    pub fn begin_frame(&mut self, window: &Window) -> &Context {
        let raw_input = self.state.take_egui_input(window);
        self.ctx.begin_pass(raw_input);
        &self.ctx
    }

    pub fn end_frame(&mut self, window: &Window) -> EguiFrame {
        let output = self.ctx.end_pass();
        self.state
            .handle_platform_output(window, output.platform_output);

        EguiFrame {
            clipped_primitives: self.ctx.tessellate(output.shapes, output.pixels_per_point),
            textures_delta: output.textures_delta,
            pixels_per_point: output.pixels_per_point,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        width: u32,
        height: u32,
        frame: EguiFrame,
    ) {
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: frame.pixels_per_point,
        };

        for (id, image_delta) in &frame.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        self.renderer.update_buffers(
            device,
            queue,
            encoder,
            &frame.clipped_primitives,
            &screen_descriptor,
        );

        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("egui"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.renderer.render(
            &mut render_pass.forget_lifetime(),
            &frame.clipped_primitives,
            &screen_descriptor,
        );

        for id in &frame.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
