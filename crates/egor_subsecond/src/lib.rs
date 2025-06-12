use dioxus_devtools::{connect_subsecond, subsecond};
use egor_app::{Context, InitContext, Plugin};

pub struct HotReloadPlugin<T, S> {
    internal: T,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, T: Plugin<S>> HotReloadPlugin<T, S> {
    pub fn new(internal: T) -> Self {
        Self {
            internal,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, T: Plugin<S>> Plugin<S> for HotReloadPlugin<T, S> {
    fn init(&mut self, ctx: &mut InitContext<S>) {
        connect_subsecond();
        self.internal.init(ctx);
    }

    fn update(&mut self, ctx: &mut Context<S>) {
        subsecond::call(|| self.internal.update(ctx));
    }
}
