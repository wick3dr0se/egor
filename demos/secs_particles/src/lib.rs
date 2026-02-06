mod lightning;

use egor::{
    app::{App, FrameContext},
    math::{Vec2, vec2},
    render::Color,
};
use rand::Rng;
use secs::World;
use std::f32::consts::TAU;

const MAX_PARTICLES: usize = 29_999;

enum ParticleType {
    Fire,
    Ice,
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    size: f32,
    color: [f32; 3],
    particle_type: ParticleType,
}

struct Fireball {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    size: f32,
    trail_timer: f32,
    depth: u8,
}

struct IceCube {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    size: f32,
    rotation: f32,
    rot_vel: f32,
    depth: u8,
}

// Android expects `android_main()` entry point to exist here in lib.rs.
// This macro generates the entry point for Android targets and nothing for other targets
// It expects a `main()` function and it needs to be public, so it can be called outside
// Calling this in main.rs, gives us a way to run the same exact code on desktop, wasm & Android
egor::main!(main);
pub fn main() {
    let world = World::default();
    let mut rng = rand::thread_rng();
    let mut spawn_timer = 0.0;
    let mut shake = Vec2::ZERO;

    App::new()
        .title("Egor ECS Particles Demo")
        .vsync(false)
        .run(move |FrameContext { gfx, timer, .. }| {
            let screen = gfx.screen_size();
            shake *= 0.88;
            gfx.camera().center(shake, screen);

            spawn_timer += timer.delta;
            if spawn_timer > 0.4 {
                spawn_timer = 0.0;
                let mut rand_pos = || {
                    vec2(
                        rng.gen_range(-screen.x * 0.45..screen.x * 0.45),
                        rng.gen_range(-screen.y * 0.45..screen.y * 0.45),
                    )
                };
                let (a, b) = (rand_pos(), rand_pos());
                let dir = (b - a).normalize_or_zero();

                match rng.gen_range(0..3) {
                    0 => {
                        world.spawn((Fireball {
                            pos: a,
                            vel: dir * rng.gen_range(200.0..300.0),
                            life: rng.gen_range(1.2..1.8),
                            size: rng.gen_range(20.0..30.0),
                            trail_timer: 0.0,
                            depth: 0,
                        },));
                    }
                    1 => {
                        world.spawn((IceCube {
                            pos: b,
                            vel: dir * rng.gen_range(220.0..320.0),
                            life: rng.gen_range(1.0..1.6),
                            size: rng.gen_range(18.0..28.0),
                            rotation: rng.gen_range(0.0..TAU),
                            rot_vel: rng.gen_range(-8.0..8.0),
                            depth: 0,
                        },));
                    }
                    _ => {
                        lightning::spawn(&world, &mut rng, a, b, 0, 0);
                        shake += vec2(rng.gen_range(-6.0..6.0), rng.gen_range(-6.0..6.0));
                    }
                }
            }

            let mut particle_count = 0;
            world.query(|_, _: &Particle| particle_count += 1);

            let mut particle_spawns = Vec::new();
            let mut fireball_spawns = Vec::new();
            let mut ice_spawns = Vec::new();

            world.query(|e, f: &mut Fireball| {
                f.life -= timer.delta;
                if f.life <= 0.0 {
                    let base = if f.depth == 0 { 60 } else { 30 };
                    for _ in 0..base.min(MAX_PARTICLES - particle_count) {
                        let a = rng.gen_range(0.0..TAU);
                        particle_spawns.push(Particle {
                            pos: f.pos,
                            vel: vec2(a.cos(), a.sin()) * rng.gen_range(100.0..400.0),
                            life: rng.gen_range(0.5..1.4),
                            max_life: 1.4,
                            size: rng.gen_range(5.0..16.0),
                            color: [1.0, rng.gen_range(0.3..0.9), rng.gen_range(0.0..0.2)],
                            particle_type: ParticleType::Fire,
                        });
                    }

                    if f.depth < 5 {
                        let split_count = rng.gen_range(1..5) as f32 * 3.0;
                        let base_ang = f.vel.y.atan2(f.vel.x);
                        for i in 0..split_count as u16 {
                            let spread = (i as f32 - split_count * 0.5) * 0.4;
                            let ang = base_ang + spread + rng.gen_range(-0.2..0.2);
                            fireball_spawns.push(Fireball {
                                pos: f.pos,
                                vel: vec2(ang.cos(), ang.sin()) * rng.gen_range(180.0..280.0),
                                life: rng.gen_range(0.7..1.2),
                                size: f.size * 0.65,
                                trail_timer: 0.0,
                                depth: f.depth + 3,
                            });
                        }
                    }

                    shake += vec2(rng.gen_range(-18.0..18.0), rng.gen_range(-18.0..18.0));
                    world.despawn(e);
                    return;
                }

                f.pos += f.vel * timer.delta;
                f.trail_timer += timer.delta;
                if f.trail_timer > 0.02 && particle_count < MAX_PARTICLES {
                    f.trail_timer = 0.0;
                    particle_spawns.push(Particle {
                        pos: f.pos + vec2(rng.gen_range(-4.0..4.0), rng.gen_range(-4.0..4.0)),
                        vel: -f.vel * 0.3
                            + vec2(rng.gen_range(-40.0..40.0), rng.gen_range(-40.0..40.0)),
                        life: 0.5,
                        max_life: 0.5,
                        size: f.size * 0.5,
                        color: [1.0, rng.gen_range(0.4..0.7), 0.0],
                        particle_type: ParticleType::Fire,
                    });
                }

                gfx.polygon()
                    .segments(8)
                    .at(f.pos)
                    .radius(f.size * 1.4)
                    .color(Color::new([1.0, 0.3, 0.0, 0.3]));
                gfx.polygon()
                    .segments(6)
                    .at(f.pos)
                    .radius(f.size)
                    .color(Color::new([1.0, 0.6, 0.1, 1.0]));
                gfx.polygon()
                    .segments(6)
                    .at(f.pos)
                    .radius(f.size * 0.5)
                    .color(Color::new([1.0, 0.9, 0.7, 1.0]));
            });

            world.query(|e, ice: &mut IceCube| {
                ice.life -= timer.delta;
                if ice.life <= 0.0 {
                    let base = if ice.depth == 0 { 50 } else { 25 };
                    for _ in 0..base.min(MAX_PARTICLES - particle_count) {
                        let a = rng.gen_range(0.0..TAU);
                        particle_spawns.push(Particle {
                            pos: ice.pos,
                            vel: vec2(a.cos(), a.sin()) * rng.gen_range(80.0..300.0),
                            life: rng.gen_range(0.6..1.3),
                            max_life: 1.3,
                            size: rng.gen_range(4.0..12.0),
                            color: [rng.gen_range(0.6..0.9), rng.gen_range(0.8..1.0), 1.0],
                            particle_type: ParticleType::Ice,
                        });
                    }

                    if ice.depth < 6 {
                        let split_count = if ice.depth == 0 {
                            rng.gen_range(3..5)
                        } else {
                            rng.gen_range(2..4)
                        };
                        for _ in 0..split_count {
                            let a = rng.gen_range(0.0..TAU);
                            ice_spawns.push(IceCube {
                                pos: ice.pos,
                                vel: vec2(a.cos(), a.sin()) * rng.gen_range(150.0..250.0),
                                life: rng.gen_range(0.6..1.0),
                                size: ice.size * 0.55,
                                rotation: rng.gen_range(0.0..TAU),
                                rot_vel: rng.gen_range(-10.0..10.0),
                                depth: ice.depth + 1,
                            });
                        }
                    }

                    shake += vec2(rng.gen_range(-4.0..4.0), rng.gen_range(-4.0..4.0));
                    world.despawn(e);
                    return;
                }

                ice.pos += ice.vel * timer.delta;
                ice.rotation += ice.rot_vel * timer.delta;

                gfx.rect()
                    .at(ice.pos)
                    .size(Vec2::splat(ice.size * 1.2))
                    .rotate(ice.rotation)
                    .color(Color::new([0.5, 0.7, 1.0, 0.3]));
                gfx.rect()
                    .at(ice.pos)
                    .size(Vec2::splat(ice.size))
                    .rotate(ice.rotation)
                    .color(Color::new([0.7, 0.9, 1.0, 0.9]));
                gfx.rect()
                    .at(ice.pos)
                    .size(Vec2::splat(ice.size * 0.5))
                    .rotate(ice.rotation)
                    .color(Color::new([0.9, 0.95, 1.0, 0.8]));
            });

            for f in fireball_spawns {
                world.spawn((f,));
            }
            for i in ice_spawns {
                world.spawn((i,));
            }
            for p in particle_spawns {
                if particle_count < MAX_PARTICLES {
                    world.spawn((p,));
                    particle_count += 1;
                }
            }

            lightning::update(&world, timer, gfx);

            let mut drawn = 0;
            world.query(|e, p: &mut Particle| {
                p.life -= timer.delta;
                if p.life <= 0.0 {
                    world.despawn(e);
                    return;
                }
                p.pos += p.vel * timer.delta;
                p.vel *= 0.97;
                let t = p.life / p.max_life;

                match p.particle_type {
                    ParticleType::Fire => {
                        gfx.polygon()
                            .segments(6)
                            .at(p.pos)
                            .radius(p.size * t)
                            .color(Color::new([p.color[0], p.color[1], p.color[2], t * 0.8]));
                    }
                    ParticleType::Ice => {
                        gfx.rect()
                            .at(p.pos)
                            .size(Vec2::splat(p.size * t))
                            .rotate(p.life * 3.0)
                            .color(Color::new([p.color[0], p.color[1], p.color[2], t * 0.9]));
                    }
                }
                drawn += 1;
            });

            world.flush_despawned();
            gfx.text(&format!("particles: {} | fps: {:.0}", drawn, timer.fps))
                .color(Color::WHITE);
        });
}
