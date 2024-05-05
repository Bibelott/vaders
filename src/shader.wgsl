struct VertOut {
    @builtin(position) pos: vec4f,
    @location(0) tex_c: vec2f,
}

@group(1)
@binding(0)
var<uniform> model: mat4x4<f32>;
@group(0)
@binding(0)
var<uniform> projection: mat4x4<f32>;

@vertex
fn vs_main(@location(0) pos: vec2f, @location(1) tex_coords: vec2f) -> VertOut {
    var out: VertOut;
    out.pos = projection * model * vec4f(pos, 0.0, 1.0);
    out.tex_c = tex_coords;
    return out;
}

@group(1)
@binding(1)
var texture: texture_2d<f32>;
@group(1)
@binding(2)
var samp: sampler;

@fragment
fn fs_main(vert: VertOut) -> @location(0) vec4f {
    return textureSample(texture, samp, vert.tex_c);
}

