use egor::{
    app::{App, FrameContext},
    hot::{HotFn, HotFnPtr},
    math::{Vec2, vec2},
    render::{Align, Color, Graphics, OffscreenTarget},
};

use std::fs;

fn load_effect(gfx: &mut Graphics, effect: &str) -> usize {
    let common = fs::read_to_string("shaders/common.wgsl").unwrap();
    let fragment = fs::read_to_string(format!("shaders/{effect}.wgsl")).unwrap();
    gfx.load_shader(&(common + &fragment))
}

struct PostFx {
    target: OffscreenTarget,
    texture: usize,
    size: (u32, u32),
    effect: &'static str,
}

/// The target, its texture and the effect, rebuilt when this function changes.
///
/// A function item rather than a closure, because hot reload patches functions and
/// cannot wrap captures.
fn new_postfx(gfx: &mut Graphics) -> PostFx {
    let screen = gfx.screen_size();
    let size = (screen.x as u32, screen.y as u32);
    let mut target = gfx.create_offscreen(size.0, size.1);

    PostFx {
        texture: gfx.offscreen_as_texture(&mut target),
        target,
        size,
        // HOT RELOAD: edit this and the state rebuilds, swapping the effect live.
        // Try: vignette, crt, pixelate
        effect: "pixelate",
    }
}

fn main() {
    let mut postfx: Option<PostFx> = None;
    let mut postfx_ptr: Option<HotFnPtr> = None;

    App::new()
        .title("Egor Hot Reload/Post Processing Demo")
        .window_size(800, 600)
        .run(move |FrameContext { gfx, .. }| {
            // a patched struct can be read with its old layout, so rebuild when the
            // function that built it changes
            let mut state_fn = HotFn::current(new_postfx);
            let ptr = state_fn.ptr_address();
            if postfx_ptr != Some(ptr) {
                postfx_ptr = Some(ptr);
                postfx = Some(state_fn.call((gfx,)));
            }
            let postfx = postfx.as_mut().unwrap();

            gfx.clear(Color::WHITE);
            let size = gfx.screen_size();
            let center = size * 0.5;

            let wanted = (size.x as u32, size.y as u32);
            if postfx.size != wanted {
                let mut target = gfx.create_offscreen(wanted.0, wanted.1);
                postfx.texture = gfx.offscreen_as_texture(&mut target);
                postfx.target = target;
                postfx.size = wanted;
            }

            // the shader comes off disk every frame, so editing the wgsl is live too
            let shader = load_effect(gfx, postfx.effect);

            gfx.render_offscreen(&mut postfx.target, |gfx| {
                gfx.rect()
                    .at(center - Vec2::splat(100.0))
                    .size(Vec2::splat(200.0))
                    .color(Color::new([1.0, 0.3, 0.5, 1.0]));
                gfx.rect()
                    .at(vec2(size.x * 0.1, size.y * 0.7))
                    .size(vec2(size.x * 0.15, size.y * 0.1))
                    .color(Color::new([0.2, 0.8, 1.0, 1.0]));
                gfx.polygon()
                    .at(vec2(size.x * 0.75, size.y * 0.5))
                    .radius(size.y * 0.1)
                    .segments(32)
                    .color(Color::new([1.0, 0.8, 0.2, 1.0]));

                // Text queued here belongs to this target, so the postfx shader distorts
                // it with the shapes rather than drawing it crisp over the top.
                gfx.text("offscreen")
                    .at(vec2(size.x * 0.1, size.y * 0.2))
                    .size(48.0)
                    .color(Color::BLACK);
            });

            gfx.with_shader(shader, |gfx| {
                gfx.rect()
                    .at(vec2(0., 0.))
                    .size(size)
                    .texture(postfx.texture)
                    .color(Color::WHITE);
            });

            gfx.text(postfx.effect)
                .at(vec2(size.x - 10.0, size.y - 10.0))
                .anchor(Align::BottomRight)
                .size(14.0)
                .color(Color::new([0.3, 0.3, 0.34, 1.0]));
        });
}
