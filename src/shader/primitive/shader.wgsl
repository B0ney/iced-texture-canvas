struct Uniforms {
    @location(0) projection: mat4x4<f32>
}

@group(1) @binding(0) 
var<uniform> uniforms: Uniforms;

@group(0) @binding(0)
var t_color: texture_2d<f32>;

@group(0) @binding(1)
var t_sampler: sampler;

struct VertexIn {
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    let pos = array(
        // 1st triangle
        vec2f(0.0,  0.0),  // center
        vec2f(1.0,  0.0),  // right, center
        vec2f(0.0,  1.0),  // center, top

        // 2nd triangle
        vec2f(0.0,  1.0),  // center, top
        vec2f(1.0,  0.0),  // right, center
        vec2f(1.0,  1.0),  // right, top
    );

    let xy = pos[in.vertex_index];
    
    var out: VertexOut;
    out.tex_coord = vec2f(xy.x, xy.y);
    out.position =  uniforms.projection * vec4f(xy, 0.0, 1.0) ;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(t_color, t_sampler, in.tex_coord);
}
