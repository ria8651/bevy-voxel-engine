#import "common.wgsl"

struct PalleteEntry {
    colour: vec4<f32>,
};

struct Uniforms {
    pallete: array<PalleteEntry, 256>,
    resolution: vec2<f32>,
    last_camera: mat4x4<f32>,
    camera: mat4x4<f32>,
    camera_inverse: mat4x4<f32>,
    levels: array<vec4<u32>, 2>,
    offsets: array<vec4<u32>, 2>,
    time: f32,
    texture_size: u32,
    show_ray_steps: u32,
    accumulation_frames: f32,
    freeze: u32,
    misc_bool: u32,
    misc_float: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;
@group(0) @binding(1)
var<storage, read_write> gh: array<u32>; // nodes
@group(0) @binding(2)
var texture: texture_storage_3d<r8uint, read_write>;

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    let seed = vec3<u32>(vec3<f32>(location.xyz) + u.time * 240.0);

    let rand = hash(seed);

    let material = textureLoad(texture, location).r;
    if (material == 58u && rand.x < 0.01) {
        textureStore(texture, location, vec4(0u, 0u, 0u, 0u));
    }
}