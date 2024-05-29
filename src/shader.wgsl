
struct Uniforms {
    center: vec2f,
    scale: f32
}

@group(0) @binding(0) 
var<uniform> uniforms: Uniforms;

struct VertexIn {
	@builtin(vertex_index) vertex_index: u32,
}

struct VertexOut {
	@builtin(position) position: vec2f,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut {

}

// Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    textureSample(t_diffuse, s_diffuse, in.tex_coords);
}