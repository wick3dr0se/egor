use egor::{app::App, renderer::graphics::Color};

fn main() {
    let tiger = include_bytes!("assets/ghostscript_tiger.png");
    let wizard = include_bytes!("assets/wizard.png");

    App::new()
        .with_texture(tiger)
        .with_texture(wizard)
        .run(|gfx| {
            gfx.clear(Color::GREEN);

            gfx.quad().at(-1.0, 0.0).texture(0).draw();

            gfx.quad().size(0.5, 1.0).color(Color::BLUE).draw();

            gfx.quad().at(-1.0, -1.0).size(2.0, 1.0).texture(1).draw();

            gfx.circle().segments(100).draw();
        });
}
