use dioxus_devtools::{connect_subsecond, subsecond};
use egor_app::{InitContext, Plugin, input::Input, time::FrameTimer};
use egor_render::Graphics;

pub struct HotReloadPlugin;

impl Plugin for HotReloadPlugin {
    fn init(&mut self, _ctx: &mut InitContext) {
        connect_subsecond();
    }

    fn update(
        &mut self,
        next: &mut dyn FnMut(&FrameTimer, &mut Graphics, &mut Input),
        timer: &FrameTimer,
        graphics: &mut Graphics,
        input: &mut Input,
    ) {
        subsecond::call(|| next(timer, graphics, input));
    }
}
