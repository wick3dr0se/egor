@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, input.tex_coords);
    let scanline = sin(input.tex_coords.y * 600.0 * 3.14159) * 0.5 + 0.5;
    let darkened = tex_color.rgb * (0.8 + scanline * 0.2);
    return vec4<f32>(darkened * vec3<f32>(0.9, 1.0, 0.9), 1.0);
}