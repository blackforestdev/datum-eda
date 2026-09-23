@group(0) @binding(0) var glyph_page: texture_2d<f32>;
struct Input {
    @location(0) rect: vec4<f32>,
    @location(1) tex: vec4<u32>,
    @location(2) color: u32,
    @location(3) is_color: u32,
};
struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) tex: vec2<f32>,
    @location(1) @interpolate(flat) color: vec4<f32>,
    @location(2) @interpolate(flat) is_color: u32,
};
fn linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045));
}
@vertex fn vertex(input: Input, @builtin(vertex_index) index: u32) -> Output {
    let corners = array<vec2<f32>, 6>(vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0));
    let corner = corners[index];
    var output: Output;
    output.position = vec4(input.rect.xy + corner * input.rect.zw, 0.0, 1.0);
    output.tex = vec2<f32>(input.tex.xy) + corner * vec2<f32>(input.tex.zw);
    let channels = vec4<f32>(f32((input.color >> 16u) & 255u), f32((input.color >> 8u) & 255u), f32(input.color & 255u), f32(input.color >> 24u)) / 255.0;
    output.color = vec4(linear(channels.rgb), channels.a);
    output.is_color = input.is_color;
    return output;
}
@fragment fn fragment(input: Output) -> @location(0) vec4<f32> {
    let texel = textureLoad(glyph_page, vec2<i32>(input.tex), 0);
    if input.is_color != 0u { return texel; }
    return vec4(input.color.rgb, input.color.a * texel.r);
}
