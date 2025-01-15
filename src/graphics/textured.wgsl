struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) texture: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    in: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.pos, 1.0);
    out.color = vec4<f32>(in.color, 1.0);
    out.tex_coords = vec2<f32>(in.texture);
    return out;
};

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture, tex_sampler, in.tex_coords);
    return tex_color * in.color;
};