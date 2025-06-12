use dioxus_devtools::{connect_subsecond, subsecond};
use egor_app::{Context, InitContext, Plugin};

pub struct HotReloadPlugin<T> {
    internal: T,
}

impl<T: Plugin> HotReloadPlugin<T> {
    pub fn new(internal: T) -> Self {
        Self { internal }
    }
}

impl<T: Plugin> Plugin for HotReloadPlugin<T> {
    fn init(&mut self, ctx: &mut InitContext) {
        connect_subsecond();
        self.internal.init(ctx);
    }

    fn update(&mut self, ctx: &mut Context) {
        subsecond::call(|| self.internal.update(ctx));
    }
}
