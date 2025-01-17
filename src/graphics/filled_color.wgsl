struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) _texture: vec2<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    in: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.pos, 0.0, 1.0);
    out.color = vec4<f32>(in.color, 1.0);
    return out;
};

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return in.color;
};