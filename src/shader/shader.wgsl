
struct Uniforms {
    @location(0) center: vec2<f32>,
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
	@location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
	var out: VertexOut;
	out.uv = vec2<f32>(0.0);
	out.uv.x = select(0.0, 2.0, in.vertex_index == 1u);
    out.uv.y = select(0.0, 2.0, in.vertex_index == 2u);
	out.position = vec4<f32>(out.uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0) ;
	return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.uv);
}