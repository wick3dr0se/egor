use egor::{
    Color,
    app::App,
    input::{KeyCode, MouseButton},
};
use rand::Rng;

struct Bullet {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}
struct Enemy {
    x: f32,
    y: f32,
    speed: f32,
}

fn main() {
    let (mut px, mut py) = (0.0, 0.0);
    let mut bullets = Vec::new();
    let mut rng = rand::thread_rng();

    let mut enemies = (0..50)
        .map(|_| {
            let a = rng.gen_range(0.0..std::f32::consts::TAU);
            let d = rng.gen_range(300.0..800.0);
            Enemy {
                x: a.cos() * d,
                y: a.sin() * d,
                speed: rng.gen_range(0.5..1.5),
            }
        })
        .collect::<Vec<_>>();

    let mut game_over = false;

    App::init(|ctx| ctx.set_title("Demo Egor Shooter")).run(move |t, g, i| {
        if game_over {
            return;
        }

        let [w, h] = g.screen_size();
        let (mx, my) = i.mouse_position();
        let (cx, cy) = (px + (mx - w / 2.0), py + (my - h / 2.0));

        px += (i.key_held(KeyCode::KeyD) as i8 - i.key_held(KeyCode::KeyA) as i8) as f32 * 5.0;
        py += (i.key_held(KeyCode::KeyS) as i8 - i.key_held(KeyCode::KeyW) as i8) as f32 * 5.0;
        g.camera().target(px, py);

        if i.mouse_pressed(MouseButton::Left) {
            let dx = cx - px;
            let dy = cy - py;
            let len = (dx * dx + dy * dy).sqrt();
            bullets.push(Bullet {
                x: px,
                y: py,
                vx: dx / len * 10.0,
                vy: dy / len * 10.0,
            });
        }

        for e in &mut enemies {
            let dx = px - e.x;
            let dy = py - e.y;
            let len = (dx * dx + dy * dy).sqrt().max(0.001);
            e.x += dx / len * e.speed;
            e.y += dy / len * e.speed;
        }

        bullets.retain(|b| {
            let mut hit = false;
            enemies.retain(|e| {
                if ((b.x - e.x).powi(2) + (b.y - e.y).powi(2)).sqrt() < 10.0 {
                    hit = true;
                    false
                } else {
                    true
                }
            });
            !hit
        });

        for b in &mut bullets {
            b.x += b.vx;
            b.y += b.vy;
            g.rect().at(b.x, b.y).size(5.0, 10.0).color(Color::BLUE);
        }

        for e in &enemies {
            let angle = (py - e.y).atan2(px - e.x);
            if ((px - e.x).powi(2) + (py - e.y).powi(2)).sqrt() < 15.0 {
                game_over = true;
            }
            g.tri()
                .at(e.x, e.y)
                .size(20.0)
                .rotation(angle)
                .color(Color::RED);
        }

        g.rect()
            .at(px, py)
            .size(20.0, 20.0)
            .rotation((cy - py).atan2(cx - px))
            .color(Color::GREEN);
    });
}
