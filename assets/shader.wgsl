#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings
#import "common.wgsl"

@group(1) @binding(0)
var<uniform> mesh: Mesh;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> @builtin(position) vec4<f32> {
    let world_position = mesh.model * vec4<f32>(vertex.position, 1.0);
    return world_position;
}

@group(2) @binding(0)
var<uniform> u: Uniforms;
@group(2) @binding(1)
var<storage, read_write> gh: array<u32>;
@group(2) @binding(2)
var texture: texture_storage_3d<r16uint, read_write>;
@group(2) @binding(3)
var screen_texture: texture_storage_2d_array<rgba16float, read_write>;

fn get_value_index(index: u32) -> bool {
    return ((gh[index / 32u] >> (index % 32u)) & 1u) != 0u;
}

// return vec2(
//     texture_value & 0xFFu,
//     texture_value >> 8u,
// );

fn get_texture_value(pos: vec3<i32>) -> u32 {
    return textureLoad(texture, vec3<i32>(pos.zyx)).r;
}

struct Voxel {
    data: u32,
    pos: vec3<f32>,
    grid_size: u32,
};

fn get_value(pos: vec3<f32>) -> Voxel {
    let scaled = pos * 0.5 + 0.5;

    let size0 = u.levels[0][0];
    let size1 = u.levels[0][1];
    let size2 = u.levels[0][2];
    let size3 = u.levels[0][3];
    let size4 = u.levels[1][0];
    let size5 = u.levels[1][1];
    let size6 = u.levels[1][2];
    let size7 = u.levels[1][3];

    let scaled0 = vec3<u32>(scaled * f32(size0));
    let scaled1 = vec3<u32>(scaled * f32(size1));
    let scaled2 = vec3<u32>(scaled * f32(size2));
    let scaled3 = vec3<u32>(scaled * f32(size3));
    let scaled4 = vec3<u32>(scaled * f32(size4));
    let scaled5 = vec3<u32>(scaled * f32(size5));
    let scaled6 = vec3<u32>(scaled * f32(size6));
    let scaled7 = vec3<u32>(scaled * f32(size7));

    let index0 = u.offsets[0][0] + scaled0.x * size0 * size0 + scaled0.y * size0 + scaled0.z;
    let index1 = u.offsets[0][1] + scaled1.x * size1 * size1 + scaled1.y * size1 + scaled1.z;
    let index2 = u.offsets[0][2] + scaled2.x * size2 * size2 + scaled2.y * size2 + scaled2.z;
    let index3 = u.offsets[0][3] + scaled3.x * size3 * size3 + scaled3.y * size3 + scaled3.z;
    let index4 = u.offsets[1][0] + scaled4.x * size4 * size4 + scaled4.y * size4 + scaled4.z;
    let index5 = u.offsets[1][1] + scaled5.x * size5 * size5 + scaled5.y * size5 + scaled5.z;
    let index6 = u.offsets[1][2] + scaled6.x * size6 * size6 + scaled6.y * size6 + scaled6.z;
    let index7 = u.offsets[1][3] + scaled7.x * size7 * size7 + scaled7.y * size7 + scaled7.z;

    let state0 = get_value_index(index0);
    let state1 = get_value_index(index1);
    let state2 = get_value_index(index2);
    let state3 = get_value_index(index3);
    let state4 = get_value_index(index4);
    let state5 = get_value_index(index5);
    let state6 = get_value_index(index6);
    let state7 = get_value_index(index7);

    if (!state0 && size0 != 0u) {
        let rounded_pos = ((vec3<f32>(scaled0) + 0.5) / f32(size0)) * 2.0 - 1.0;
        return Voxel(0u, rounded_pos, size0);
    }
    if (!state1 && size1 != 0u) {
        let rounded_pos = ((vec3<f32>(scaled1) + 0.5) / f32(size1)) * 2.0 - 1.0;
        return Voxel(0u, rounded_pos, size1);
    }
    if (!state2 && size2 != 0u) {
        let rounded_pos = ((vec3<f32>(scaled2) + 0.5) / f32(size2)) * 2.0 - 1.0;
        return Voxel(0u, rounded_pos, size2);
    }
    if (!state3 && size3 != 0u) {
        let rounded_pos = ((vec3<f32>(scaled3) + 0.5) / f32(size3)) * 2.0 - 1.0;
        return Voxel(0u, rounded_pos, size3);
    }
    if (!state4 && size4 != 0u) {
        let rounded_pos = ((vec3<f32>(scaled4) + 0.5) / f32(size4)) * 2.0 - 1.0;
        return Voxel(0u, rounded_pos, size4);
    }
    if (!state5 && size5 != 0u) {
        let rounded_pos = ((vec3<f32>(scaled5) + 0.5) / f32(size5)) * 2.0 - 1.0;
        return Voxel(0u, rounded_pos, size5);
    }
    if (!state6 && size6 != 0u) {
        let rounded_pos = ((vec3<f32>(scaled6) + 0.5) / f32(size6)) * 2.0 - 1.0;
        return Voxel(0u, rounded_pos, size6);
    }
    if (!state7 && size7 != 0u) {
        let rounded_pos = ((vec3<f32>(scaled7) + 0.5) / f32(size7)) * 2.0 - 1.0;
        return Voxel(0u, rounded_pos, size7);
    }

    let rounded_pos = (floor(pos * f32(u.texture_size) * 0.5) + 0.5) / (f32(u.texture_size) * 0.5);
    let data = get_texture_value(vec3<i32>(scaled * f32(u.texture_size)));
    return Voxel(data, rounded_pos, u.texture_size);
}

