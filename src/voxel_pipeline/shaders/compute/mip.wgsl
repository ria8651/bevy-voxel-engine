#import bevy_voxel_engine::common

@group(0) @binding(0)
var<uniform> voxel_uniforms: VoxelUniforms;
@group(0) @binding(1)
var voxel_world: texture_storage_3d<r16uint, read>;
@group(0) @binding(2)
var mip_texture: texture_storage_3d<rgba8unorm, read_write>; 

@group(1) @binding(0)
var from_texture: texture_storage_3d<rgba8unorm, read_write>; 
@group(1) @binding(1)
var to_texture: texture_storage_3d<rgba8unorm, read_write>;


fn get_texture_value(pos: vec3<i32>) -> vec2<u32> {
    let texture_value = textureLoad(voxel_world, pos.zyx).r;
    return vec2(
        texture_value & 0xFFu,
        texture_value >> 8u,
    );
}

@compute @workgroup_size(4, 4, 4)
fn copy(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    let material = get_texture_value(pos);
    if material.x != 0u {
        textureStore(mip_texture, pos.zyx, vec4(voxel_uniforms.materials[material.x].rgb, 1.0));
    } else {
        textureStore(mip_texture, pos.zyx, vec4(0.0));
    }
}

@compute @workgroup_size(4, 4, 4)
fn mip(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));

    var sum = vec3(0.0);
    var alpha = 0.0;
    for (var i = 0u; i < 8u; i += 1u) {
        let offset = vec3(
            i & 1u,
            (i >> 1u) & 1u,
            (i >> 2u) & 1u,
        );
        let value = textureLoad(from_texture, pos.zyx * 2 + vec3<i32>(offset));

        sum += value.rgb * value.a;
        alpha += value.a;
    }
    sum /= alpha;
    alpha /= 8.0;

    textureStore(to_texture, pos.zyx, vec4(sum.rgb, alpha));
}