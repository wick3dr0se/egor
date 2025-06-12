mod animation;

use egor::{
    app::{App, Context},
    input::{KeyCode, MouseButton},
    render::Color,
};
use rand::{Rng, RngCore};

use crate::animation::SpriteAnim;

struct Bullet {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}
struct Zombie {
    x: f32,
    y: f32,
    speed: f32,
    hp: f32,
    flash: f32,
}
struct Soldier {
    x: f32,
    y: f32,
    hp: f32,
    flash: f32,
}

fn spawn_wave(cx: f32, cy: f32, count: usize, speed: (f32, f32), hp: f32) -> Vec<Zombie> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            let a = rng.gen_range(0.0..std::f32::consts::TAU);
            let d = rng.gen_range(300.0..800.0);
            Zombie {
                x: cx + a.cos() * d,
                y: cy + a.sin() * d,
                speed: rng.gen_range(speed.0..speed.1),
                hp,
                flash: 0.0,
            }
        })
        .collect()
}

fn spawn_bullets(px: f32, py: f32, tx: f32, ty: f32, count: usize) -> Vec<Bullet> {
    let angle = (ty - py).atan2(tx - px);
    let spread = 0.3;
    let half = (count as f32 - 1.0) / 2.0;
    (0..count)
        .map(|i| {
            let offset = (i as f32 - half) * spread / half.max(1.0);
            let a = angle + offset;
            Bullet {
                x: px,
                y: py,
                vx: a.cos() * 500.0,
                vy: a.sin() * 500.0,
            }
        })
        .collect()
}

fn recolor_image(im: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) {
    let mut rgb = [0u8; 3];
    rand::thread_rng().fill_bytes(&mut rgb);
    for p in im.pixels_mut() {
        p.0[0] = rgb[0];
        p.0[1] = rgb[1];
        p.0[2] = rgb[2];
    }
}

