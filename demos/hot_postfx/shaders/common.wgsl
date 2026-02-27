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
    @location(3) affine: vec4<f32>,
    @location(4) translate: vec2<f32>,
    @location(5) color: vec4<f32>,
    @location(6) uv: vec4<f32>,
};

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> VertexOutput {
    let rotscale = mat2x2<f32>(inst.affine.xy, inst.affine.zw);
    let world_pos = rotscale * vert.position + inst.translate;
    let uv = vec2<f32>(
        mix(inst.uv.x, inst.uv.z, vert.tex_coords.x),
        mix(inst.uv.y, inst.uv.w, vert.tex_coords.y),
    );

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    out.color = vert.color * inst.color;
    out.tex_coords = uv;
    return out;
}