struct HitInfo {
    hit: bool,
    data: u32,
    material: vec4<f32>,
    pos: vec3<f32>,
    reprojection_pos: vec3<f32>,
    normal: vec3<f32>,
    steps: u32,
};

fn intersect_scene(r: Ray, steps: u32) -> HitInfo {
    if (u.skybox != 0u) {
        // plane
        // let normal = vec3(0.0, 1.0, 0.0);
        // let denom = dot(normal, r.dir);
        // if (abs(denom) > 0.00001) {
        //     let t = dot(normal, -normal - r.pos) / denom;
        //     if (t >= 0.0) {
        //         let pos = r.pos + r.dir * t;
        //         return HitInfo(true, vec4(vec3(0.2), 0.0), pos, pos, normal, steps);
        //     }
        // }

        let t = ray_box_dist(r, vec3(-1.0), vec3(1.0, -10000.0, 1.0)).x;
        if (t != 0.0) {
            let pos = r.pos + r.dir * t;
            let normal = trunc(pos * vec3(1.00001, 0.0, 1.00001));
            return HitInfo(true, 0u, vec4(vec3(0.2), 0.0), pos, pos, normal, steps);
        }

        let t = ray_box_dist(r, vec3(3.0), vec3(-3.0, -10000.0, -3.0)).y;
        if (t != 0.0) {
            let pos = r.pos + r.dir * t;
            if (pos.y > -1.0) {
                let normal = -trunc(pos / vec3(2.99999));
                let col = skybox(normalize(pos - vec3(0.0, -1.0, 0.0)), u.time);
                // let col = vec3(0.3, 0.3, 0.8);
                return HitInfo(true, 0u, vec4(col, 1.0), pos, pos, normal, steps);
            } else {
                let normal = -trunc(pos / vec3(2.99999, 10000.0, 2.99999));
                return HitInfo(true, 0u, vec4(vec3(0.2), 0.0), pos, pos, normal, steps);
            }
        }
    }

    return HitInfo(false, 0u, vec4(0.0), vec3(0.0), vec3(0.0), vec3(0.0), steps);
}

