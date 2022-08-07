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
var<storage, read_write> gh: array<atomic<u32>>; // nodes
@group(0) @binding(2)
var texture: texture_storage_3d<r8uint, read_write>;

fn set_value_index(index: u32) {
    // return ((gh[index / 32u] >> (index % 32u)) & 1u) != 0u;
    // gh[index / 32u] = gh[index / 32u] | (1u << (index % 32u));
    atomicOr(&gh[index / 32u], 1u << (index % 32u));
}

@compute @workgroup_size(8, 8, 8)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    let seed = vec3<u32>(vec3<f32>(pos.xyz) + u.time * 240.0);

    let rand = hash(seed);

    let material = textureLoad(texture, pos).r;
    if (material == 58u && rand.x < 0.01 && u.misc_bool != 0u) {
        textureStore(texture, pos, vec4(0u));
    }
}

@compute @workgroup_size(8, 8, 8)
fn rebuild_gh(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    
    let material = textureLoad(texture, pos.zyx).r;
    if (material != 0u && u.misc_bool != 0u) {
        // set bits in grid hierarchy
        let size0 = u.levels[0][0];
        let size1 = u.levels[0][1];
        let size2 = u.levels[0][2];
        let size3 = u.levels[0][3];
        let size4 = u.levels[1][0];
        let size5 = u.levels[1][1];
        let size6 = u.levels[1][2];
        let size7 = u.levels[1][3];

        let pos0 = (vec3<u32>(pos) * size0) / u.texture_size;
        let pos1 = (vec3<u32>(pos) * size1) / u.texture_size;
        let pos2 = (vec3<u32>(pos) * size2) / u.texture_size;
        let pos3 = (vec3<u32>(pos) * size3) / u.texture_size;
        let pos4 = (vec3<u32>(pos) * size4) / u.texture_size;
        let pos5 = (vec3<u32>(pos) * size5) / u.texture_size;
        let pos6 = (vec3<u32>(pos) * size6) / u.texture_size;
        let pos7 = (vec3<u32>(pos) * size7) / u.texture_size;

        let index0 = u.offsets[0][0] + pos0.x * size0 * size0 + pos0.y * size0 + pos0.z;
        let index1 = u.offsets[0][1] + pos1.x * size1 * size1 + pos1.y * size1 + pos1.z;
        let index2 = u.offsets[0][2] + pos2.x * size2 * size2 + pos2.y * size2 + pos2.z;
        let index3 = u.offsets[0][3] + pos3.x * size3 * size3 + pos3.y * size3 + pos3.z;
        let index4 = u.offsets[1][0] + pos4.x * size4 * size4 + pos4.y * size4 + pos4.z;
        let index5 = u.offsets[1][1] + pos5.x * size5 * size5 + pos5.y * size5 + pos5.z;
        let index6 = u.offsets[1][2] + pos6.x * size6 * size6 + pos6.y * size6 + pos6.z;
        let index7 = u.offsets[1][3] + pos7.x * size7 * size7 + pos7.y * size7 + pos7.z;

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