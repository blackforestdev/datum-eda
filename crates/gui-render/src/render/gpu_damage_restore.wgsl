@group(0) @binding(0) var prefix: texture_multisampled_2d<f32>;
@vertex fn vertex(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corners = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    return vec4(corners[index], 0.0, 1.0);
}
@fragment fn fragment(
    @builtin(position) position: vec4<f32>,
    @builtin(sample_index) sample: u32,
) -> @location(0) vec4<f32> {
    return textureLoad(prefix, vec2<i32>(position.xy), i32(sample));
}
