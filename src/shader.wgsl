struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.tex_coords = input.tex_coords;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if (input.tex_coords.x < 0.0 || input.tex_coords.x > 1.0 || 
        input.tex_coords.y < 0.0 || input.tex_coords.y > 1.0) {
        return input.color;
    }
    return textureSample(texture, tex_sampler, input.tex_coords) * input.color;
}