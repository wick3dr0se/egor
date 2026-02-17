use egor::{
    app::{App, FrameContext},
    math::{Vec2, vec2},
    render::{Color},
};

fn main() {
    let mut vignette_shader = None;
    let mut offscreen_target = None;
    let mut offscreen_texture_id = None;
    
    App::new()
        .title("Egor Hot Reload/Post Processing Demo")
        .window_size(800, 600)
        .run(move |FrameContext { gfx, .. }| {
            gfx.clear(Color::WHITE);
            let screen_size = gfx.screen_size();

            if vignette_shader.is_none() {
                vignette_shader = Some(gfx.load_shader(
                    r#"
                    struct Camera {
                        view_proj: mat4x4<f32>,
                    }
                    @group(1) @binding(0) var<uniform> camera: Camera;
                    @group(0) @binding(0) var t_diffuse: texture_2d<f32>;
                    @group(0) @binding(1) var s_diffuse: sampler;
                    
                    struct VertexInput {
                        @location(0) position: vec2<f32>,
                        @location(1) tex_coords: vec2<f32>,
                        @location(2) color: vec4<f32>,
                    }
                    
                    struct VertexOutput {
                        @builtin(position) clip_position: vec4<f32>,
                        @location(0) tex_coords: vec2<f32>,
                        @location(1) color: vec4<f32>,
                    }
                    
                    @vertex
                    fn vs_main(input: VertexInput) -> VertexOutput {
                        var output: VertexOutput;
                        output.clip_position = camera.view_proj * vec4<f32>(input.position, 0.0, 1.0);
                        output.tex_coords = input.tex_coords;
                        output.color = input.color;
                        return output;
                    }
                    
                    @fragment
                    fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
                        let tex_color = textureSample(t_diffuse, s_diffuse, input.tex_coords);
                        
                        // Vignette calculation
                        let uv = input.tex_coords * 2.0 - 1.0;
                        let dist = length(uv);
                        let vignette = 1.0 - smoothstep(0.5, 1.4, dist);
                        
                        return vec4<f32>(tex_color.rgb * vignette, tex_color.a);
                    }
                "#,
                ));
            }
            
            if offscreen_target.is_none() {
                let target = gfx.create_offscreen(
                    screen_size.x as u32, 
                    screen_size.y as u32
                );
                // Register texture BEFORE first use
                let tex_id = gfx.offscreen_as_texture(&target);
                offscreen_texture_id = Some(tex_id);
                offscreen_target = Some(target);
            }
            
            let offscreen = offscreen_target.as_mut().unwrap();
            let texture_id = offscreen_texture_id.unwrap();
            
            gfx.render_offscreen(offscreen, |gfx| {
                gfx.clear(Color::BLACK);
                
                gfx.rect()
                    .at(vec2(300., 200.))
                    .size(Vec2::splat(200.0))
                    .color(Color::new([1.0, 0.3, 0.5, 1.0]));
                
                gfx.rect()
                    .at(vec2(100., 400.))
                    .size(vec2(150.0, 100.0))
                    .color(Color::new([0.2, 0.8, 1.0, 1.0]));
                
                gfx.polygon()
                    .at(vec2(600., 300.))
                    .radius(80.0)
                    .segments(32)
                    .color(Color::new([1.0, 0.8, 0.2, 1.0]));
            });
            
            if let Some(shader_id) = vignette_shader {
                gfx.with_shader(shader_id, |gfx| {
                    gfx.rect()
                        .at(vec2(0., 0.))
                        .size(screen_size)
                        .texture(texture_id)
                        .color(Color::WHITE);
                });
            }
        });
}