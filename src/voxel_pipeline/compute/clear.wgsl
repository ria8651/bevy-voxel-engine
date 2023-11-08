#import bevy_voxel_engine::common::{
    VoxelUniforms,
    ANIMATION_FLAG,
    PORTAL_FLAG
}

#import bevy_voxel_engine::bindings::{
    voxel_world,
    voxel_uniforms,
    gh
}

fn get_texture_value(pos: vec3<i32>) -> vec2<u32> {
    let texture_value = textureLoad(voxel_world, pos.zyx).r;
    return vec2(
        texture_value & 0xFFu,
        texture_value >> 8u,
    );
}

@compute @workgroup_size(4, 4, 4)
fn clear(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));

    let material = get_texture_value(pos);

    // Delete old animation data
    if ((material.y & (ANIMATION_FLAG | PORTAL_FLAG)) > 0u) {
        textureStore(voxel_world, pos.zyx, vec4(0u));
        return;
    }
}