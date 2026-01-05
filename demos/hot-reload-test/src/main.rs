use egor::{
    app::App,
    math::{Vec2, vec2},
    render::Color,
};

fn main() {
    App::new().title("Hot Reload Demo").run(move |gfx, _, _| {
        // Feel free to change this code and see hot-reload in action!
        gfx.rect()
            .at(vec2(0., 0.))
            .size(Vec2::splat(100.0))
            .color(Color::RED);
    });
}
