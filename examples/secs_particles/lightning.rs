use std::f32::consts::TAU;

use egor::{
    math::{Vec2, vec2},
    render::{Color, Graphics},
    time::FrameTimer,
};
use rand::Rng;
use secs::World;

struct LightningSeg {
    a: Vec2,
    b: Vec2,
    life: f32,
    glow: f32,
    thickness: f32,
}

pub fn spawn(
    world: &World,
    rng: &mut impl Rng,
    mut start: Vec2,
    target: Vec2,
    depth: usize,
    branch: u8,
) {
    if depth > 4 {
        return;
    }

    let total_len = start.distance(target);
    let mut traveled = 0.0;
    let thickness = 4.0 / (branch as f32 + 1.0);

    while traveled < total_len {
        let seg_len: f32 = rng.gen_range(8.0..20.0);
        let step_len = seg_len.min(total_len - traveled);
        let dir = (target - start).normalize_or_zero();
        let offset_angle: f32 = rng.gen_range(-0.8..0.8);
        let seg_dir = vec2(
            dir.x * offset_angle.cos() - dir.y * offset_angle.sin(),
            dir.x * offset_angle.sin() + dir.y * offset_angle.cos(),
        );
        let next = start + seg_dir * step_len;

        world.spawn((LightningSeg {
            a: start,
            b: next,
            life: rng.gen_range(0.12..0.22),
            glow: rng.gen_range(0.7..1.0),
            thickness,
        },));

        if branch < 3 && rng.gen_bool(0.5) {
            let forks = if branch == 0 { rng.gen_range(1..3) } else { 1 };
            for _ in 0..forks {
                let fork_angle: f32 = rng.gen_range(-TAU / 3.0..TAU / 3.0);
                let fork_dir = vec2(
                    dir.x * fork_angle.cos() - dir.y * fork_angle.sin(),
                    dir.x * fork_angle.sin() + dir.y * fork_angle.cos(),
                );
                let fork_len: f32 = rng.gen_range(40.0..100.0) / (branch as f32 + 1.0);
                spawn(
                    world,
                    rng,
                    start,
                    start + fork_dir * fork_len,
                    depth + 1,
                    branch + 1,
                );
            }
        }

        start = next;
        traveled += step_len;
    }
}

pub fn update(world: &World, timer: &FrameTimer, gfx: &mut Graphics) {
    world.query(|e, s: &mut LightningSeg| {
        s.life -= timer.delta;
        if s.life <= 0.0 {
            world.despawn(e);
            return;
        }

        let alpha = s.life * s.glow;
        gfx.polyline()
            .points(&[s.a, s.b])
            .thickness(s.thickness * 6.0)
            .color(Color::new([0.3, 0.5, 1.0, alpha * 0.2]));
        gfx.polyline()
            .points(&[s.a, s.b])
            .thickness(s.thickness * 3.0)
            .color(Color::new([0.6, 0.8, 1.0, alpha * 0.5]));
        gfx.polyline()
            .points(&[s.a, s.b])
            .thickness(s.thickness)
            .color(Color::new([1.0, 1.0, 1.0, alpha]));
    });
}
