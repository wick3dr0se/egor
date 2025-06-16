mod animation;

use crate::animation::SpriteAnim;
use egor::{
    app::{App, Context},
    input::{KeyCode, MouseButton},
    math::{Vec2, vec2},
    render::Color,
};
use rand::{Rng, RngCore};

const PLAYER_SIZE: f32 = 64.0;
const BULLET_RADIUS: f32 = 5.0;
const BULLET_LENGTH: f32 = 10.0;
const HIT_RADIUS: f32 = 10.0;

struct Bullet {
    pos: Vec2,
    vel: Vec2,
}
struct Zombie {
    pos: Vec2,
    speed: f32,
    hp: f32,
    flash: f32,
}
struct Soldier {
    pos: Vec2,
    hp: f32,
    flash: f32,
}

struct GameState {
    player: Soldier,
    player_anim: SpriteAnim,
    enemies: Vec<Zombie>,
    enemy_anim: SpriteAnim,
    bullets: Vec<Bullet>,
    wave: usize,
    kills: usize,
    hp: f32,
    fire_cd: f32,
    fire_rate: f32,
    spread: usize,
    game_over: bool,
    zombie_image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    time_since_recolor: f32,
}

fn spawn_wave(origin: Vec2, count: usize, speed: (f32, f32), hp: f32) -> Vec<Zombie> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            let a = rng.gen_range(0.0..std::f32::consts::TAU);
            let d = rng.gen_range(300.0..800.0);
            Zombie {
                pos: origin + vec2(a.cos(), a.sin()) * d,
                speed: rng.gen_range(speed.0..speed.1),
                hp,
                flash: 0.0,
            }
        })
        .collect()
}

fn spawn_bullets(origin: Vec2, target: Vec2, count: usize) -> Vec<Bullet> {
    let angle = (target - origin).y.atan2((target - origin).x);
    let spread = 0.3;
    let half = (count as f32 - 1.0) / 2.0;

    (0..count)
        .map(|i| {
            let offset = (i as f32 - half) * spread / half.max(1.0);
            let a = angle + offset;
            Bullet {
                pos: origin,
                vel: vec2(a.cos(), a.sin()) * 500.0,
            }
        })
        .collect()
}

fn recolor_image(im: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) {
    let mut rgb = [0u8; 3];
    rand::thread_rng().fill_bytes(&mut rgb);
    for p in im.pixels_mut() {
        p.0[..3].copy_from_slice(&rgb);
    }
}

fn handle_bullet_hits(bullets: &mut Vec<Bullet>, enemies: &mut Vec<Zombie>, player: Vec2) -> usize {
    let mut kills = 0;
    bullets.retain(|b| {
        for e in enemies.iter_mut() {
            if b.pos.distance(e.pos) < HIT_RADIUS {
                e.hp -= 1.0;
                e.flash = 0.1;
                return false;
            }
        }
        let offscreen = (b.pos - player).length() > 2000.0;
        !offscreen
    });

    enemies.retain(|e| {
        if e.hp <= 0.0 {
            kills += 1;
            false
        } else {
            true
        }
    });

    kills
}

