#define_import_path bevy_voxel_engine::bindings

#import bevy_voxel_engine::common::VoxelUniforms

@group(0) @binding(0)
var<uniform> voxel_uniforms: VoxelUniforms;
@group(0) @binding(1)
var voxel_world: texture_storage_3d<r16uint, read_write>;
@group(0) @binding(2)
var<storage, read_write> gh: array<u32>;