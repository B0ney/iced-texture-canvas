
struct Uniforms {
    @location(0) center: vec2<f32>,
    @location(1) scale: f32
}

@group(1) @binding(0) 
var<uniform> uniforms: Uniforms;

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

struct VertexIn {
	@builtin(vertex_index) vertex_index: u32,
}

struct VertexOut {
	@builtin(position) position: vec4<f32>,
    // @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
	let position = vec2f(0.0,0.0);
    var out: VertexOut;
    out.position = vec4f(0,0,0,1);
    // out.tex_coords = vec2f(0.0, 0.0);

	return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, vec2f(0.0,0.0));
}