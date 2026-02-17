@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, input.tex_coords);
    let uv = input.tex_coords * 2.0 - 1.0;
    let dist = length(uv);
    let vignette = 1.0 - smoothstep(0.5, 1.4, dist);
    return vec4<f32>(tex_color.rgb * vignette, 1.0);
}