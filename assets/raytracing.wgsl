fn get_value_index(index: u32) -> bool {
    return ((gh[index / 32u] >> (index % 32u)) & 1u) != 0u;
}

struct Voxel {
    data: u32,
    pos: vec3<f32>,
    grid_size: u32,
};

fn get_value(pos: vec3<f32>) -> Voxel {
    let scaled = pos * 0.5 + 0.5;

    let size0 = voxel_uniforms.levels[0][0];
    let size1 = voxel_uniforms.levels[0][1];
    let size2 = voxel_uniforms.levels[0][2];
    let size3 = voxel_uniforms.levels[0][3];
    let size4 = voxel_uniforms.levels[1][0];
    let size5 = voxel_uniforms.levels[1][1];
    let size6 = voxel_uniforms.levels[1][2];
    let size7 = voxel_uniforms.levels[1][3];

    let scaled0 = vec3<u32>(scaled * f32(size0));
    let scaled1 = vec3<u32>(scaled * f32(size1));
    let scaled2 = vec3<u32>(scaled * f32(size2));
    let scaled3 = vec3<u32>(scaled * f32(size3));
    let scaled4 = vec3<u32>(scaled * f32(size4));
    let scaled5 = vec3<u32>(scaled * f32(size5));
    let scaled6 = vec3<u32>(scaled * f32(size6));
    let scaled7 = vec3<u32>(scaled * f32(size7));

    let state0 = get_value_index(voxel_uniforms.offsets[0][0] + scaled0.x * size0 * size0 + scaled0.y * size0 + scaled0.z);
    let state1 = get_value_index(voxel_uniforms.offsets[0][1] + scaled1.x * size1 * size1 + scaled1.y * size1 + scaled1.z);
    let state2 = get_value_index(voxel_uniforms.offsets[0][2] + scaled2.x * size2 * size2 + scaled2.y * size2 + scaled2.z);
    let state3 = get_value_index(voxel_uniforms.offsets[0][3] + scaled3.x * size3 * size3 + scaled3.y * size3 + scaled3.z);
    let state4 = get_value_index(voxel_uniforms.offsets[1][0] + scaled4.x * size4 * size4 + scaled4.y * size4 + scaled4.z);
    let state5 = get_value_index(voxel_uniforms.offsets[1][1] + scaled5.x * size5 * size5 + scaled5.y * size5 + scaled5.z);
    let state6 = get_value_index(voxel_uniforms.offsets[1][2] + scaled6.x * size6 * size6 + scaled6.y * size6 + scaled6.z);
    let state7 = get_value_index(voxel_uniforms.offsets[1][3] + scaled7.x * size7 * size7 + scaled7.y * size7 + scaled7.z);

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

    let rounded_pos = (floor(pos * f32(voxel_uniforms.texture_size) * 0.5) + 0.5) / (f32(voxel_uniforms.texture_size) * 0.5);
    let data = textureLoad(voxel_world, vec3<i32>(scaled * f32(voxel_uniforms.texture_size)).zyx).r;
    return Voxel(data, rounded_pos, voxel_uniforms.texture_size);
}

struct HitInfo {
    hit: bool,
    data: u32,
    material: vec4<f32>,
    pos: vec3<f32>,
    portal_offset: vec3<f32>,
    normal: vec3<f32>,
    rot: mat3x3<f32>,
    steps: u32,
};

let IDENTITY = mat3x3<f32>(
    vec3<f32>(1.0, 0.0, 0.0), 
    vec3<f32>(0.0, 1.0, 0.0), 
    vec3<f32>(0.0, 0.0, 1.0)
);

fn intersect_scene(r: Ray, steps: u32) -> HitInfo {
    if (trace_uniforms.skybox != 0u) {
        // // pillar
        // let t = ray_box_dist(r, vec3(-1.0), vec3(1.0, -10000.0, 1.0)).x;
        // if (t != 0.0) {
        //     let pos = r.pos + r.dir * t;
        //     let normal = trunc(pos * vec3(1.00001, 0.0, 1.00001));
        //     return HitInfo(true, 0u, vec4(vec3(0.2), 0.0), pos, vec3(0.0), normal, IDENTITY, steps);
        // }

        // // skybox
        // let t = ray_box_dist(r, vec3(3.0), vec3(-3.0, -10000.0, -3.0)).y;
        // if (t != 0.0) {
        //     let pos = r.pos + r.dir * t;
        //     if (pos.y > -1.0) {
        //         let normal = -trunc(pos / vec3(2.99999));
        //         let col = skybox(normalize(pos - vec3(0.0, -1.0, 0.0)), u.time);
        //         // let col = vec3(0.3, 0.3, 0.8);
        //         return HitInfo(true, 0u, vec4(col, 1.0), pos, vec3(0.0), normal, IDENTITY, steps);
        //     } else {
        //         let normal = -trunc(pos / vec3(2.99999, 10000.0, 2.99999));
        //         return HitInfo(true, 0u, vec4(vec3(0.2), 0.0), pos, vec3(0.0), normal, IDENTITY, steps);
        //     }
        // }

        let normal = vec3(0.0, 1.0, 0.0);
        let hit = ray_plane(r, vec3(0.0, -1.0, 0.0), normal);
        if (any(hit != vec3(0.0))) {
            let pos = hit + normal * 0.000002;
            let colour = vec3(113.0, 129.0, 44.0) / 255.0;
            return HitInfo(true, 0u, vec4(colour, 0.0), pos, vec3(0.0), normal, IDENTITY, steps);
        }
    }

    return HitInfo(false, 0u, vec4(0.0), vec3(0.0), vec3(0.0), vec3(0.0), IDENTITY, steps);
}

