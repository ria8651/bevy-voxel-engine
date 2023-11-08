#import bevy_voxel_engine::common::{
    VoxelUniforms,
    PORTAL_FLAG
}

@group(0) @binding(0)
var<uniform> voxel_uniforms: VoxelUniforms;
@group(0) @binding(1)
var voxel_world: texture_storage_3d<r16uint, read_write>;
// Mind the atomic here, this is why we don't import bindings.wgsl
@group(0) @binding(2)
var<storage, read_write> gh: array<atomic<u32>>;

fn get_texture_value(pos: vec3<i32>) -> vec2<u32> {
    let texture_value = textureLoad(voxel_world, pos.zyx).r;
    return vec2(
        texture_value & 0xFFu,
        texture_value >> 8u,
    );
}

fn set_value_index(index: u32) {
    atomicOr(&gh[index / 32u], 1u << (index % 32u));
}

@compute @workgroup_size(4, 4, 4)
fn rebuild_gh(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    
    let material = get_texture_value(pos);
    if (material.x != 0u || (material.y & PORTAL_FLAG) > 0u) {
        // set bits in grid hierarchy
        let size0 = voxel_uniforms.levels[0].x;
        let size1 = voxel_uniforms.levels[1].x;
        let size2 = voxel_uniforms.levels[2].x;
        let size3 = voxel_uniforms.levels[3].x;
        let size4 = voxel_uniforms.levels[4].x;
        let size5 = voxel_uniforms.levels[5].x;
        let size6 = voxel_uniforms.levels[6].x;
        let size7 = voxel_uniforms.levels[7].x;

        let pos0 = (vec3<u32>(pos) * size0) / voxel_uniforms.texture_size;
        let pos1 = (vec3<u32>(pos) * size1) / voxel_uniforms.texture_size;
        let pos2 = (vec3<u32>(pos) * size2) / voxel_uniforms.texture_size;
        let pos3 = (vec3<u32>(pos) * size3) / voxel_uniforms.texture_size;
        let pos4 = (vec3<u32>(pos) * size4) / voxel_uniforms.texture_size;
        let pos5 = (vec3<u32>(pos) * size5) / voxel_uniforms.texture_size;
        let pos6 = (vec3<u32>(pos) * size6) / voxel_uniforms.texture_size;
        let pos7 = (vec3<u32>(pos) * size7) / voxel_uniforms.texture_size;

        let index0 = voxel_uniforms.offsets[0].x + pos0.x * size0 * size0 + pos0.y * size0 + pos0.z;
        let index1 = voxel_uniforms.offsets[1].x + pos1.x * size1 * size1 + pos1.y * size1 + pos1.z;
        let index2 = voxel_uniforms.offsets[2].x + pos2.x * size2 * size2 + pos2.y * size2 + pos2.z;
        let index3 = voxel_uniforms.offsets[3].x + pos3.x * size3 * size3 + pos3.y * size3 + pos3.z;
        let index4 = voxel_uniforms.offsets[4].x + pos4.x * size4 * size4 + pos4.y * size4 + pos4.z;
        let index5 = voxel_uniforms.offsets[5].x + pos5.x * size5 * size5 + pos5.y * size5 + pos5.z;
        let index6 = voxel_uniforms.offsets[6].x + pos6.x * size6 * size6 + pos6.y * size6 + pos6.z;
        let index7 = voxel_uniforms.offsets[7].x + pos7.x * size7 * size7 + pos7.y * size7 + pos7.z;

        if (size0 != 0u) {
            set_value_index(index0);
        }
        if (size1 != 0u) {
            set_value_index(index1);
        }
        if (size2 != 0u) {
            set_value_index(index2);
        }
        if (size3 != 0u) {
            set_value_index(index3);
        }
        if (size4 != 0u) {
            set_value_index(index4);
        }
        if (size5 != 0u) {
            set_value_index(index5);
        }
        if (size6 != 0u) {
            set_value_index(index6);
        }
        if (size7 != 0u) {
            set_value_index(index7);
        }
    }
}