use egor::{app::App, input::KeyCode, renderer::graphics::Color};

fn main() {
    let tiger = include_bytes!("assets/ghostscript_tiger.png");
    let wizard = include_bytes!("assets/wizard.png");
    let mut pos = (0.0, 0.0);

    App::new()
        .with_texture(tiger)
        .with_texture(wizard)
        .run(|gfx, input| {
            gfx.clear(Color::GREEN);

            let up = input.keys_pressed(&[KeyCode::ArrowUp, KeyCode::KeyW]);
            let left = input.keys_pressed(&[KeyCode::ArrowLeft, KeyCode::KeyA]);
            let down = input.keys_pressed(&[KeyCode::ArrowDown, KeyCode::KeyS]);
            let right = input.keys_pressed(&[KeyCode::ArrowRight, KeyCode::KeyD]);
            let speed = 5.0;
            let vel = (
                (right as i8 - left as i8) as f32 * speed,
                (down as i8 - up as i8) as f32 * speed,
            );

            pos = (pos.0 + vel.0, pos.1 + vel.1);

            gfx.quad().at(-1.0, 0.0).texture(0).draw();
            gfx.quad().size(pos.0, pos.1).color(Color::BLUE).draw();
            gfx.quad().at(-1.0, -1.0).size(2.0, 1.0).texture(1).draw();

            gfx.circle().segments(100).draw();
        });
}
