use egor::{Color, KeyCode, app::App};

fn main() {
    let mut pos = (0.0, 0.0);

    App::init(|ctx| {
        ctx.set_title("Egor");
        ctx.load_texture(include_bytes!("../../assets/ghostscript_tiger.png"));
    })
    .run(move |g, i| {
        let [cx, cy] = [g.screen_size()[0] / 2.0, g.screen_size()[1] / 2.0];

        let size = 512.0;
        let half = size / 2.0;

        let up = i.keys_held(&[KeyCode::ArrowUp, KeyCode::KeyW]);
        let left = i.keys_held(&[KeyCode::ArrowLeft, KeyCode::KeyA]);
        let down = i.keys_held(&[KeyCode::ArrowDown, KeyCode::KeyS]);
        let right = i.keys_held(&[KeyCode::ArrowRight, KeyCode::KeyD]);
        let speed = 5.0;
        let vel = (
            (right as i8 - left as i8) as f32 * speed,
            (down as i8 - up as i8) as f32 * speed,
        );
        pos = (pos.0 + vel.0, pos.1 + vel.1);

        g.tri().at(cx - half, cy - half).color(Color::GREEN);
        g.tri().at(cx + half, cy - half);
        g.tri().at(cx + half, cy + half).color(Color::GREEN);
        g.tri().at(cx - half, cy + half);
        g.rect().at(pos.0, pos.1).size(size, size).texture(0);
    });
}
