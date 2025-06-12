use rand::Rng;
use secs::World;

use egor::{
    app::{App, Context},
    render::Color,
};

struct Position {
    x: f32,
    y: f32,
}
struct Velocity {
    x: f32,
    y: f32,
}

fn main() {
    let mut rng = rand::thread_rng();
    let world = World::default();
    let speed = 100.0;

    for _ in 0..9999 {
        world.spawn((
            Position {
                x: rng.gen_range(-300.0..300.0),
                y: rng.gen_range(-300.0..300.0),
            },
            Velocity {
                x: rng.gen_range(-1.0..1.0),
                y: rng.gen_range(-1.0..1.0),
            },
        ));
    }
    App::init(world, move |_world, ctx| {
        ctx.set_title("Egor ECS Particles Demo");
    })
    .run(move |world, ctx: &mut Context| {
        let [w, h] = ctx.graphics.screen_size();
        let (hw, hh) = (w / 2.0, h / 2.0);

        world.query(|_, pos: &mut Position, vel: &Velocity| {
            pos.x += vel.x * ctx.timer.delta * speed;
            pos.y += vel.y * ctx.timer.delta * speed;

            if pos.x < -hw {
                pos.x += w;
            }
            if pos.x > hw {
                pos.x -= w;
            }
            if pos.y < -hh {
                pos.y += h;
            }
            if pos.y > hh {
                pos.y -= h;
            }

            ctx.graphics
                .rect()
                .at(pos.x, pos.y)
                .size(10.0, 10.0)
                .color(Color::WHITE);
        });
    });
}
