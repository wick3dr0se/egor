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
    fn init(&mut self, state: &mut S, ctx: &mut InitContext) {
        connect_subsecond();
        self.internal.init(state, ctx);
    }

    fn update(&mut self, state: &mut S, ctx: &mut Context) {
        subsecond::call(|| self.internal.update(state, ctx));
    }
}
