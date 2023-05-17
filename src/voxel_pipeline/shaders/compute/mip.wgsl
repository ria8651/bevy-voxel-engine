#import bevy_voxel_engine::common

@compute @workgroup_size(4, 4, 4)
fn mip(@builtin(global_invocation_id) invocation_id: vec3<u32>) {

}