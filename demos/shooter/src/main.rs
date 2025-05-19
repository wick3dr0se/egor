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
    hp: u32,
    flash: f32,
}

fn spawn_wave(cx: f32, cy: f32, count: usize, speed_range: (f32, f32), hp: u32) -> Vec<Enemy> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            let a = rng.gen_range(0.0..std::f32::consts::TAU);
            let d = rng.gen_range(300.0..800.0);
            Enemy {
                x: cx + a.cos() * d,
                y: cy + a.sin() * d,
                speed: rng.gen_range(speed_range.0..speed_range.1),
                hp,
                flash: 0.0,
            }
        })
        .collect()
}

fn spawn_bullet_spread(
    px: f32,
    py: f32,
    target_x: f32,
    target_y: f32,
    count: usize,
) -> Vec<Bullet> {
    let mut bullets = Vec::new();
    let dx = target_x - px;
    let dy = target_y - py;
    let angle = dy.atan2(dx);
    let spread = 0.3;

    let half = (count as f32 - 1.0) / 2.0;
    for i in 0..count {
        let offset = (i as f32 - half) * spread / half.max(1.0);
        let a = angle + offset;
        bullets.push(Bullet {
            x: px,
            y: py,
            vx: a.cos() * 500.0,
            vy: a.sin() * 500.0,
        });
    }
    bullets
}

fn main() {
    let mut game_over = false;
    let mut wave = 1;

    let mut player_hp: i32 = 10;
    let mut player_flash = 0.0;
    let player_speed = 200.0;
    let (mut px, mut py) = (0.0, 0.0);

    let mut enemy_hp = 1;
    let mut enemies = spawn_wave(px, py, 20, (50.0, 150.0), enemy_hp);

    let mut bullets = Vec::new();
    let mut last_shot = 0.0;
    let mut spread_count = 1;
    let mut fire_rate = 10.0;

    App::init(|ctx| ctx.set_title("Shooter")).run(move |t, g, i| {
        if game_over {
            return;
        }

        let [w, h] = g.screen_size();
        let (mx, my) = i.mouse_position();
        let (cx, cy) = (px - w / 2.0 + mx, py - h / 2.0 + my);

        px += (i.keys_held(&[KeyCode::KeyD, KeyCode::ArrowRight]) as i8
            - i.keys_held(&[KeyCode::KeyA, KeyCode::ArrowLeft]) as i8) as f32
            * player_speed
            * t.delta;
        py += (i.keys_held(&[KeyCode::KeyS, KeyCode::ArrowDown]) as i8
            - i.keys_held(&[KeyCode::KeyW, KeyCode::ArrowUp]) as i8) as f32
            * player_speed
            * t.delta;

        g.camera().target(px, py);

        last_shot -= t.delta;
        if i.mouse_held(MouseButton::Left) && last_shot <= 0.0 {
            bullets.extend(spawn_bullet_spread(px, py, cx, cy, spread_count));
            last_shot = 1.0 / fire_rate;
        }

        for e in &mut enemies {
            let (dx, dy) = (px - e.x, py - e.y);
            let len = (dx * dx + dy * dy).sqrt().max(0.001);
            e.x += dx / len * e.speed * t.delta;
            e.y += dy / len * e.speed * t.delta;
        }

        bullets.retain(|b| {
            let mut hit = false;
            for e in &mut enemies {
                let (dx, dy) = (b.x - e.x, b.y - e.y);
                if (dx * dx + dy * dy).sqrt() < 10.0 {
                    e.hp = e.hp.saturating_sub(1);
                    e.flash = 0.1;
                    hit = true;
                    break;
                }
            }
            let (bx, by) = (b.x - px, b.y - py);
            let out_of_view = bx.abs() > w / 2.0 || by.abs() > h / 2.0;

            !hit && !out_of_view
        });

        enemies.retain(|e| e.hp > 0);

        for b in &mut bullets {
            b.x += b.vx * t.delta;
            b.y += b.vy * t.delta;
            g.rect().at(b.x, b.y).size(5.0, 10.0).color(Color::BLUE);
        }

        for e in &mut enemies {
            let angle = (py - e.y).atan2(px - e.x);
            if ((px - e.x).powi(2) + (py - e.y).powi(2)).sqrt() < 15.0 {
                player_hp = player_hp.saturating_sub(1);
                player_flash = 0.1;
            }
            e.flash = (e.flash - t.delta).max(0.0);
            let color = if e.flash > 0.0 {
                Color::WHITE
            } else {
                Color::RED
            };
            g.tri().at(e.x, e.y).size(20.0).rotation(angle).color(color);
        }

        player_flash = (player_flash - t.delta).max(0.0);
        let player_color = if player_flash > 0.0 {
            Color::WHITE
        } else {
            Color::GREEN
        };

        g.rect()
            .at(px, py)
            .size(20.0, 20.0)
            .rotation((cy - py).atan2(cx - px))
            .color(player_color);

        if player_hp == 0 {
            game_over = true;
        }

        if enemies.is_empty() {
            wave += 1;
            if wave % 3 == 0 {
                enemy_hp *= 2;
            }

            spread_count = (spread_count + 1).min(20);
            fire_rate = (fire_rate + 1.0).min(100.0);

            enemies = spawn_wave(
                px,
                py,
                10 + wave * 5,
                (50.0 + wave as f32 * 2.0, 100.0 + wave as f32 * 2.0),
                enemy_hp,
            );
        }
    });
}
