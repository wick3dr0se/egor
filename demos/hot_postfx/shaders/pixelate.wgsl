@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_size = 8.0;
    let resolution = vec2<f32>(800.0, 600.0);
    
    let pixelated_uv = floor(input.tex_coords * resolution / pixel_size) 
                       * pixel_size / resolution;
    
    let tex_color = textureSample(t_diffuse, s_diffuse, pixelated_uv);
    return vec4<f32>(tex_color.rgb, 1.0);
}