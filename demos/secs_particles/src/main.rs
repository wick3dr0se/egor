use rand::Rng;
use secs::World;

use egor::{
    app::{App, Context},
    math::{Vec2, vec2},
    render::Color,
};

struct Position {
    vec: Vec2,
}
struct Velocity {
    vec: Vec2,
}

fn wraparound(v: &mut Vec2, size: Vec2) {
    *v = (*v + size / 2.0).rem_euclid(size) - size / 2.0;
}

fn main() {
    let mut rng = rand::thread_rng();
    let world = World::default();
    let speed = 100.0;

    for _ in 0..9999 {
        world.spawn((
            Position {
                vec: vec2(rng.gen_range(-300.0..300.0), rng.gen_range(-300.0..300.0)),
            },
            Velocity {
                vec: vec2(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)),
            },
        ));
    }
    App::init(world, move |_world, ctx| {
        ctx.set_title("Egor ECS Particles Demo");
    })
    .run(move |world, ctx: &mut Context| {
        let screen_size = ctx.graphics.screen_size();

        world.query(|_, pos: &mut Position, vel: &Velocity| {
            pos.vec += vel.vec * speed * ctx.timer.delta;
            wraparound(&mut pos.vec, screen_size);

            ctx.graphics
                .rect()
                .at(pos.vec)
                .size(Vec2::splat(10.0))
                .color(Color::WHITE);
        });
    });
}
