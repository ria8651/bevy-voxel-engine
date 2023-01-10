#import bevy_voxel_engine::common

@group(0) @binding(0)
var<uniform> voxel_uniforms: VoxelUniforms;
@group(0) @binding(1)
var voxel_world: texture_storage_3d<r16uint, read_write>;
@group(0) @binding(2)
var<storage, read_write> gh: array<atomic<u32>>;

struct ComputeUniforms {
    time: f32,
    delta_time: f32,
}

@group(1) @binding(0)
var<uniform> compute_uniforms: ComputeUniforms;
@group(1) @binding(1)
var<storage, read_write> physics_data: array<u32>;

// note: raytracing.wgsl requires common.wgsl and for you to define u, voxel_world and gh before you import it
#import bevy_voxel_engine::raytracing

@compute @workgroup_size(1, 1, 1)
fn physics(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let header_len = i32(physics_data[0]);
    let dispatch_size = i32(ceil(pow(f32(header_len), 1.0 / 3.0)));

    let pos = vec3(i32(invocation_id.x), i32(invocation_id.y), i32(invocation_id.z));
    let index = pos.x * dispatch_size * dispatch_size + pos.y * dispatch_size + pos.z + 1;

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
        var gravity = vec3(
            bitcast<f32>(physics_data[data_index + 6]),
            bitcast<f32>(physics_data[data_index + 7]),
            bitcast<f32>(physics_data[data_index + 8]),
        );
        var collision_effect = vec3(
            bitcast<f32>(physics_data[data_index + 9]),
            bitcast<f32>(physics_data[data_index + 10]),
            bitcast<f32>(physics_data[data_index + 11]),
        );
        var hit_normal = vec3(0.0);
        var portal_rotation = IDENTITY;

        velocity += gravity * compute_uniforms.delta_time;

        if (data_type == 0) {
            // point

            // step point by ray
            if (any(abs(velocity) > vec3(0.0001))) {
                let direction = Ray(world_pos, normalize(velocity));
                let distance = length(velocity) * compute_uniforms.delta_time;
                let hit = shoot_ray(direction, distance, COLLISION_FLAG);
                portal_rotation = hit.portals;
                world_pos = hit.pos;
                velocity = (hit.portals * vec4(velocity, 0.0)).xyz;

                if (hit.hit) {
                    // velocity = reflect(velocity, normalize(hit.normal));
                    // velocity = hit.normal * 10.0;
                    velocity = velocity - dot(velocity, hit.normal) * hit.normal;
                    hit_normal = hit.normal;
                    
                    // collision effects
                    let texture_coords = vec3<i32>(world_pos * VOXELS_PER_METER + vec3(f32(voxel_uniforms.texture_size) / 2.0));
                    if collision_effect.x != 0.0 {
                        let radius = collision_effect.y;
                        let range = i32(ceil(radius * VOXELS_PER_METER));
                        for (var x = -range; x <= range; x++) {
                            for (var y = -range; y <= range; y++) {
                                for (var z = -range; z <= range; z++) {
                                    let offset = vec3(x, y, z);
                                    let texture_coords = texture_coords + offset;
                                    if (length(vec3<f32>(offset) / VOXELS_PER_METER) >= radius) {
                                        continue;
                                    }

                                    // destroy
                                    if (collision_effect.x == 1.0) {
                                        textureStore(voxel_world, texture_coords.zyx, vec4(0u));
                                    }
                                    // place
                                    if (collision_effect.x == 2.0) {
                                        let material = bitcast<u32>(collision_effect.z);
                                        textureStore(voxel_world, texture_coords.zyx, vec4(material));
                                    }
                                    // set flags
                                    if (collision_effect.x == 3.0) {
                                        let flags = bitcast<u32>(collision_effect.z);
                                        var voxel = textureLoad(voxel_world, texture_coords.zyx).r;
                                        voxel |= flags << 8u;
                                        textureStore(voxel_world, texture_coords.zyx, vec4(voxel));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if (data_type == 1) {
            // player
            if (any(abs(velocity) > vec3(0.01))) {
                let direction = normalize(velocity);
                let distance = length(velocity) * compute_uniforms.delta_time;

                let size = vec3(
                    bitcast<i32>(physics_data[data_index + 24]),
                    bitcast<i32>(physics_data[data_index + 25]),
                    bitcast<i32>(physics_data[data_index + 26]),
                );
                let v_sign = sign(velocity);

                // x face
                for (var y = -size.y; y <= size.y; y++) {
                    for (var z = -size.z; z <= size.z; z++) {
                        let offset = vec3(f32(size.x) * v_sign.x, f32(y), f32(z)) / (VOXELS_PER_METER * 1.0001);
                        let hit = shoot_ray(Ray((world_pos + offset), direction), distance, COLLISION_FLAG);
                        
                        let plane_normal = vec3(1.0, 0.0, 0.0);
                        if (hit.hit && all(abs(hit.normal) == plane_normal)) {
                            velocity = velocity - dot(velocity, plane_normal) * plane_normal;
                            // world_pos = hit.pos - offset;
                        }
                    }
                }

                // y face
                for (var x = -size.x; x <= size.x; x++) {
                    for (var z = -size.z; z <= size.z; z++) {
                        let offset = vec3(f32(x), f32(size.y) * v_sign.y, f32(z)) / (VOXELS_PER_METER * 1.001);
                        let hit = shoot_ray(Ray((world_pos + offset), direction), distance, COLLISION_FLAG);
                        
                        let plane_normal = vec3(0.0, 1.0, 0.0);
                        if (hit.hit && all(abs(hit.normal) == plane_normal)) {
                            velocity = velocity - dot(velocity, plane_normal) * plane_normal;
                            // world_pos = hit.pos - offset;
                        }
                    }
                }

                // z face
                for (var x = -size.x; x <= size.x; x++) {
                    for (var y = -size.y; y <= size.y; y++) {
                        let offset = vec3(f32(x), f32(y), f32(size.z) * v_sign.z) / (VOXELS_PER_METER * 1.0001);
                        let hit = shoot_ray(Ray((world_pos + offset), direction), distance, COLLISION_FLAG);
                        
                        let plane_normal = vec3(0.0, 0.0, 1.0);
                        if (hit.hit && all(abs(hit.normal) == plane_normal)) {
                            velocity = velocity - dot(velocity, plane_normal) * plane_normal;
                            // world_pos = hit.pos - offset;
                        }
                    }
                }

                if (any(abs(velocity) > vec3(0.01))) {
                    let direction = normalize(velocity * compute_uniforms.delta_time);
                    let distance = length(velocity) * compute_uniforms.delta_time;
                    let hit = shoot_ray(Ray(world_pos, direction), distance, 1u);
                    portal_rotation = hit.portals;
                    velocity = (hit.portals * vec4(velocity, 0.0)).xyz;
                    world_pos = hit.pos;
                }
            }
        }

        physics_data[data_index + 0] = bitcast<u32>(world_pos.x);
        physics_data[data_index + 1] = bitcast<u32>(world_pos.y);
        physics_data[data_index + 2] = bitcast<u32>(world_pos.z);
        physics_data[data_index + 3] = bitcast<u32>(velocity.x);
        physics_data[data_index + 4] = bitcast<u32>(velocity.y);
        physics_data[data_index + 5] = bitcast<u32>(velocity.z);
        physics_data[data_index + 12] = bitcast<u32>(hit_normal.x);
        physics_data[data_index + 13] = bitcast<u32>(hit_normal.y);
        physics_data[data_index + 14] = bitcast<u32>(hit_normal.z);
        physics_data[data_index + 15] = bitcast<u32>(portal_rotation.x.x);
        physics_data[data_index + 16] = bitcast<u32>(portal_rotation.x.y);
        physics_data[data_index + 17] = bitcast<u32>(portal_rotation.x.z);
        physics_data[data_index + 18] = bitcast<u32>(portal_rotation.y.x);
        physics_data[data_index + 19] = bitcast<u32>(portal_rotation.y.y);
        physics_data[data_index + 20] = bitcast<u32>(portal_rotation.y.z);
        physics_data[data_index + 21] = bitcast<u32>(portal_rotation.z.x);
        physics_data[data_index + 22] = bitcast<u32>(portal_rotation.z.y);
        physics_data[data_index + 23] = bitcast<u32>(portal_rotation.z.z);
    }
}