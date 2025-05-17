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
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut rng = rand::rng();
    let mut enemies: Vec<Enemy> = (0..50)
        .map(|_| {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let dist = rng.random_range(300.0..800.0);
            Enemy {
                x: px + angle.cos() * dist,
                y: py + angle.sin() * dist,
                speed: rng.random_range(0.5..1.5),
            }
        })
        .collect();

    App::init(|ctx| {
        ctx.set_title("Egor Shooter Demo");
    })
    .run(move |g, i| {
        let [w, h] = g.screen_size();
        let (hw, hh) = (w / 2.0, h / 2.0);
        let (mx, my) = i.mouse_position();
        let cx = px + (mx - hw);
        let cy = py + (my - hh);

        let up = i.keys_held(&[KeyCode::ArrowUp, KeyCode::KeyW]);
        let left = i.keys_held(&[KeyCode::ArrowLeft, KeyCode::KeyA]);
        let down = i.keys_held(&[KeyCode::ArrowDown, KeyCode::KeyS]);
        let right = i.keys_held(&[KeyCode::ArrowRight, KeyCode::KeyD]);

        let speed = 5.0;
        px += (right as i8 - left as i8) as f32 * speed;
        py += (down as i8 - up as i8) as f32 * speed;

        g.camera().target(px, py);

        if i.mouse_pressed(MouseButton::Left) {
            let dx = cx - px;
            let dy = cy - py;
            let len = (dx * dx + dy * dy).sqrt();
            let norm_x = dx / len;
            let norm_y = dy / len;
            let bullet_speed = 10.0;

            bullets.push(Bullet {
                x: px,
                y: py,
                vx: norm_x * bullet_speed,
                vy: norm_y * bullet_speed,
            });
        }

        for e in &mut enemies {
            let dx = px - e.x;
            let dy = py - e.y;
            let len = (dx * dx + dy * dy).sqrt().max(0.001);
            let dir_x = dx / len;
            let dir_y = dy / len;
            e.x += dir_x * e.speed;
            e.y += dir_y * e.speed;
        }

        bullets.retain(|b| {
            let mut hit = false;
            enemies.retain(|e| {
                let dist = ((b.x - e.x).powi(2) + (b.y - e.y).powi(2)).sqrt();
                if dist < 10.0 {
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
            let dist = ((px - e.x).powi(2) + (py - e.y).powi(2)).sqrt();
            if dist < 15.0 {
                println!("Game Over");
                std::process::exit(0);
            }
            g.tri()
                .at(e.x, e.y)
                .size(20.0)
                .rotation(angle)
                .color(Color::RED);
        }

        let angle = (cy - py).atan2(cx - px);
        g.rect()
            .at(px, py)
            .size(20.0, 20.0)
            .rotation(angle)
            .color(Color::GREEN);
    });
}
