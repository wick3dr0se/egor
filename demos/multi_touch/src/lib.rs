use egor::{
    app::{App, FrameContext},
    input::TouchPhase,
    math::{Vec2, vec2},
    render::Color,
};
use std::collections::HashMap;

const TOUCH_COLORS: &[[f32; 4]] = &[
    [1.0, 0.3, 0.3, 0.9],
    [0.3, 1.0, 0.3, 0.9],
    [0.3, 0.5, 1.0, 0.9],
    [1.0, 1.0, 0.2, 0.9],
    [1.0, 0.4, 1.0, 0.9],
    [0.2, 1.0, 1.0, 0.9],
    [1.0, 0.6, 0.2, 0.9],
    [0.7, 0.3, 1.0, 0.9],
];

egor::main!(main);
pub fn main() {
    let mut touch_paths: HashMap<u64, Vec<Vec2>> = HashMap::new();

    App::new()
        .title("Egor Multi-Touch Demo")
        .simulate_touch_with_mouse(true)
        .run(
            move |FrameContext {
                      gfx, input, timer, ..
                  }| {
                let screen = gfx.screen_size();
                gfx.camera().center(Vec2::ZERO, screen);

                // Track touch paths
                for touch in input.touches() {
                    let world_pos = gfx
                        .camera()
                        .screen_to_world(vec2(touch.location.x as f32, touch.location.y as f32));
                    match touch.phase {
                        TouchPhase::Started => {
                            touch_paths.insert(touch.id, vec![world_pos]);
                        }
                        TouchPhase::Moved => {
                            touch_paths.entry(touch.id).or_default().push(world_pos);
                        }
                        TouchPhase::Ended | TouchPhase::Cancelled => {
                            touch_paths.remove(&touch.id);
                        }
                    }
                }

                // Draw touch paths and dots
                for (idx, (id, path)) in touch_paths.iter().enumerate() {
                    let c = TOUCH_COLORS[idx % TOUCH_COLORS.len()];
                    let color = Color::new(c);
                    let trail_color = Color::new([c[0], c[1], c[2], 0.5]);

                    if path.len() >= 2 {
                        let mut builder = gfx
                            .path()
                            .thickness(3.0)
                            .stroke_color(trail_color)
                            .begin(path[0]);
                        for &pt in &path[1..] {
                            builder = builder.line_to(pt);
                        }
                    }

                    if let Some(&pos) = path.last() {
                        gfx.polygon().segments(16).at(pos).radius(12.0).color(color);
                        gfx.polygon()
                            .segments(16)
                            .at(pos)
                            .radius(6.0)
                            .color(Color::WHITE);

                        let screen_pos = gfx.camera().world_to_screen(pos);
                        gfx.text(&format!("touch {}", id))
                            .at(screen_pos + vec2(16.0, -16.0))
                            .color(color);
                    }
                }

                gfx.text(&format!(
                    "touches: {} | fps: {:.0}",
                    input.touch_count(),
                    timer.fps
                ))
                .color(Color::WHITE);
            },
        );
}
