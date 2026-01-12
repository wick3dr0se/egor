use egor::{
    app::{App, FrameContext},
    input::MouseButton,
    math::Vec2,
    math::vec2,
    render::Color,
};

struct ClickDemo {
    squares: Vec<Vec2>,
    square_size: f32,
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    let mut state = ClickDemo {
        squares: vec![],
        square_size: 50.0,
    };

    App::new()
        .window_size(800, 600)
        .title("Position Click Demo")
        .run(move |FrameContext { gfx, input, .. }| {
            // Clear background to a light color
            gfx.clear(Color::new([0.9, 0.9, 0.95, 1.0]));

            // Check for mouse clicks and add new square positions
            if input.mouse_pressed(MouseButton::Left) {
                let mouse_pos = input.mouse_position();
                let screen_pos = vec2(mouse_pos.0, mouse_pos.1);
                let screen_size = gfx.screen_size();
                let world_pos = gfx.camera().screen_to_world(screen_pos, screen_size);
                state.squares.push(world_pos);
            }

            // Check for touch input and add new square positions
            if input.touch_pressed() {
                if let Some((touch_x, touch_y)) = input.touch_position() {
                    let screen_pos = vec2(touch_x, touch_y);
                    let screen_size = gfx.screen_size();
                    let world_pos = gfx.camera().screen_to_world(screen_pos, screen_size);
                    state.squares.push(world_pos);
                }
            }

            // Draw all the green squares
            for pos in &state.squares {
                gfx.rect()
                    .at(*pos)
                    .size(Vec2::splat(state.square_size))
                    .color(Color::GREEN);
            }

            // Draw instruction text
            gfx.text("Click or touch to draw green squares!")
                .color(Color::BLACK)
                .at(vec2(10.0, 10.0));

            gfx.text(&format!("Squares: {}", state.squares.len()))
                .color(Color::BLACK)
                .at(vec2(10.0, 30.0));

            // Show touch status
            let touch_status = if input.touch_active() {
                "Touch: Active"
            } else if input.touch_position().is_some() {
                "Touch: Enabled"
            } else {
                "Touch: Not Detected"
            };
            gfx.text(touch_status)
                .color(Color::BLACK)
                .at(vec2(10.0, 50.0));
        });
}