fn main() {
    let mut game_over = false;
    let mut wave = 1;
    let mut hp = 1.0;
    let mut fire_cd = 0.0;
    let mut fire_rate = 2.0;
    let mut spread = 1;
    let mut kills = 0;

    let mut player = Soldier {
        x: 0.0,
        y: 0.0,
        hp: 100.0,
        flash: 0.0,
    };
    let mut player_anim = SpriteAnim::new(1, 17, 17, 0.2);
    let mut enemies = spawn_wave(0.0, 0.0, 5, (50.0, 125.0), hp);
    let mut enemy_anim = SpriteAnim::new(1, 11, 11, 0.2);
    let mut bullets = vec![];

    let mut zombie_image = image::load_from_memory(include_bytes!("../assets/zombie.png"))
        .unwrap()
        .to_rgba8();
    let mut time_since_recolor = 0.;

    App::init((), |ctx| {
        ctx.set_title("Egor Shooter Demo");
        ctx.load_texture(include_bytes!("../assets/soldier.png"));
        ctx.load_texture(include_bytes!("../assets/zombie.png"));
    })
    .plugin(move |ctx: &mut Context<()>| {
        let [w, h] = ctx.graphics.screen_size();

        if game_over {
            ctx.graphics
                .text("GAME OVER")
                .color(Color::RED)
                .at(w / 2. - 40., h / 2.);
            return;
        }

        ctx.graphics.clear(Color::WHITE);

        let (mx, my) = ctx.input.mouse_position();
        let (cx, cy) = (player.x - w / 2. + mx, player.y - h / 2. + my);

        let dx = ctx.input.keys_held(&[KeyCode::KeyD, KeyCode::ArrowRight]) as i8
            - ctx.input.keys_held(&[KeyCode::KeyA, KeyCode::ArrowLeft]) as i8;
        let dy = ctx.input.keys_held(&[KeyCode::KeyS, KeyCode::ArrowDown]) as i8
            - ctx.input.keys_held(&[KeyCode::KeyW, KeyCode::ArrowUp]) as i8;
        let moving = dx != 0 || dy != 0;

        player.x += dx as f32 * 200.0 * ctx.timer.delta;
        player.y += dy as f32 * 200.0 * ctx.timer.delta;
        ctx.graphics.camera().target(player.x, player.y);

        fire_cd -= ctx.timer.delta;
        if ctx.input.mouse_held(MouseButton::Left) && fire_cd <= 0.0 {
            bullets.extend(spawn_bullets(player.x, player.y, cx, cy, spread));
            fire_cd = 1.0 / fire_rate;
        }

        for e in &mut enemies {
            let (dx, dy) = (player.x - e.x, player.y - e.y);
            let dist = (dx * dx + dy * dy).sqrt().max(0.001);
            e.x += dx / dist * e.speed * ctx.timer.delta;
            e.y += dy / dist * e.speed * ctx.timer.delta;
        }

        bullets.retain(|b| {
            let mut hit = false;
            for e in &mut enemies {
                let (dx, dy) = (b.x - e.x, b.y - e.y);
                if (dx * dx + dy * dy).sqrt() < 10.0 {
                    e.hp -= 1.0;
                    e.flash = 0.1;
                    hit = true;
                    break;
                }
            }
            let offscreen = (b.x - player.x).abs() > w / 2. || (b.y - player.y).abs() > h / 2.;
            !hit && !offscreen
        });

        enemies.retain(|e| {
            if e.hp <= 0.0 {
                kills += 1;
                false
            } else {
                true
            }
        });

        for b in &mut bullets {
            b.x += b.vx * ctx.timer.delta;
            b.y += b.vy * ctx.timer.delta;
            ctx.graphics
                .rect()
                .at(b.x, b.y)
                .size(5., 10.)
                .color(Color::BLUE);
        }
        time_since_recolor += ctx.timer.delta;
        if time_since_recolor > 1. {
            time_since_recolor = 0.;
            recolor_image(&mut zombie_image);

            ctx.graphics.update_texture_raw(
                1,
                zombie_image.width(),
                zombie_image.height(),
                &zombie_image,
            );
        }

        enemy_anim.update(ctx.timer.delta);
        for e in &mut enemies {
            let angle = (player.y - e.y).atan2(player.x - e.x);
            if ((player.x - e.x).powi(2) + (player.y - e.y).powi(2)).sqrt() < 15.0 {
                player.hp -= 1.0;
                player.flash = 0.1;
            }
            e.flash = (e.flash - ctx.timer.delta).max(0.0);
            ctx.graphics
                .rect()
                .at(e.x, e.y)
                .size(64., 64.)
                .rotation(angle + std::f32::consts::FRAC_PI_2)
                .color(if e.flash > 0.0 {
                    Color::RED
                } else {
                    Color::WHITE
                })
                .texture(1)
                .uv(enemy_anim.uv());
        }

        if player.hp <= 0.0 {
            game_over = true;
        }

        player.flash = (player.flash - ctx.timer.delta).max(0.0);
        let rot = (cy - player.y).atan2(cx - player.x) + std::f32::consts::FRAC_PI_2;
        let uv = if moving {
            player_anim.update(ctx.timer.delta);
            player_anim.uv()
        } else {
            player_anim.frame_uv(0)
        };

        ctx.graphics
            .rect()
            .at(player.x, player.y)
            .size(64., 64.)
            .rotation(rot)
            .color(if player.flash > 0.0 {
                Color::RED
            } else {
                Color::WHITE
            })
            .texture(0)
            .uv(uv);

        if enemies.is_empty() {
            wave += 1;
            if wave % 3 == 0 {
                hp *= 1.1;
                spread = (spread + 1).min(20);
            }
            fire_rate += 0.1;
            enemies = spawn_wave(
                player.x,
                player.y,
                (wave + 2) * 3,
                (50. + wave as f32 * 3.0, 125. + wave as f32 * 3.0),
                hp,
            );
        }

        ctx.graphics.text(&format!("Wave: {wave}")).at(10.0, 10.0);
        ctx.graphics
            .text(&format!("Zombies killed: {kills}"))
            .at(10.0, 30.0);
        ctx.graphics
            .text(&format!("HP: {:.0}", player.hp))
            .at(10.0, 50.0);
        ctx.graphics
            .text(&format!("Fire rate: {:.1}/s", fire_rate))
            .at(10.0, 70.0);
        ctx.graphics
            .text(&format!("Bullet Spread: {spread}"))
            .at(10.0, 90.0);
    })
    .run();
}
