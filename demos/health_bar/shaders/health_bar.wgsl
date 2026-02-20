struct Camera {
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

struct HealthBar {
    fill: f32,
    time: f32,
    low_r: f32,
    low_g: f32,
    low_b: f32,
    high_r: f32,
    high_g: f32,
    high_b: f32,
}

@group(2) @binding(0) var<uniform> params: HealthBar;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = camera.view_proj * vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    output.tex_coords = input.tex_coords;
    return output;
}

fn sdCapsule(p: vec2<f32>, h: f32, r: f32) -> f32 {
    let px = clamp(p.x, -h, h);
    return length(p - vec2<f32>(px, 0.0)) - r;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.tex_coords;
    let centered = (uv - 0.5) * vec2<f32>(2.0, 2.0);

    let aspect = 10.0;
    let p = vec2<f32>(centered.x * aspect, centered.y);
    let half_width = aspect - 1.0;

    let d = sdCapsule(p, half_width, 0.3);

    if d > 0.6 {
        discard;
    }

    let low = vec3<f32>(params.low_r, params.low_g, params.low_b);
    let high = vec3<f32>(params.high_r, params.high_g, params.high_b);
    let bar_color = mix(low, high, params.fill);

    let fill_x = (uv.x - 0.05) / 0.9;
    let fill_edge = smoothstep(params.fill + 0.12, params.fill - 0.12, fill_x);
    let edge_dist = abs(fill_x - params.fill);

    let pulse_strength = mix(1.0, 0.0, params.fill);
    let time_pulse = (sin(params.time * 10.0) + 1.0) / 2.0;
    let pulse = 1.0 - (time_pulse * pulse_strength * 0.5);

    let bloom = 3.0;

    let fill_bloom = exp(-edge_dist * 15.0) * bloom * pulse;
    let edge_glow = exp(-abs(d) * 1.0) * bloom * pulse;
    let inner_glow = (1.0 - smoothstep(0.0, 0.25, abs(centered.y))) * bloom;
    let specular = pow(max(1.0 - abs(centered.y), 0.0), 5.0) * bloom * smoothstep(0.0, -0.15, d);

    let bg = vec3<f32>(0.08, 0.08, 0.02);
    let lit_color = bar_color * pulse + bar_color * inner_glow + bar_color * edge_glow + vec3<f32>(specular);
    let color = mix(bg, lit_color, fill_edge) + bar_color * fill_bloom;
    let final_color = color + bar_color * edge_glow * 0.9;

    let alpha = 1.0 - smoothstep(-0.05, 0.6, d);

    return vec4<f32>(final_color, alpha * pulse);
}
