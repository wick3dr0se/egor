use egor::{
    app::{App, FrameContext},
    input::MouseButton,
    math::{Vec2, vec2},
    render::Color,
};
use rand::{Rng, rngs::ThreadRng};

struct Crab {
    pos: Vec2,
    vel: Vec2,
}

fn spawn_crab(rng: &mut ThreadRng, bounds: Vec2) -> Crab {
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let pos = vec2(
        rng.gen_range(0.0..bounds.x * 0.33),
        rng.gen_range(0.0..bounds.y * 0.33),
    );
    Crab {
        pos,
        vel: vec2(angle.cos(), angle.sin()) * CRAB_SPEED,
    }
}

fn bounce(pos: &mut Vec2, vel: &mut Vec2, bounds: Vec2, size: f32) {
    if pos.x < 0.0 || pos.x > bounds.x - size {
        vel.x *= -1.0;
        pos.x = pos.x.clamp(0.0, bounds.x - size);
    }
    if pos.y < 0.0 || pos.y > bounds.y - size {
        vel.y *= -1.0;
        pos.y = pos.y.clamp(0.0, bounds.y - size);
    }
}

const CRAB_SIZE: f32 = 32.0;
const CRAB_SPEED: f32 = 600.0;

fn main() {
    let mut crabs = Vec::new();
    let mut ferris_tex = 0;
    let mut rng = rand::thread_rng();

    App::new().title("Egor Ferrismark Demo").run(
        move |FrameContext {
                  gfx, timer, input, ..
              }| {
            let size = gfx.screen_size();

            if timer.frame == 0 {
                ferris_tex = gfx.load_texture(include_bytes!("../assets/ferris_smol.png"));
                crabs.extend((0..2).map(|_| spawn_crab(&mut rng, size)));
            }

            if input.mouse_pressed(MouseButton::Left) {
                crabs.extend((0..9999).map(|_| spawn_crab(&mut rng, size)));
            }

            for c in &mut crabs {
                c.pos += c.vel * timer.delta;
                bounce(&mut c.pos, &mut c.vel, size, CRAB_SIZE);
                gfx.rect()
                    .at(c.pos)
                    .size(Vec2::splat(CRAB_SIZE))
                    .texture(ferris_tex);
            }

            gfx.text("Egor Ferrismark")
                .at((size.x / 2.0 - 50.0, 20.0))
                .size(20.0)
                .color(Color::WHITE);
            gfx.text(&format!("Crabs: {}", crabs.len()))
                .at(vec2(10.0, 10.0))
                .color(Color::WHITE);
            gfx.text(&format!("FPS: {}", timer.fps))
                .at(vec2(10.0, 28.0))
                .color(Color::WHITE);
        },
    );
}
