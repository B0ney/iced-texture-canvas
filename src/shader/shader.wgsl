
struct Uniforms {
    @location(0) center: vec2<f32>,
}

@group(1) @binding(0) 
var<uniform> uniforms: Uniforms;

@group(0) @binding(0)
var t_color: texture_2d<f32>;

@group(0) @binding(1)
var t_sampler: sampler;

struct VertexIn {
	@location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
}

struct VertexOut {
	@builtin(position) position: vec4<f32>,
	@location(0) tex_coord: vec2<f32>,
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
	var out: VertexOut;
    out.tex_coord = in.tex_coord;
    out.position = vec4<f32>(uniforms.center, 0.0, 1.0) + vec4<f32>(in.position, 0.0, 1.0);
	return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
	let tex = textureSample(t_color, t_sampler, in.tex_coord);
    return tex;
}