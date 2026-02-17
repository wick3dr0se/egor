struct Camera {
    view_proj: mat4x4<f32>,
}
@group(1) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

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