fn main() {
    let state = GameState {
        player: Soldier {
            pos: Vec2::ZERO,
            hp: 100.0,
            flash: 0.0,
        },
        player_anim: SpriteAnim::new(1, 17, 17, 0.2),
        enemies: spawn_wave(Vec2::ZERO, 5, (50.0, 125.0), 1.0),
        enemy_anim: SpriteAnim::new(1, 11, 11, 0.2),
        bullets: vec![],
        wave: 1,
        kills: 0,
        hp: 1.0,
        fire_cd: 0.0,
        fire_rate: 2.0,
        spread: 1,
        game_over: false,
        zombie_image: image::load_from_memory(include_bytes!("../assets/zombie.png"))
            .unwrap()
            .to_rgba8(),
        time_since_recolor: 0.0,
    };

    App::init(state, |_, ctx| {
        ctx.set_title("Egor Shooter Demo");
        ctx.load_texture(include_bytes!("../assets/soldier.png"));
        ctx.load_texture(include_bytes!("../assets/zombie.png"));
    })
    .run(move |state, ctx: &mut Context| {
        let screen_size = ctx.graphics.screen_size();
        let screen_half = screen_size / 2.0;

        if state.game_over {
            ctx.graphics
                .text("GAME OVER")
                .color(Color::RED)
                .at(screen_size.x / 2. - 40., screen_size.y / 2.);
            return;
        }

        ctx.graphics.clear(Color::WHITE);

        let origin =
            state.player.pos - screen_half + Into::<Vec2>::into(ctx.input.mouse_position());

        let dx = ctx.input.keys_held(&[KeyCode::KeyD, KeyCode::ArrowRight]) as i8
            - ctx.input.keys_held(&[KeyCode::KeyA, KeyCode::ArrowLeft]) as i8;
        let dy = ctx.input.keys_held(&[KeyCode::KeyS, KeyCode::ArrowDown]) as i8
            - ctx.input.keys_held(&[KeyCode::KeyW, KeyCode::ArrowUp]) as i8;
        let moving = dx != 0 || dy != 0;

        state.player.pos += vec2(dx as f32, dy as f32) * 200.0 * ctx.timer.delta;
        ctx.graphics.camera().target(state.player.pos);

        state.fire_cd -= ctx.timer.delta;
        if ctx.input.mouse_held(MouseButton::Left) && state.fire_cd <= 0.0 {
            state
                .bullets
                .extend(spawn_bullets(state.player.pos, origin, state.spread));
            state.fire_cd = 1.0 / state.fire_rate;
        }

        for e in &mut state.enemies {
            let dir = (state.player.pos - e.pos).normalize_or_zero();
            e.pos += dir * e.speed * ctx.timer.delta;
        }

        state.kills += handle_bullet_hits(&mut state.bullets, &mut state.enemies, state.player.pos);

        for b in &mut state.bullets {
            b.pos += b.vel * ctx.timer.delta;
            ctx.graphics
                .rect()
                .at(b.pos)
                .size(BULLET_RADIUS, BULLET_LENGTH)
                .color(Color::BLUE);
        }

        state.time_since_recolor += ctx.timer.delta;
        if state.time_since_recolor > 1.0 {
            state.time_since_recolor = 0.0;
            recolor_image(&mut state.zombie_image);
            ctx.graphics.update_texture_raw(
                1,
                state.zombie_image.width(),
                state.zombie_image.height(),
                &state.zombie_image,
            );
        }

        state.enemy_anim.update(ctx.timer.delta);
        for e in &mut state.enemies {
            let dir = state.player.pos - e.pos;
            let angle = dir.y.atan2(dir.x);

            if dir.length() < 15.0 {
                state.player.hp -= 1.0;
                state.player.flash = 0.1;
            }

            e.flash = (e.flash - ctx.timer.delta).max(0.0);
            ctx.graphics
                .rect()
                .at(e.pos)
                .size(PLAYER_SIZE, PLAYER_SIZE)
                .rotation(angle + std::f32::consts::FRAC_PI_2)
                .color(if e.flash > 0.0 {
                    Color::RED
                } else {
                    Color::WHITE
                })
                .texture(1)
                .uv(state.enemy_anim.uv());
        }

        if state.player.hp <= 0.0 {
            state.game_over = true;
        }

        state.player.flash = (state.player.flash - ctx.timer.delta).max(0.0);
        let dir = origin - state.player.pos;
        let rot = dir.y.atan2(dir.x) + std::f32::consts::FRAC_PI_2;

        let uv = if moving {
            state.player_anim.update(ctx.timer.delta);
            state.player_anim.uv()
        } else {
            state.player_anim.frame_uv(0)
        };

        ctx.graphics
            .rect()
            .at(state.player.pos)
            .size(PLAYER_SIZE, PLAYER_SIZE)
            .rotation(rot)
            .color(if state.player.flash > 0.0 {
                Color::RED
            } else {
                Color::WHITE
            })
            .texture(0)
            .uv(uv);

        if state.enemies.is_empty() {
            state.wave += 1;
            if state.wave % 3 == 0 {
                state.hp *= 1.1;
                state.spread = (state.spread + 1).min(20);
            }
            state.fire_rate += 0.1;
            state.enemies = spawn_wave(
                state.player.pos,
                (state.wave + 2) * 3,
                (
                    50. + state.wave as f32 * 3.0,
                    125. + state.wave as f32 * 3.0,
                ),
                state.hp,
            );
        }

        ctx.graphics
            .text(&format!("Wave: {}", state.wave))
            .at(10.0, 10.0);
        ctx.graphics
            .text(&format!("Zombies killed: {}", state.kills))
            .at(10.0, 30.0);
        ctx.graphics
            .text(&format!("HP: {:.0}", state.player.hp))
            .at(10.0, 50.0);
        ctx.graphics
            .text(&format!("Fire rate: {:.1}/s", state.fire_rate))
            .at(10.0, 70.0);
        ctx.graphics
            .text(&format!("Bullet Spread: {}", state.spread))
            .at(10.0, 90.0);
    });
}
