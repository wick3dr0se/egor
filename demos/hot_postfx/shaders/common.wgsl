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

struct InstanceInput {
    @location(3) model_0: vec4<f32>,
    @location(4) model_1: vec4<f32>,
    @location(5) model_2: vec4<f32>,
    @location(6) model_3: vec4<f32>,
    @location(7) color: vec4<f32>,
    @location(8) uv: vec4<f32>,
};

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> VertexOutput {
    let model = mat4x4(inst.model_0, inst.model_1, inst.model_2, inst.model_3);
    let uv = vec2<f32>(
        mix(inst.uv.x, inst.uv.z, vert.tex_coords.x),
        mix(inst.uv.y, inst.uv.w, vert.tex_coords.y),
    );

    var out: VertexOutput;
    out.clip_position = camera.view_proj * model * vec4<f32>(vert.position, 0.0, 1.0);
    out.color = vert.color * inst.color;
    out.tex_coords = uv;
    return out;
}
