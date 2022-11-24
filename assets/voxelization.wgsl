#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings
#import "common.wgsl"

@group(1) @binding(0)
var<uniform> mesh: Mesh;

#import bevy_pbr::mesh_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.pos = mesh_position_local_to_clip(mesh.model, vec4<f32>(vertex.position, 1.0));
    out.uv = vertex.uv;
    return out;
}

@group(2) @binding(0)
var<uniform> u: Uniforms;
@group(2) @binding(1)
var voxel_world: texture_storage_3d<r16uint, read_write>;
@group(2) @binding(2)
var<storage, read> gh: array<u32>;

fn get_texture_value(pos: vec3<i32>) -> vec2<u32> {
    let texture_value = textureLoad(voxel_world, pos.zyx).r;
    return vec2(
        texture_value & 0xFFu,
        texture_value >> 8u,
    );
}

fn write_pos(pos: vec3<i32>, material: u32, flags: u32) {
    // let voxel_type = get_texture_value(pos);
    // if (voxel_type.x == 0u) {
        textureStore(voxel_world, pos.zyx, vec4(material | (flags << 8u)));
    // }
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_pos = in.pos;
    let texture_pos = vec3<i32>(i32(texture_pos.x), 0, i32(texture_pos.y));

    write_pos(texture_pos, 10u, ANIMATION_FLAG);

    return vec4<f32>(vec3(0.0, 1.0, 0.0), 1.0);
}