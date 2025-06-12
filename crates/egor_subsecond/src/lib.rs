use dioxus_devtools::{connect_subsecond, subsecond};
use egor_app::{Context, InitContext, Plugin};

pub struct HotReloadPlugin;

impl Plugin for HotReloadPlugin {
    fn init(&mut self, _ctx: &mut InitContext) {
        connect_subsecond();
    }

    fn update(&mut self, next: &mut dyn FnMut(&mut Context), ctx: &mut Context) {
        subsecond::call(|| next(ctx));
    }
}
