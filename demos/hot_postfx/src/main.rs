use egor::{
    app::{App, FrameContext},
    math::{Vec2, vec2},
    render::{Color, Graphics, OffscreenTarget, RenderTarget},
};

use std::fs;

fn load_effect(gfx: &mut Graphics, effect: &str) -> usize {
    let common = fs::read_to_string("shaders/common.wgsl").unwrap();
    let fragment = fs::read_to_string(format!("shaders/{effect}.wgsl")).unwrap();
    gfx.load_shader(&(common + &fragment))
}

fn main() {
    let mut offscreen_target = None;
    let mut texture_id = 0;

    App::new()
        .title("Egor Hot Reload/Post Processing Demo")
        .window_size(800, 600)
        .run(move |FrameContext { gfx, .. }| {
            gfx.clear(Color::WHITE);
            let size = gfx.screen_size();
            let center = size * 0.5;

            let target_size = (size.x as u32, size.y as u32);

            if offscreen_target
                .as_ref()
                .is_none_or(|t: &OffscreenTarget| t.size() != target_size)
            {
                let mut offscreen = gfx.create_offscreen(target_size.0, target_size.1);
                texture_id = gfx.offscreen_as_texture(&mut offscreen);
                offscreen_target = Some(offscreen);
            }

            // HOT RELOAD: change this line and save to swap effects live!
            // Try swapping to: vignette, crt, pixelate
            let shader = load_effect(gfx, "pixelate");

            gfx.render_offscreen(offscreen_target.as_mut().unwrap(), |gfx| {
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
            });

            gfx.with_shader(shader, |gfx| {
                gfx.rect()
                    .at(vec2(0., 0.))
                    .size(size)
                    .texture(texture_id)
                    .color(Color::WHITE);
            });
        });
}