fn shoot_ray(r: Ray) -> HitInfo {
    var pos = r.pos;
    let dir_mask = vec3<f32>(r.dir == vec3(0.0));
    var dir = r.dir + dir_mask * 0.000001;

    if (!in_bounds(r.pos)) {
        // Get position on surface of the octree
        let dist = ray_box_dist(r, vec3(-1.0), vec3(1.0)).x;
        if (dist == 0.0) {
            return intersect_scene(r, 1u);
        }

        pos = r.pos + dir * dist;
    }

    var r_sign = sign(dir);
    var voxel_pos = pos;
    var steps = 0u;
    var normal = trunc(pos * 1.00001);
    var voxel = Voxel(0u, vec3(0.0), 0u);
    var reprojection_pos = vec3(0.0);
    var jumped = false;
    while (steps < 1000u) {
        voxel = get_value(voxel_pos);

        // portals
        let should_portal_skip = voxel.data >> 15u == 1u;
        if (should_portal_skip && !jumped) {
            let portal = u.portals[i32((voxel.data >> 8u) & 127u)]; // 0b01111111u retreive the portal index before overwiting the voxel

            let voxel_size = 2.0 / f32(u.texture_size);
            let portal_pos_1 = vec3<f32>(portal.pos.xyz - vec3(i32(u.texture_size / 2u))) * voxel_size;
            let portal_pos_2 = vec3<f32>(portal.other_pos.xyz - vec3(i32(u.texture_size / 2u))) * voxel_size;

            let ray_rot_angle = acos(dot(portal.normal.xyz, portal.other_normal.xyz));
            let ray_rot_axis = cross(portal.normal.xyz, portal.other_normal.xyz);
            let ray_rot_mat = create_rot_mat(ray_rot_axis, ray_rot_angle);

            let new_pos = (ray_rot_mat * (voxel_pos - portal_pos_1)) + portal_pos_2;
            let new_dir = ray_rot_mat * -dir;

            // let new_pos = voxel_pos + (portal_pos_2 - portal_pos_1); //  + portal.normal.xyz * voxel_size

            pos = new_pos;
            // dir = new_dir;
            // r_sign = sign(dir);

            jumped = true;
            voxel = get_value(pos);

            // return HitInfo(true, voxel.data, vec4(portal_pos_2, 0.0), voxel_pos, voxel_pos - reprojection_pos, normal, steps);
        }

        // exit if solid
        if ((voxel.data & 0xFFu) != 0u) {
            break;
        }

        let voxel_size = 2.0 / f32(voxel.grid_size);
        let t_max = (voxel.pos - pos + r_sign * voxel_size / 2.0) / dir;

        // https://www.shadertoy.com/view/4dX3zl (good old shader toy)
        let mask = vec3<f32>(t_max.xyz <= min(t_max.yzx, t_max.zxy));
        normal = mask * -r_sign;

        let t_current = min(min(t_max.x, t_max.y), t_max.z);
        voxel_pos = pos + dir * t_current - normal * 0.000002;


        if (!in_bounds(voxel_pos)) {
            return intersect_scene(Ray(pos, dir), steps);
        }

        steps = steps + 1u;
    }

    return HitInfo(true, voxel.data, u.materials[voxel.data & 0xFFu], voxel_pos, voxel_pos - reprojection_pos, normal, steps);
}

        // // if (u.misc_bool != 0u && all(abs(cs) <= vec2(0.01))) {
        // //     let texture_pos = vec3<i32>((voxel_pos * 0.5 + 0.5) * f32(u.texture_size));
        // //     textureStore(texture, texture_pos.zyx, vec4(46u));
        // // }

        // let should_portal_skip = voxel.data >> 7u == 1u;
        // if (should_portal_skip && !jumped) {
        //     let portal = u.portals[i32(voxel.data & 127u)]; // 0b01111111u retreive the portal index before overwiting the voxel
        //     // voxel = get_value(voxel_pos);
        //     // // reached back face of portal, proceed through the fourth dimension / wormhole or whatever
        //     // if (voxel.data >> 7u != 1u) {
        //         // let ray_rot_angle = acos(dot(portal.normal.xyz, portal.other_normal.xyz));
        //         // let ray_rot_axis = cross(portal.normal.xyz, portal.other_normal.xyz);
        //         // let ray_rot_mat = create_rot_mat(ray_rot_axis, -ray_rot_angle);
                
        //         // let ray_offset = voxel_pos - vec3<f32>(portal.pos.xyz - vec3(i32(u.texture_size / 2u))) * voxel_size;
        //         // let ray_offset_rot = ray_rot_mat * ray_offset;
        //         // let new_voxel_pos = ray_offset_rot + vec3<f32>(portal.other_pos.xyz - vec3(i32(u.texture_size / 2u))) * voxel_size;
        //         // let portal_offset = new_voxel_pos - pos;

        //         // return HitInfo(true, vec4(pos, 1.0), voxel_pos, voxel_pos - reprojection_pos, normal, steps);
        //         // dir = ray_rot_mat * -dir;
        //         // dir = portal.normal.xyz;
        //         // voxel_pos = new_voxel_pos;
        //         // pos = voxel_pos;
        //         // dir = normalize(-voxel_pos);
        //         // r_sign = sign(dir);
        //         // pos = pos + 10.0 * portal.normal.xyz * voxel_size;
        //         let portal_pos_1 = vec3<f32>(portal.pos.xyz - vec3(i32(u.texture_size / 2u))) * voxel_size;
        //         let portal_pos_2 = vec3<f32>(portal.other_pos.xyz - vec3(i32(u.texture_size / 2u))) * voxel_size;
        //         // pos = voxel_pos + (portal_pos_2 - portal_pos_1);
        //         pos = voxel_pos + vec3(1.0, 0.0, 0.0) * u.misc_float * voxel_size * 10.0 + portal.normal.xyz * voxel_size;
        //         jumped = true;
        //         // return HitInfo(true, vec4(pos * 10.0, 1.0), voxel_pos, voxel_pos - reprojection_pos, normal, steps);
        //     // }

        //     voxel = get_value(pos);
        // }

let light_dir = vec3<f32>(1.3, -1.0, 0.8);

