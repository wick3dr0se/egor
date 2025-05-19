use egor::{Color, app::App};
use rand::Rng;
use secs::World;

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

    App::init(|ctx| ctx.set_title("Egor ECS Particles Demo")).run(move |g, _| {
        let [w, h] = g.screen_size();
        let (hw, hh) = (w / 2.0, h / 2.0);

        world.query(|_, pos: &mut Position, vel: &Velocity| {
            pos.x += vel.x;
            pos.y += vel.y;

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

            g.rect()
                .at(pos.x, pos.y)
                .size(10.0, 10.0)
                .color(Color::WHITE);
        });
    });
}
