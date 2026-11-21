mod animation;
mod tilemap;

use rand::Rng;

use egor::{
    app::{App, FrameContext, WindowEvent},
    input::{KeyCode, MouseButton},
    math::{Rect, Vec2, vec2},
    render::{Align, Color, OffscreenTarget},
};

use crate::{animation::SpriteAnim, tilemap::EgorMap};

const PLAYER_SIZE: f32 = 64.0;
const BULLET_SIZE: Vec2 = vec2(5.0, 10.0);

struct Bullet {
    rect: Rect,
    vel: Vec2,
}

struct Zombie {
    rect: Rect,
    speed: f32,
    hp: f32,
    flash: f32,
}

struct Soldier {
    rect: Rect,
    hp: f32,
    flash: f32,
}

struct GameState {
    map: EgorMap,
    minimap: Option<OffscreenTarget>,
    minimap_tex: usize,
    player: Soldier,
    player_anim: SpriteAnim,
    player_tex: usize,
    enemies: Vec<Zombie>,
    enemy_anim: SpriteAnim,
    enemy_tex: usize,
    bullets: Vec<Bullet>,
    wave: usize,
    kills: usize,
    hp: f32,
    fire_cd: f32,
    fire_rate: f32,
    spread: usize,
    game_over: bool,
    ticker: f32,
    spin: f32,
    look: Vec2,
}

fn spawn_wave(
    map: &EgorMap,
    position: Vec2,
    count: usize,
    speed: (f32, f32),
    hp: f32,
) -> Vec<Zombie> {
    let mut rng = rand::thread_rng();
    let half = PLAYER_SIZE * 0.5;

    (0..count)
        .filter_map(|_| {
            // check the whole body, not just the center point
            let pos = (0..40).find_map(|_| {
                let a = rng.gen_range(0.0..std::f32::consts::TAU);
                let d = rng.gen_range(220.0..520.0);
                let pos = position + vec2(a.cos(), a.sin()) * d;
                (!map.overlaps_wall(pos, half - 0.01)).then_some(pos)
            })?;

            Some(Zombie {
                rect: Rect::new(pos - Vec2::splat(half), Vec2::splat(PLAYER_SIZE)),
                speed: rng.gen_range(speed.0..speed.1),
                hp,
                flash: 0.0,
            })
        })
        .collect()
}

fn spawn_bullets(position: Vec2, target: Vec2, count: usize) -> Vec<Bullet> {
    let angle = (target - position).y.atan2((target - position).x);
    let spread = 0.3;
    let half = (count as f32 - 1.0) / 2.0;

    (0..count)
        .map(|i| {
            let offset = (i as f32 - half) * spread / half.max(1.0);
            let a = angle + offset;
            Bullet {
                rect: Rect::new(position - BULLET_SIZE / 2.0, BULLET_SIZE),
                vel: vec2(a.cos(), a.sin()) * 500.0,
            }
        })
        .collect()
}