let PI: f32 = 3.14159265358979323846264338327950288;

/// physics_distance is in terms of t so make sure to normalize your 
/// ray direction if you want it to be in world cordinates.
/// only hits voxels that have any of the flags set or hits everything if flags is 0
fn shoot_ray(r: Ray, physics_distance: f32, flags: u32) -> HitInfo {
    var pos = r.pos;
    let dir_mask = vec3<f32>(r.dir == vec3(0.0));
    var dir = r.dir + dir_mask * 0.000001;

    var distance = 0.0;
    if (!in_bounds(r.pos)) {
        // Get position on surface of the octree
        let dist = ray_box_dist(r, vec3(-1.0), vec3(1.0)).x;
        if (dist == 0.0) {
            if (physics_distance > 0.0) {
                return HitInfo(false, 0u, vec4(0.0), pos + dir * physics_distance, vec3(0.0), vec3(0.0), IDENTITY, 1u);
            }
            return intersect_scene(r, 1u);
        }

        pos = r.pos + dir * dist;
        distance += dist;
    }

    // let voxel = get_value(pos);
    // let normal = trunc(pos * 1.00001);
    // return HitInfo(true, voxel.data, u.materials[voxel.data & 0xFFu], pos + normal * 0.000004, vec3(0.0), normal, IDENTITY, 10u);

    var r_sign = sign(dir);
    var tcpotr = pos; // the current position of the ray
    var steps = 0u;
    var normal = trunc(pos * 1.00001);
    var voxel = Voxel(0u, vec3(0.0), 0u);
    var portal_offset = vec3(0.0);
    var rot = IDENTITY;
    while (steps < 1000u) {
        voxel = get_value(tcpotr);

        // portals
        let should_portal_skip = ((voxel.data >> 8u) & PORTAL_FLAG) > 0u;
        if (should_portal_skip) {
            let portal = voxel_uniforms.portals[i32(voxel.data & 0xFFu)];
            let voxel_size = 2.0 / f32(voxel_uniforms.texture_size);

            let intersection = ray_plane(Ray(pos, dir), portal.pos, portal.normal);
            if (all(abs(intersection - portal.pos) < vec3(voxel_size) * (vec3<f32>(portal.half_size) + 0.5)) && all(intersection != vec3(0.0))) {
                tcpotr = intersection + r_sign * abs(portal.normal) * 0.0000001;
                if (length(tcpotr - pos) + distance > physics_distance && physics_distance > 0.0) {
                    return HitInfo(false, 0u, vec4(0.0), pos + dir * (physics_distance - distance), vec3(0.0), vec3(0.0), rot, steps);
                }

                let ray_rot_angle = acos(dot(portal.normal, portal.other_normal));
                var ray_rot_axis: vec3<f32>;
                if (any(abs(portal.normal) != abs(portal.other_normal))) {
                    ray_rot_axis = cross(portal.normal, portal.other_normal);
                } else {
                    if (all(abs(portal.normal) == vec3(0.0, 1.0, 0.0))) {
                        ray_rot_axis = vec3(1.0, 0.0, 0.0);
                    } else {
                        ray_rot_axis = vec3(0.0, 1.0, 0.0);
                    }
                }
                let ray_rot_mat = create_rot_mat(ray_rot_axis, PI - ray_rot_angle);

                let new_pos = (ray_rot_mat * (tcpotr - portal.pos)) + portal.other_pos;
                let new_dir = ray_rot_mat * dir;

                // return HitInfo(true, voxel.data, vec4(tcpotr * 10.0, 0.0), tcpotr, tcpotr - portal_offset, normal, steps);
                // return HitInfo(true, voxel.data, vec4(vec3(ray_rot_angle / u.misc_float / PI), 1.0), tcpotr, tcpotr - portal_offset, normal, steps);

                portal_offset -= new_pos - tcpotr;
                distance += length(tcpotr - pos);

                rot = ray_rot_mat * rot;
                pos = new_pos;
                dir = new_dir;
                r_sign = sign(dir);
                tcpotr = pos;

                // jumped = true;

                voxel = get_value(tcpotr);
            }
        }

        if ((voxel.data & 0xFFu) != 0u && !should_portal_skip && (((voxel.data >> 8u) & flags) > 0u || flags == 0u)) {
            break;
        }

        let voxel_size = 2.0 / f32(voxel.grid_size);
        let t_max = (voxel.pos - pos + r_sign * voxel_size / 2.0) / dir;

        // https://www.shadertoy.com/view/4dX3zl (good old shader toy)
        let mask = vec3<f32>(t_max.xyz <= min(t_max.yzx, t_max.zxy));
        normal = mask * -r_sign;

        let t_current = min(min(t_max.x, t_max.y), t_max.z);
        tcpotr = pos + dir * t_current - normal * 0.000002;

        if (t_current + distance > physics_distance && physics_distance > 0.0) {
            return HitInfo(false, 0u, vec4(0.0), pos + dir * (physics_distance - distance), portal_offset, vec3(0.0), rot, steps);
        }

        if (!in_bounds(tcpotr)) {
            if (physics_distance > 0.0) {
                return HitInfo(false, 0u, vec4(0.0), pos + dir * (physics_distance - distance), portal_offset, vec3(0.0), rot, steps);
            }
            return intersect_scene(Ray(pos, dir), steps);
        }

        steps = steps + 1u;
    }

    return HitInfo(true, voxel.data, voxel_uniforms.materials[voxel.data & 0xFFu], tcpotr + normal * 0.000004, portal_offset, normal, rot, steps);
}