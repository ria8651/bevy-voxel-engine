#import "common.wgsl"

@group(0) @binding(0)
var<uniform> u: Uniforms;
@group(0) @binding(1)
var<storage, read_write> gh: array<atomic<u32>>;
@group(0) @binding(2)
var texture: texture_storage_3d<r16uint, read_write>;
@group(0) @binding(3)
var<storage, read_write> physics_data: array<u32>;
@group(0) @binding(4)
var<storage, read> animation_data: array<u32>;

// note: raytracing.wgsl requires common.wgsl and for you to define u, gh and texture before you import it
#import "raytracing.wgsl"

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

let VOXELS_PER_METER: f32 = 4.0;

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
    if ((material.y & (ANIMATION_FLAG | PORTAL_FLAG)) > 0u && rand.x < u.misc_float) {
        textureStore(texture, pos.zyx, vec4(0u));
    }
}

@compute @workgroup_size(1, 1, 1)
fn update_physics(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let header_len = i32(physics_data[0]);
    let dispatch_size = i32(ceil(pow(f32(header_len), 1.0 / 3.0)));

    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    let index = pos.x * dispatch_size * dispatch_size + pos.y * dispatch_size + pos.z + 1;

    let wtr = VOXELS_PER_METER * 2.0 / f32(u.texture_size); // world to render ratio
    let rtw = f32(u.texture_size) / (VOXELS_PER_METER * 2.0); // render to world ratio

    if (index <= header_len) {
        let data_index = i32(u32(physics_data[index]) & 0x00FFFFFFu);
        let data_type = i32(u32(physics_data[index]) >> 24u);

        var world_pos = vec3(
            bitcast<f32>(physics_data[data_index + 0]),
            bitcast<f32>(physics_data[data_index + 1]),
            bitcast<f32>(physics_data[data_index + 2]),
        );
        var velocity = vec3(
            bitcast<f32>(physics_data[data_index + 3]),
            bitcast<f32>(physics_data[data_index + 4]),
            bitcast<f32>(physics_data[data_index + 5]),
        );
        var hit_normal = vec3(0.0);
        var portal_rotation = IDENTITY;
        if (data_type == 0) {
            // point

            // step point by ray
            if (any(abs(velocity) > vec3(0.0001))) {
                let direction = Ray(world_pos * wtr, normalize(velocity));
                let distance = length(velocity) * u.delta_time * wtr;
                let hit = shoot_ray(direction, distance, 16u);
                portal_rotation = hit.rot;
                world_pos = hit.pos * rtw;
                velocity = hit.rot * velocity;

                if (hit.hit) {
                    // velocity = reflect(velocity, normalize(hit.normal));
                    // velocity = hit.normal * 10.0;
                    velocity = velocity - dot(velocity, hit.normal) * hit.normal;
                    hit_normal = hit.normal;
                }
            }
        } else if (data_type == 1) {
            // player
            if (any(abs(velocity) > vec3(0.01))) {
                let direction = normalize(velocity);
                let distance = length(velocity) * u.delta_time * wtr;

                let size = vec3(2, 4, 2);
                let v_sign = sign(velocity);

                // x face
                for (var y = -size.y; y <= size.y; y++) {
                    for (var z = -size.z; z <= size.z; z++) {
                        let offset = vec3(f32(size.x) * v_sign.x, f32(y), f32(z)) / (VOXELS_PER_METER * 1.0001);
                        let hit = shoot_ray(Ray((world_pos + offset) * wtr, direction), distance, 16u);
                        
                        let plane_normal = vec3(1.0, 0.0, 0.0);
                        if (hit.hit && all(abs(hit.normal) == plane_normal)) {
                            velocity = velocity - dot(velocity, plane_normal) * plane_normal;
                            // world_pos = hit.pos * rtw - offset;
                        }
                    }
                }

                // y face
                for (var x = -size.x; x <= size.x; x++) {
                    for (var z = -size.z; z <= size.z; z++) {
                        let offset = vec3(f32(x), f32(size.y) * v_sign.y, f32(z)) / (VOXELS_PER_METER * 1.001);
                        let hit = shoot_ray(Ray((world_pos + offset) * wtr, direction), distance, 16u);
                        
                        let plane_normal = vec3(0.0, 1.0, 0.0);
                        if (hit.hit && all(abs(hit.normal) == plane_normal)) {
                            velocity = velocity - dot(velocity, plane_normal) * plane_normal;
                            // world_pos = hit.pos * rtw - offset;
                        }
                    }
                }

                // z face
                for (var x = -size.x; x <= size.x; x++) {
                    for (var y = -size.y; y <= size.y; y++) {
                        let offset = vec3(f32(x), f32(y), f32(size.z) * v_sign.z) / (VOXELS_PER_METER * 1.0001);
                        let hit = shoot_ray(Ray((world_pos + offset) * wtr, direction), distance, 16u);
                        
                        let plane_normal = vec3(0.0, 0.0, 1.0);
                        if (hit.hit && all(abs(hit.normal) == plane_normal)) {
                            velocity = velocity - dot(velocity, plane_normal) * plane_normal;
                            // world_pos = hit.pos * rtw - offset;
                        }
                    }
                }

                if (any(abs(velocity) > vec3(0.01))) {
                    let direction = normalize(velocity * u.delta_time);
                    let distance = length(velocity) * u.delta_time * wtr;
                    let hit = shoot_ray(Ray(world_pos * wtr, direction), distance, 0u);
                    portal_rotation = hit.rot;
                    velocity = hit.rot * velocity;
                    world_pos = hit.pos * rtw;
                }
            }
        }
        physics_data[data_index + 0] = bitcast<u32>(world_pos.x);
        physics_data[data_index + 1] = bitcast<u32>(world_pos.y);
        physics_data[data_index + 2] = bitcast<u32>(world_pos.z);
        physics_data[data_index + 3] = bitcast<u32>(velocity.x);
        physics_data[data_index + 4] = bitcast<u32>(velocity.y);
        physics_data[data_index + 5] = bitcast<u32>(velocity.z);
        physics_data[data_index + 6] = bitcast<u32>(hit_normal.x);
        physics_data[data_index + 7] = bitcast<u32>(hit_normal.y);
        physics_data[data_index + 8] = bitcast<u32>(hit_normal.z);
        physics_data[data_index + 9] = bitcast<u32>(portal_rotation.x.x);
        physics_data[data_index + 10] = bitcast<u32>(portal_rotation.x.y);
        physics_data[data_index + 11] = bitcast<u32>(portal_rotation.x.z);
        physics_data[data_index + 12] = bitcast<u32>(portal_rotation.y.x);
        physics_data[data_index + 13] = bitcast<u32>(portal_rotation.y.y);
        physics_data[data_index + 14] = bitcast<u32>(portal_rotation.y.z);
        physics_data[data_index + 15] = bitcast<u32>(portal_rotation.z.x);
        physics_data[data_index + 16] = bitcast<u32>(portal_rotation.z.y);
        physics_data[data_index + 17] = bitcast<u32>(portal_rotation.z.z);
    }
}

