fn get_value_index(index: u32) -> bool {
    return ((gh[index / 32u] >> (index % 32u)) & 1u) != 0u;
}

// return vec2(
//     texture_value & 0xFFu,
//     texture_value >> 8u,
// );

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
    let data = textureLoad(texture, vec3<i32>(scaled * f32(u.texture_size)).zyx).r;
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

let PI: f32 = 3.14159265358979323846264338327950288;

// max_distance is in terms of t so make sure to normalize your ray direction
fn shoot_ray(r: Ray, max_distance: f32) -> HitInfo {
    var pos = r.pos;
    let dir_mask = vec3<f32>(r.dir == vec3(0.0));
    var dir = r.dir + dir_mask * 0.000001;

    var distance = 0.0;
    if (!in_bounds(r.pos)) {
        // Get position on surface of the octree
        let dist = ray_box_dist(r, vec3(-1.0), vec3(1.0)).x;
        if (dist == 0.0) {
            if (max_distance > 0.0) {
                return HitInfo(false, 0u, vec4(0.0), pos + dir * max_distance, vec3(0.0), vec3(0.0), 1u);
            }
            return intersect_scene(r, 1u);
        }

        pos = r.pos + dir * dist;
        distance += dist;
    }

    var r_sign = sign(dir);
    var the_current_position_of_the_ray = pos;
    var steps = 0u;
    var normal = trunc(pos * 1.00001);
    var voxel = Voxel(0u, vec3(0.0), 0u);
    var reprojection_pos = vec3(0.0);
    var jumped = false;
    while (steps < 1000u) {
        voxel = get_value(the_current_position_of_the_ray);

        // portals
        let should_portal_skip = voxel.data >> 15u == 1u;
        if (should_portal_skip && !jumped) {
            let portal = u.portals[i32((voxel.data >> 8u) & 127u)]; // 0b01111111u retreive the portal index before overwiting the voxel
            let voxel_size = 2.0 / f32(u.texture_size);

            let intersection = ray_plane(Ray(pos, dir), portal.pos, portal.normal);
            if (all(abs(intersection - portal.pos) < vec3(voxel_size) * (vec3<f32>(portal.half_size) + 0.5)) && all(intersection != vec3(0.0))) {
                the_current_position_of_the_ray = intersection + r_sign * abs(portal.normal) * 0.000001;

                let ray_rot_angle = acos(dot(portal.normal, portal.other_normal));
                var ray_rot_axis: vec3<f32>;
                if (any(abs(portal.normal) != abs(portal.other_normal))) {
                    ray_rot_axis = cross(portal.normal, portal.other_normal);
                } else {
                    ray_rot_axis = vec3(0.0, 1.0, 0.0);
                }
                let ray_rot_mat = create_rot_mat(ray_rot_axis, PI - ray_rot_angle);

                let new_pos = (ray_rot_mat * (the_current_position_of_the_ray - portal.pos)) + portal.other_pos;
                let new_dir = ray_rot_mat * dir;

                // return HitInfo(true, voxel.data, vec4(the_current_position_of_the_ray * 10.0, 0.0), the_current_position_of_the_ray, the_current_position_of_the_ray - reprojection_pos, normal, steps);
                // return HitInfo(true, voxel.data, vec4(vec3(ray_rot_angle / u.misc_float / PI), 1.0), the_current_position_of_the_ray, the_current_position_of_the_ray - reprojection_pos, normal, steps);

                reprojection_pos += new_pos - the_current_position_of_the_ray;
                distance += length(new_pos - pos);

                pos = new_pos;
                dir = new_dir;
                r_sign = sign(dir);
                the_current_position_of_the_ray = pos;

                // jumped = true;

                voxel = get_value(the_current_position_of_the_ray);
            }
        }

        if ((voxel.data & 0xFFu) != 0u && !should_portal_skip) {
            break;
        }

        let voxel_size = 2.0 / f32(voxel.grid_size);
        let t_max = (voxel.pos - pos + r_sign * voxel_size / 2.0) / dir;

        // https://www.shadertoy.com/view/4dX3zl (good old shader toy)
        let mask = vec3<f32>(t_max.xyz <= min(t_max.yzx, t_max.zxy));
        normal = mask * -r_sign;

        let t_current = min(min(t_max.x, t_max.y), t_max.z);
        the_current_position_of_the_ray = pos + dir * t_current - normal * 0.000002;

        if (t_current + distance > max_distance && max_distance > 0.0) {
            return HitInfo(false, 0u, vec4(0.0), pos + dir * (max_distance - distance), vec3(0.0), vec3(0.0), steps);
        }

        if (!in_bounds(the_current_position_of_the_ray)) {
            if (max_distance > 0.0) {
                return HitInfo(false, 0u, vec4(0.0), pos + dir * (max_distance - distance), vec3(0.0), vec3(0.0), steps);
            }
            return intersect_scene(Ray(pos, dir), steps);
        }

        steps = steps + 1u;
    }

    return HitInfo(true, voxel.data, u.materials[voxel.data & 0xFFu], the_current_position_of_the_ray, the_current_position_of_the_ray - reprojection_pos, normal, steps);
}