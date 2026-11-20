//! Every path users are expected to write. If this stops compiling, the public
//! surface changed, whether or not the docs mention it.

use egor::{
    app::{App, FrameContext, WindowEvent},
    input::{Input, KeyCode, MouseButton},
    math::{IVec2, Rect, Vec2, ivec2, vec2},
    render::{
        Align, BorderRadii, Camera, ClipRect, Color, Graphics, MemoryHints, OffscreenTarget, Pivot,
        RenderTarget, Space,
    },
    time::FrameTimer,
};

#[allow(dead_code)]
fn app_types(_: App, _: FrameContext<'_>, _: WindowEvent, _: Input, _: KeyCode, _: MouseButton) {}

#[allow(dead_code)]
fn draw_types(_: Graphics<'_>, _: Camera, _: Color, _: ClipRect) {}

#[allow(dead_code)]
fn render_types(_: Space, _: MemoryHints, _: OffscreenTarget, _: fn(&dyn RenderTarget)) {}

#[allow(dead_code)]
fn math_types(_: IVec2, _: Rect, _: Vec2, _: Pivot, _: Align, _: BorderRadii) {}

#[allow(dead_code)]
fn time_type(_: FrameTimer) {}

#[allow(dead_code)]
fn constructors() {
    let _ = vec2(1.0, 2.0);
    let _ = ivec2(1, 2);
    let _ = Rect::new(vec2(0.0, 0.0), vec2(1.0, 1.0));
    let _ = Color::RED;
    let _ = Pivot::Center;
    let _ = Align::MiddleCenter;
}

#[allow(dead_code)]
fn drawing(gfx: &mut Graphics<'_>, target: &mut OffscreenTarget, camera: &mut Camera) {
    gfx.clear(Color::BLACK);
    gfx.rect()
        .at(vec2(0.0, 0.0))
        .size(vec2(1.0, 1.0))
        .color(Color::RED)
        .pivot(Pivot::Center)
        .layer(1)
        .screen();
    let _ = gfx.screen_size();
    let _ = camera.zoom();
    let _ = gfx.load_texture_raw(1, 1, &[0, 0, 0, 0]);
    let _ = gfx.create_offscreen(4, 4);
    let _ = gfx.offscreen_as_texture(target);
    gfx.with_clip(Rect::new(vec2(0.0, 0.0), vec2(8.0, 8.0)), |gfx| {
        gfx.text("label")
            .at(vec2(0.0, 0.0))
            .size(12.0)
            .color(Color::BLACK)
            .anchor(Align::MiddleCenter)
            .layer(2)
            .world();
    });
}

#[cfg(feature = "ui")]
#[allow(dead_code)]
fn egui_is_reachable() {
    let _ = core::any::type_name::<egor::app::egui::Context>();
}