fn write_pos(pos: vec3<i32>, material: u32, data: u32) {
    let voxel_type = get_texture_value(pos.zyx);
    if (voxel_type.x == 0u) {
        textureStore(texture, pos.zyx, vec4(material | (data << 8u)));
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

        
        let texture_pos = vec3(
            bitcast<i32>(animation_data[data_index + 0]),
            bitcast<i32>(animation_data[data_index + 1]),
            bitcast<i32>(animation_data[data_index + 2]),
        );
        if (data_type == 0) {
            // particle
            let material = animation_data[data_index + 3];
            write_pos(texture_pos, material, ANIMATION_FLAG);
        } else if (data_type == 1) {
            // portal
            let half_size = vec3(
                bitcast<i32>(animation_data[data_index + 3]),
                bitcast<i32>(animation_data[data_index + 4]),
                bitcast<i32>(animation_data[data_index + 5]),
            );
            let portal_index = animation_data[data_index + 6];
            for (var x = -half_size.x; x <= half_size.x; x++) {
                for (var y = -half_size.y; y <= half_size.y; y++) {
                    for (var z = -half_size.z; z <= half_size.z; z++) {
                        let texture_pos = texture_pos + vec3(x, y, z);
                        write_pos(texture_pos, portal_index, PORTAL_FLAG);
                    }
                }
            }
        } else if (data_type == 2) {
            // portal frame
            let portal_index = animation_data[data_index + 3];
            let half_size = vec3(
                bitcast<i32>(animation_data[data_index + 4]),
                bitcast<i32>(animation_data[data_index + 5]),
                bitcast<i32>(animation_data[data_index + 6]),
            );
            for (var x = -half_size.x; x <= half_size.x; x++) {
                for (var y = -half_size.y; y <= half_size.y; y++) {
                    for (var z = -half_size.z; z <= half_size.z; z++) {
                        let pos = vec3(x, y, z);
                        if (abs(pos.x) == half_size.x || abs(pos.y) == half_size.y) {
                            if (abs(pos.x) == half_size.x || abs(pos.z) == half_size.z) {
                                if (abs(pos.y) == half_size.y || abs(pos.z) == half_size.z) {
                                    write_pos(texture_pos + pos, portal_index, ANIMATION_FLAG | COLLISION_FLAG);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@compute @workgroup_size(1, 1, 1)
fn rebuild_gh(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    
    let material = get_texture_value(pos.zyx);
    if (material.x != 0u || (material.y & PORTAL_FLAG) > 0u) {
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