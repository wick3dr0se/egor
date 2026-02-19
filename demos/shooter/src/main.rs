mod animation;
mod tilemap;

use rand::Rng;

use egor::{
    app::{App, FrameContext, WindowEvent, egui::Window},
    input::{KeyCode, MouseButton},
    math::{Rect, Vec2, vec2},
    render::{Color, OffscreenTarget},
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
}

fn spawn_wave(position: Vec2, count: usize, speed: (f32, f32), hp: f32) -> Vec<Zombie> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            let a = rng.gen_range(0.0..std::f32::consts::TAU);
            let d = rng.gen_range(300.0..800.0);
            let pos = position + vec2(a.cos(), a.sin()) * d;
            Zombie {
                rect: Rect::new(pos, Vec2::splat(PLAYER_SIZE)),
                speed: rng.gen_range(speed.0..speed.1),
                hp,
                flash: 0.0,
            }
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
    let mut state = GameState {
        map: EgorMap::new(include_str!("../assets/map.json")),
        minimap: None,
        minimap_tex: 0,
        player: Soldier {
            rect: Rect::new(Vec2::ZERO, Vec2::splat(PLAYER_SIZE)),
            hp: 100.0,
            flash: 0.0,
        },
        player_anim: SpriteAnim::new(3, 6, 16, 0.2),
        player_tex: 0,
        enemies: spawn_wave(Vec2::ZERO, 5, (50.0, 125.0), 1.0),
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
    };

    App::new().title("Egor Shooter Demo").run(
        move |FrameContext {
                  gfx,
                  input,
                  timer,
                  egui_ctx,
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
                state.map.load_tileset(
                    gfx,
                    include_bytes!("../assets/otsp_tiles_01.png"),
                    "otsp_tiles_01.png",
                );
                state.map.load_tileset(
                    gfx,
                    include_bytes!("../assets/otsp_walls_01.png"),
                    "otsp_walls_01.png",
                );
                state.player_tex = gfx.load_texture(include_bytes!("../assets/soldier.png"));
                state.enemy_tex = gfx.load_texture(include_bytes!("../assets/zombie.png"));
                let mut minimap = gfx.create_offscreen(200, 200);
                state.minimap_tex = gfx.offscreen_as_texture(&mut minimap);
                state.minimap = Some(minimap);
                return;
            }

            let screen_size = gfx.screen_size();

            if state.game_over {
                gfx.text("GAME OVER")
                    .color(Color::RED)
                    .at(vec2(screen_size.x / 2. - 40., screen_size.y / 2.));
                return;
            }
            if let Some(minimap) = &mut state.minimap {
                gfx.render_offscreen(minimap, |gfx| {
                    gfx.clear(Color::BLACK);

                    gfx.camera().set_zoom(0.15);
                    gfx.camera()
                        .center(state.player.rect.center(), vec2(200.0, 200.0));

                    for e in &state.enemies {
                        gfx.rect()
                            .at(e.rect.position)
                            .color(Color::RED)
                            .size(Vec2::splat(48.0));
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
            let screen_half = screen_size / 2.0;
            let position = state.player.rect.position - screen_half
                + Into::<Vec2>::into(input.mouse_position());

            let dx = input.keys_held(&[KeyCode::KeyD, KeyCode::ArrowRight]) as i8
                - input.keys_held(&[KeyCode::KeyA, KeyCode::ArrowLeft]) as i8;
            let dy = input.keys_held(&[KeyCode::KeyS, KeyCode::ArrowDown]) as i8
                - input.keys_held(&[KeyCode::KeyW, KeyCode::ArrowUp]) as i8;
            let moving = dx != 0 || dy != 0;

            state
                .player
                .rect
                .translate(vec2(dx as f32, dy as f32) * 200.0 * timer.delta);

            gfx.camera().center(state.player.rect.position, screen_size);
            gfx.clear(Color::WHITE);
            state.map.render(gfx);

            state.fire_cd -= timer.delta;
            if input.mouse_held(MouseButton::Left) && state.fire_cd <= 0.0 {
                state.bullets.extend(spawn_bullets(
                    state.player.rect.center(),
                    position,
                    state.spread,
                ));
                state.fire_cd = 1.0 / state.fire_rate;
            }

            for e in &mut state.enemies {
                let dir = (state.player.rect.position - e.rect.position).normalize_or_zero();
                e.rect.translate(dir * e.speed * timer.delta);
            }

            state.kills += handle_bullet_hits(
                &mut state.bullets,
                &mut state.enemies,
                state.player.rect.position,
            );

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
            let dir = position - state.player.rect.position;
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
                    state.player.rect.position,
                    (state.wave + 2) * 3,
                    (
                        50. + state.wave as f32 * 3.0,
                        125. + state.wave as f32 * 3.0,
                    ),
                    state.hp,
                );
            }

            if state.minimap.is_some() {
                let screen_pos = vec2(screen_size.x - 210.0, 10.0);
                let world_pos = gfx.camera().screen_to_world(screen_pos);

                gfx.rect()
                    .at(world_pos)
                    .size(vec2(200.0, 200.0))
                    .texture(state.minimap_tex);
            }

            Window::new("Debug").show(egui_ctx, |ui| {
                ui.label(format!("FPS: {}", timer.fps));
                ui.label(format!("Wave: {}", state.wave));
                ui.label(format!("Zombies killed: {}", state.kills));
                ui.label(format!("HP: {:.0}", state.player.hp));
                ui.label(format!("Fire rate: {:.1}/s", state.fire_rate));
                ui.label(format!("Bullet Spread: {}", state.spread));
            });
        },
    );
}
