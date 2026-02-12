use wgpu::{CommandEncoder, Queue, SurfaceTexture, TextureView};

/// Trait for presenting rendered frames
pub trait Presentable {
    fn present(self: Box<Self>);
}

impl Presentable for SurfaceTexture {
    fn present(self: Box<Self>) {
        (*self).present();
    }
}

pub struct Frame {
    pub view: TextureView,
    pub encoder: CommandEncoder,
    pub(crate) presentable: Option<Box<dyn Presentable>>,
}

impl Frame {
    pub(crate) fn finish(self, queue: &Queue) {
        queue.submit(Some(self.encoder.finish()));
        if let Some(p) = self.presentable {
            p.present();
        }
    }
}