fn calculate_direct(material: vec4<f32>, pos: vec3<f32>, normal: vec3<f32>, seed: vec3<u32>) -> vec3<f32> {
    var lighting = 0.0;
    if (material.a == 0.0) {
        // ambient
        let ambient = 0.2;

        // diffuse
        let diffuse = max(dot(normal, -normalize(light_dir)), 0.0);

        // shadow
        var shadow = 1.0;
        if (u.shadows != 0u) {
            let rand = hash(seed) * 2.0 - 1.0;
            let shadow_ray = Ray(pos + normal * 0.0000025, -light_dir + rand * 0.1);
            let shadow_hit = shoot_ray(shadow_ray);
            shadow = f32(!(shadow_hit.hit && any(shadow_hit.material == vec4(0.0))));
        }

        lighting = ambient + diffuse * shadow;
    } else {
        lighting = 1.0;
    }
    return lighting * material.rgb;
}

@fragment
fn fragment(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32> {
    // pixel jitter
    let seed = vec3<u32>(frag_pos.xyz + u.time * 240.0);
    let jitter = vec4(hash(seed).xy - 0.5, 0.0, 0.0) / 1.1;
    var clip_space = get_clip_space(frag_pos, u.resolution);
    let aspect = u.resolution.x / u.resolution.y;
    clip_space.x = clip_space.x * aspect;
    var output_colour = vec3(0.0, 0.0, 0.0);

    let pos = u.camera_inverse * vec4(0.0, 0.0, 0.0, 1.0);
    let dir = u.camera_inverse * vec4(clip_space.x, clip_space.y, -1.0, 1.0);
    let pos = pos.xyz;
    let dir = normalize(dir.xyz - pos);
    var ray = Ray(pos.xyz, dir.xyz);

    let hit = shoot_ray(ray);
    var steps = hit.steps;

    var samples = 0.0;
    if (hit.hit || any(hit.material != vec4(0.0))) {
        // direct lighting
        let direct_lighting = calculate_direct(hit.material, hit.pos, hit.normal, seed + 15u);

        // indirect lighting
        var indirect_lighting = vec3(0.2);
        if (u.indirect_lighting != 0u) {
            let indirect_dir = cosine_hemisphere(hit.normal, seed + 10u);
            let indirect_hit = shoot_ray(Ray(hit.pos + hit.normal * 0.0000025, indirect_dir));
            if (indirect_hit.hit) {
                indirect_lighting = calculate_direct(indirect_hit.material, indirect_hit.pos, indirect_hit.normal, seed + 20u);
            } else {
                indirect_lighting = vec3<f32>(0.2);
            }
        }

        // final blend
        output_colour = direct_lighting + indirect_lighting;

        // reprojection
        let last_frame_clip_space = u.last_camera * vec4<f32>(hit.reprojection_pos, 1.0);
        var last_frame_pos = vec2<f32>(-1.0, 1.0) * (last_frame_clip_space.xy / last_frame_clip_space.z);
        last_frame_pos.x = last_frame_pos.x / aspect;
        let texture_pos = vec2<i32>((last_frame_pos.xy * 0.5 + 0.5) * u.resolution);

        var last_frame_col = textureLoad(screen_texture, texture_pos, 0);
        var last_frame_pos = textureLoad(screen_texture, texture_pos, 1);

        let last_frame_clip_space_from_texture = u.last_camera * vec4<f32>(last_frame_pos.xyz, 1.0);
        if (length(last_frame_clip_space.z - last_frame_clip_space_from_texture.z) > 0.001) {
            last_frame_col = vec4<f32>(0.0);
            last_frame_pos = vec4<f32>(0.0);
        }
        if (last_frame_clip_space.z > 0.0) {
            last_frame_col = vec4<f32>(0.0);
            last_frame_pos = vec4<f32>(0.0);
        }

        samples = min(last_frame_col.a + 1.0, u.accumulation_frames);
        if (u.freeze == 0u) {
            output_colour = last_frame_col.rgb + (output_colour - last_frame_col.rgb) / samples;
        } else {
            output_colour = last_frame_col.rgb;
        }
    } else {
        output_colour = vec3<f32>(0.2);
    }

    if (u.freeze == 0u) {
        // store colour for next frame
        let texture_pos = vec2<i32>(frag_pos.xy);
        textureStore(screen_texture, texture_pos, 0, vec4<f32>(output_colour.rgb, samples));
        textureStore(screen_texture, texture_pos, 1, hit.reprojection_pos.xyzz);
    }

    if (u.show_ray_steps != 0u) {
        output_colour = vec3<f32>(f32(steps) / 100.0);
    }

    // output_colour = hit.pos;
    // output_colour = vec3<f32>(f32(all(abs(clip_space) <= vec2(0.01))));

    let knee = 0.2;
    let power = 2.2;
    output_colour = clamp(output_colour, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>((1.0 - knee) * pow(output_colour, vec3<f32>(power)) + knee * output_colour, 1.0);
}