@group(0) @binding(0) var prefix: texture_multisampled_2d<f32>;
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) source: vec2<u32>,
};
@vertex fn vertex(@builtin(vertex_index) index: u32, @builtin(instance_index) tile: u32) -> VertexOutput {
    let corners = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    let columns = (textureDimensions(prefix).x - 1u) / 32u + 1u;
    var output: VertexOutput;
    output.position = vec4(corners[index], 0.0, 1.0);
    output.source = vec2(tile % columns, tile / columns) * 32u;
    return output;
}
@fragment fn fragment(
    input: VertexOutput,
    @builtin(sample_index) sample: u32,
) -> @location(0) vec4<f32> {
    let source = input.source + vec2<u32>(input.position.xy) % vec2<u32>(32u);
    return textureLoad(prefix, vec2<i32>(source), i32(sample));
}
