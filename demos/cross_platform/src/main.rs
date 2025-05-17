use egor::{Color, KeyCode, app::App};

fn main() {
    let (mut x, mut y) = (0.0, 0.0);

    App::init(|ctx| {
        ctx.set_title("Egor Cross Platform Demo");
        ctx.load_texture(include_bytes!("../../assets/ghostscript_tiger.png"));
    })
    .run(move |g, i| {
        let [w, h] = g.screen_size();
        let (hw, hh) = (w / 2.0, h / 2.0);

        let up = i.keys_held(&[KeyCode::ArrowUp, KeyCode::KeyW]);
        let left = i.keys_held(&[KeyCode::ArrowLeft, KeyCode::KeyA]);
        let down = i.keys_held(&[KeyCode::ArrowDown, KeyCode::KeyS]);
        let right = i.keys_held(&[KeyCode::ArrowRight, KeyCode::KeyD]);

        let speed = 5.0;
        x += (right as i8 - left as i8) as f32 * speed;
        y += (down as i8 - up as i8) as f32 * speed;
        g.camera().target(x, y);

        g.tri().at(-hw, -hh);
        g.tri().at(hw, hh);

        g.tri().at(hw, -hh).color(Color::BLUE);
        g.tri().at(-hw, hh).color(Color::BLUE);

        g.rect().at(x, y).size(256.0, 256.0).texture(0);
    });
}
