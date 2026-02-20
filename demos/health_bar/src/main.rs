use egor::{
    app::{App, FrameContext},
    math::vec2,
    render::Color,
};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct HealthBarParams {
    fill: f32,
    time: f32,
    low_color: [f32; 3],
    high_color: [f32; 3],
}

fn main() {
    let mut shader_id = 0;
    let mut uniform_id = 0;
    let mut elapsed = 0.;

    App::new()
        .title("Egor Health Bar Demo")
        .window_size(800, 600)
        .run(move |FrameContext { gfx, timer, .. }| {
            gfx.clear(Color::new([0.1, 0.1, 0.15, 1.0]));
            let size = gfx.screen_size();

            elapsed += timer.delta;

            let health = ((0.5 * elapsed).sin() + 1.) / 2.;

            if timer.frame == 0 {
                let wgsl = include_str!("../shaders/health_bar.wgsl");
                let params = HealthBarParams {
                    fill: 1.,
                    time: 0.,
                    low_color: [1., 0., 0.],
                    high_color: [0., 1., 0.],
                };
                uniform_id = gfx.create_uniform(bytemuck::bytes_of(&params));
                shader_id = gfx.load_shader_with_uniforms(wgsl, &[uniform_id]);
            }

            let params = HealthBarParams {
                fill: health,
                time: elapsed,
                low_color: [1.0, 0.0, 0.0],
                high_color: [0.0, 1.0, 0.0],
            };
            gfx.update_uniform(uniform_id, bytemuck::bytes_of(&params));

            let bar_size = vec2(300.0, 30.0);
            let bar_pos = vec2((size.x - bar_size.x) * 0.5, size.y * 0.5 - bar_size.y * 0.5);

            gfx.with_shader(shader_id, |gfx| {
                gfx.rect().at(bar_pos).size(bar_size);
            });

            gfx.text(&format!("HP: {:.0}%", health * 100.0))
                .at((size.x * 0.5 - 30.0, bar_pos.y - 30.0))
                .size(20.0)
                .color(Color::WHITE);

            gfx.text(&format!("FPS: {}", timer.fps))
                .at(vec2(10.0, 10.0))
                .color(Color::WHITE);
        });
}