fn handle_bullet_hits(bullets: &mut Vec<Bullet>, enemies: &mut Vec<Zombie>, player: Vec2) -> usize {
    let mut kills = 0;
    bullets.retain(|b| {
        for e in enemies.iter_mut() {
            if e.rect.contains(b.rect.position) {
                e.hp -= 1.0;
                e.flash = 0.1;
                return false;
            }
        }
        let offscreen = (b.rect.position - player).length() > 2000.0;
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
    let map = EgorMap::new(include_str!("../assets/map.json"));
    let start = map.spawn("player").unwrap_or_else(|| map.tile_px() * 24.0);
    let enemies = spawn_wave(&map, start, 7, (90.0, 170.0), 1.0);

    let mut state = GameState {
        map,
        minimap: None,
        minimap_tex: 0,
        player: Soldier {
            rect: Rect::new(
                start - Vec2::splat(PLAYER_SIZE * 0.5),
                Vec2::splat(PLAYER_SIZE),
            ),
            hp: 100.0,
            flash: 0.0,
        },
        player_anim: SpriteAnim::new(3, 6, 16, 0.2),
        player_tex: 0,
        enemies,
        enemy_anim: SpriteAnim::new(2, 6, 11, 0.2),
        enemy_tex: 0,
        bullets: vec![],
        wave: 1,
        kills: 0,
        hp: 1.0,
        fire_cd: 0.0,
        fire_rate: 2.0,
        spread: 1,
        game_over: false,
        ticker: 0.0,
        spin: 0.0,
        look: Vec2::ZERO,
    };

    App::new().title("Egor Shooter Demo").run(
        move |FrameContext {
                  gfx,
                  input,
                  timer,
                  events,
                  ..
              }| {
            for event in events {
                if event == &WindowEvent::CloseRequested {
                    println!("Quitting already? Don't be a sore loser");
                    println!("Final Wave: {}", state.wave);
                    println!("Killed {} zombies", state.kills);
                    state.game_over = true;
                }
            }

            if timer.frame == 0 {
                state.map.load_tileset(gfx, include_bytes!("../assets/tiles.png"), "tiles.png");
                state.player_tex = gfx.load_texture(include_bytes!("../assets/soldier.png"));
                state.enemy_tex = gfx.load_texture(include_bytes!("../assets/zombie.png"));
                let mut minimap = gfx.create_offscreen(200, 200);
                state.minimap_tex = gfx.offscreen_as_texture(&mut minimap);
                state.minimap = Some(minimap);
                return;
            }

            let screen_size = gfx.screen_size();

            // Broadcast style HUD: a clipped ticker, so long text stays in its bar
            state.ticker += timer.delta * 90.0;
            let bar = Rect::new(vec2(0.0, screen_size.y - 52.0), vec2(screen_size.x, 52.0));
            gfx.rect()
                .at(bar.position)
                .size(bar.size)
                .color(Color::new([0.05, 0.06, 0.09, 0.92]))
                .layer(50)
                .screen();

            let line = format!(
                "WAVE {}      KILLS {}      HP {:.0}      FIRE {:.1}/s      SPREAD {}      FPS {}      WASD MOVE      MOUSE AIM      CLICK FIRE      ",
                state.wave, state.kills, state.player.hp, state.fire_rate, state.spread, timer.fps
            );
            let span = 1300.0;
            let offset = state.ticker % span;
            gfx.with_clip(bar, |gfx| {
                for i in 0..2 {
                    gfx.text(&line)
                        .at(vec2(i as f32 * span - offset, bar.position.y + 17.0))
                        .size(18.0)
                        .color(Color::new([0.72, 0.83, 1.0, 1.0]))
                        .layer(51)
                        .screen();
                }
            });

            if state.game_over {
                gfx.text("GAME OVER")
                    .layer(100)
                    .color(Color::RED)
                    .size(32.0)
                    .bold()
                    .in_rect(Rect::new(Vec2::ZERO, screen_size), Align::MiddleCenter);
                return;
            }
            if let Some(minimap) = &mut state.minimap {
                gfx.render_offscreen(minimap, |gfx| {
                    gfx.clear(Color::BLACK);

                    // fit the whole map into 200px
                    let (mw, mh) = state.map.dimensions();
                    let (tw, th) = state.map.tile_size();
                    gfx.camera().set_zoom(200.0 / (mw as f32 * tw).max(mh as f32 * th));
                    gfx.camera().center(
                        vec2(mw as f32 * tw, mh as f32 * th) * 0.5,
                        vec2(200.0, 200.0),
                    );
                    state.map.render(gfx);

                    for e in &state.enemies {
                        gfx.rect()
                            .at(e.rect.center() - Vec2::splat(48.0))
                            .color(Color::RED)
                            .size(Vec2::splat(96.0));
                    }

                    gfx.rect()
                        .at(state.player.rect.position)
                        .color(Color::GREEN)
                        .texture(41);

                    for b in &state.bullets {
                        gfx.rect()
                            .at(b.rect.position)
                            .size(Vec2::splat(16.0))
                            .color(Color::WHITE);
                    }
                });
            }

            let dx = input.keys_held(&[KeyCode::KeyD, KeyCode::ArrowRight]) as i8
                - input.keys_held(&[KeyCode::KeyA, KeyCode::ArrowLeft]) as i8;
            let dy = input.keys_held(&[KeyCode::KeyS, KeyCode::ArrowDown]) as i8
                - input.keys_held(&[KeyCode::KeyW, KeyCode::ArrowUp]) as i8;
            let moving = dx != 0 || dy != 0;

            let half = PLAYER_SIZE * 0.5;
            let want = vec2(dx as f32, dy as f32).normalize_or_zero() * 200.0 * timer.delta;
            let center = state.map.slide(state.player.rect.center(), half, want);
            state.player.rect = Rect::new(center - Vec2::splat(half), Vec2::splat(PLAYER_SIZE));

            let cursor: Vec2 = input.mouse_position().into();
            gfx.camera().set_zoom(1.6);

            // shift the camera toward the cursor for a look ahead
            let lead = ((cursor - screen_size * 0.5) * 0.14).clamp_length_max(80.0);
            state.look += (lead - state.look) * (1.0 - (-6.0 * timer.delta).exp());
            gfx.camera()
                .center(state.player.rect.center() + state.look, screen_size);
            // aim with the rotation currently on screen
            gfx.camera().set_rotation(state.spin);

            // cursor to world, with the rotation applied
            let aim = gfx.camera().screen_to_world(cursor);
            let aim_dir = (aim - state.player.rect.center()).normalize_or_zero();

            // cursor position drives the lean, aim depends on rotation and would feed back
            let lean = (cursor.x / screen_size.x - 0.5) * 0.20;
            state.spin += (lean - state.spin) * (1.0 - (-8.0 * timer.delta).exp());
            gfx.camera().set_rotation(state.spin);

            gfx.clear(Color::new([0.06, 0.07, 0.09, 1.0]));
            state.map.render(gfx);

            // World text follows the camera and stays a constant pixel size, anchored
            // bottom-center so it sits above the player rather than starting at them.
            gfx.text("player")
                .at(state.player.rect.center() - vec2(0.0, 20.0))
                .anchor(Align::BottomCenter)
                .size(14.0)
                .color(Color::BLACK)
                .layer(10)
                .world();

            state.fire_cd -= timer.delta;
            if input.mouse_held(MouseButton::Left) && state.fire_cd <= 0.0 {
                state.bullets.extend(spawn_bullets(
                    state.player.rect.center(),
                    aim,
                    state.spread,
                ));
                state.fire_cd = 1.0 / state.fire_rate;
                state.spin -= aim_dir.x * 0.02;
            }

            for e in &mut state.enemies {
                let dir = (state.player.rect.center() - e.rect.center()).normalize_or_zero();
                let half = PLAYER_SIZE * 0.5;
                let moved = state.map.slide(e.rect.center(), half, dir * e.speed * timer.delta);
                e.rect = Rect::new(moved - Vec2::splat(half), Vec2::splat(PLAYER_SIZE));
            }

            state.kills += handle_bullet_hits(
                &mut state.bullets,
                &mut state.enemies,
                state.player.rect.position,
            );

            state.bullets.retain(|b| !state.map.solid_at(b.rect.center()));
            for b in &mut state.bullets {
                b.rect.translate(b.vel * timer.delta);
                let angle = b.vel.y.atan2(b.vel.x);
                gfx.rect().with(&b.rect).rotate(angle).color(Color::BLUE);
            }

            state.enemy_anim.update(timer.delta);
            for e in &mut state.enemies {
                let dir = state.player.rect.position - e.rect.position;
                let angle = dir.y.atan2(dir.x);

                if dir.length() < 15.0 {
                    state.player.hp -= 1.0;
                    state.player.flash = 0.1;
                }

                e.flash = (e.flash - timer.delta).max(0.0);
                gfx.rect()
                    .with(&e.rect)
                    .rotate(angle)
                    .color(if e.flash > 0.0 {
                        Color::RED
                    } else {
                        Color::WHITE
                    })
                    .texture(state.enemy_tex)
                    .uv(state.enemy_anim.uv());
            }

            if state.player.hp <= 0.0 {
                state.game_over = true;
            }

            state.player.flash = (state.player.flash - timer.delta).max(0.0);
            let dir = aim - state.player.rect.position;
            let angle = dir.y.atan2(dir.x);

            let uv = if moving {
                state.player_anim.update(timer.delta);
                state.player_anim.uv()
            } else {
                state.player_anim.frame_uv(0)
            };

            gfx.rect()
                .with(&state.player.rect)
                .rotate(angle)
                .color(if state.player.flash > 0.0 {
                    Color::RED
                } else {
                    Color::WHITE
                })
                .texture(state.player_tex)
                .uv(uv);

            if state.enemies.is_empty() {
                state.wave += 1;
                if state.wave.is_multiple_of(3) {
                    state.hp *= 1.1;
                    state.spread = (state.spread + 1).min(20);
                }
                state.fire_rate += 0.1;
                state.enemies = spawn_wave(
                    &state.map,
                    state.player.rect.center(),
                    (state.wave + 2) * 3,
                    (
                        90. + state.wave as f32 * 3.0,
                        170. + state.wave as f32 * 3.0,
                    ),
                    state.hp,
                );
            }

            if state.minimap.is_some() {
                gfx.rect()
                    .at(vec2(screen_size.x - 210.0, 10.0))
                    .size(vec2(200.0, 200.0))
                    .texture(state.minimap_tex)
                    .screen();
            }

        },
    );
}
