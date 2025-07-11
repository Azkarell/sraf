
@group(0) @binding(0)
var storage_texture : texture_storage_2d<rgba8unorm, read_write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    
    let index = vec2(id.x, id.y);
    let value = 0.5;
    let dim = textureDimensions(storage_texture);
    textureStore(storage_texture, index, vec4( f32(workgroup_id.x) / f32(dim.x), f32(workgroup_id.y) / f32(dim.y), 0.0, 1.0));

}