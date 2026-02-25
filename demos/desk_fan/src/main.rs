use egor::{
    app::{App, FrameContext},
    math::vec2,
    render::Color,
};

struct GameState {
    rotation: f32,
}

fn main() {
    let mut state = GameState { rotation: 0.0 };
    App::new()
        .title("Egor Desk Fan Demo")
        .run(move |FrameContext { gfx, timer, .. }| {
            let speed = 5.8;
            state.rotation += speed * timer.delta;
            let position = gfx.screen_size() / 2.0;
            let blade_length = 120.0;
            let blade_width = 40.0;

            // BASE
            gfx.path()
                .at(position + vec2(0.0, 200.0))
                .scale(vec2(1.5, 1.0))
                .thickness(3.0)
                .stroke_color(Color::BLACK)
                .fill_color(Color::new([0.2, 0.2, 0.2, 1.0]))
                .begin(vec2(-60.0, 0.0))
                .line_to(vec2(60.0, 0.0))
                .line_to(vec2(80.0, 40.0))
                .line_to(vec2(-80.0, 40.0))
                .close();

            // STAND
            gfx.path()
                .at(position)
                .thickness(12.0)
                .stroke_color(Color::new([0.3, 0.3, 0.3, 1.0]))
                .begin(vec2(0.0, 30.0))
                .line_to(vec2(0.0, 200.0));

            // BLADES
            let k = 0.552_284_8;
            let r = blade_width * 0.5;
            let tip_x = blade_length;
            for i in 0..4 {
                let base_angle = i as f32 * std::f32::consts::FRAC_PI_2;
                gfx.path()
                    .at(position)
                    .rotate(state.rotation + base_angle)
                    .scale(vec2(1.2, 1.0))
                    .thickness(2.0)
                    .stroke_color(Color::BLACK)
                    .fill_color(Color::new([0.5, 0.3, 0.9, 1.0]))
                    .begin(vec2(0.0, -r))
                    .line_to(vec2(tip_x - r, -r))
                    .cubic_to(
                        vec2(tip_x - r + r * k, -r),
                        vec2(tip_x, -r + r * k),
                        vec2(tip_x, 0.0),
                    )
                    .cubic_to(
                        vec2(tip_x, r - r * k),
                        vec2(tip_x - r + r * k, r),
                        vec2(tip_x - r, r),
                    )
                    .line_to(vec2(0.0, r))
                    .close();
            }

            // CENTER HUB
            gfx.path()
                .at(position)
                .scale(vec2(1.1, 1.1))
                .thickness(3.0)
                .stroke_color(Color::BLACK)
                .fill_color(Color::new([0.7, 0.7, 0.7, 1.0]))
                .circle(30.0);
        });
}
