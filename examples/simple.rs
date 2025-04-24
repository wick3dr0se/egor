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

            let up = input.keys_held(&[KeyCode::ArrowUp, KeyCode::KeyW]);
            let left = input.keys_held(&[KeyCode::ArrowLeft, KeyCode::KeyA]);
            let down = input.keys_held(&[KeyCode::ArrowDown, KeyCode::KeyS]);
            let right = input.keys_held(&[KeyCode::ArrowRight, KeyCode::KeyD]);
            let speed = 5.0;
            let vel = (
                (right as i8 - left as i8) as f32 * speed,
                (down as i8 - up as i8) as f32 * speed,
            );
            let [w, h] = gfx.screen_size();

            pos = (pos.0 + vel.0, pos.1 + vel.1);

            gfx.quad()
                .at(0.0, 0.0)
                .size(w / 2.0, h / 2.0)
                .texture(0)
                .draw();
            gfx.quad()
                .at(0.0, h / 2.0)
                .size(w, h / 2.0)
                .texture(1)
                .draw();

            gfx.circle().segments(100).draw();

            gfx.quad().at(pos.0, pos.1).color(Color::BLUE).draw();
        });
}
