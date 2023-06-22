#import bevy_voxel_engine::common

@group(0) @binding(0)
var<uniform> voxel_uniforms: VoxelUniforms;
@group(0) @binding(1)
var voxel_world: texture_storage_3d<r16uint, read>;
@group(0) @binding(2)
var mip_texture: texture_storage_3d<rgba8unorm, read_write>; 

fn get_texture_value(pos: vec3<i32>) -> vec2<u32> {
    let texture_value = textureLoad(voxel_world, pos.zyx).r;
    return vec2(
        texture_value & 0xFFu,
        texture_value >> 8u,
    );
}

@compute @workgroup_size(4, 4, 4)
fn mip(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    let material = get_texture_value(pos);
    let color = voxel_uniforms.materials[material.x];

    textureStore(mip_texture, pos.zyx, color);
}