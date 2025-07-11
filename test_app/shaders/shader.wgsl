@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
     @location(0) uv: vec2<f32>
};





@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    @location(0) position: vec4<f32>,

) -> VertexOutput {
    var out: VertexOutput;
    // if in_vertex_index == 0 {
    //     out.clip_position = vec4<f32>(-1.0, -1.0, 0.0, 1.0);
    //     out.uv = vec2<f32>(0.0, 0.0);
    // } else if in_vertex_index == 1 {
    //     out.clip_position = vec4<f32>(3.0, -1.0, 0.0, 1.0);
    //     out.uv = vec2<f32>(2.0, 0.0);
    // } else {
    //     out.clip_position = vec4<f32>(-1.0, 3.0, 0.0, 1.0);
    //     out.uv = vec2<f32>(0.0, 2.0);
    // }
    out.clip_position = position;
    out.uv = vec2<f32>(position.xy + vec2<f32>(1.0) / 2.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let col = textureLoad(texture, vec2<u32>(in.clip_position.xy));
    return col;
}

 

 