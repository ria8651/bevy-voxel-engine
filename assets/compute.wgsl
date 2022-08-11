#import "common.wgsl"

@group(0) @binding(0)
var<uniform> u: Uniforms;
@group(0) @binding(1)
var<storage, read_write> gh: array<atomic<u32>>; // nodes
@group(0) @binding(2)
var texture: texture_storage_3d<r16uint, read_write>;
@group(0) @binding(3)
var<storage, read> animation_data: array<u32>; // nodes

fn set_value_index(index: u32) {
    atomicOr(&gh[index / 32u], 1u << (index % 32u));
}

fn get_texture_value(pos: vec3<i32>) -> vec2<u32> {
    let texture_value = textureLoad(texture, vec3<i32>(pos)).r;
    return vec2(
        texture_value & 0xFFu,
        texture_value >> 8u,
    );
}

@compute @workgroup_size(1, 1, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    let seed = vec3<u32>(vec3<f32>(pos.xyz) + u.time * 240.0);
    let rand = hash(seed);

    let material = get_texture_value(pos.zyx);

    // just for fun
    if (material.x == 58u && rand.x < 0.01 && u.misc_bool != 0u) {
        textureStore(texture, pos.zyx, vec4(0u));
    }

    // delete old animaiton data
    if (material.y == 1u) {
        textureStore(texture, pos.zyx, vec4(0u));
    }
}

fn write_world_pos(world_pos: vec3<f32>, material: u32, data: u32) {
    let texture_pos = vec3<i32>((world_pos * 0.5 + 0.5) * f32(u.texture_size));
    let voxel_type = get_texture_value(texture_pos.zyx);
    if (voxel_type.x == 0u) {
        textureStore(texture, texture_pos.zyx, vec4(material | (data << 8u)));
    }
}

@compute @workgroup_size(1, 1, 1)
fn update_animation(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // place animation data into world
    let header_len = i32(animation_data[0]);
    let dispatch_size = i32(ceil(pow(f32(header_len), 1.0 / 3.0)));

    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    let index = pos.x * dispatch_size * dispatch_size + pos.y * dispatch_size + pos.z + 1;

    if (index <= header_len) {
        let data_index = i32(u32(animation_data[index]) & 0x00FFFFFFu);
        let data_type = i32(u32(animation_data[index]) >> 24u);

        let material = animation_data[data_index];
        let world_pos = vec3(
            bitcast<f32>(animation_data[data_index + 1]),
            bitcast<f32>(animation_data[data_index + 2]),
            bitcast<f32>(animation_data[data_index + 3]),
        );
        if (data_type == 0) {
            write_world_pos(world_pos, material, 1u); // 0b00000001u
        } else if (data_type == 1) {
            let portal_index = animation_data[data_index + 4];
            let half_size = vec3(
                bitcast<i32>(animation_data[data_index + 5]),
                bitcast<i32>(animation_data[data_index + 6]),
                bitcast<i32>(animation_data[data_index + 7]),
            );
            for (var x = -half_size.x; x <= half_size.x; x++) {
                for (var y = -half_size.y; y <= half_size.y; y++) {
                    for (var z = -half_size.z; z <= half_size.z; z++) {
                        let world_pos = world_pos + 2.0 * vec3<f32>(vec3(x, y, z)) / f32(u.texture_size);
                        write_world_pos(world_pos, material, 128u + portal_index); // 0b10000000u
                    }
                }
            }
            // write_world_pos(world_pos, material);
            // write_world_pos(world_pos + 2.0 * vec3<f32>(half_size) / f32(u.texture_size), material);
        }
    }
}

@compute @workgroup_size(1, 1, 1)
fn rebuild_gh(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    
    let material = get_texture_value(pos.zyx).r;
    if (material != 0u) {
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