#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct
#import "common.wgsl"

[[group(1), binding(0)]]
var<uniform> mesh: Mesh;

struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_position = mesh.model * vec4<f32>(vertex.position, 1.0);

    var out: VertexOutput;
    out.clip_position = world_position; //view.view_proj * 
    out.uv = vertex.uv;
    return out;
}

struct PalleteEntry {
    colour: vec4<f32>;
};

struct Uniforms {
    pallete: array<PalleteEntry, 256>;
    resolution: vec2<f32>;
    last_camera: mat4x4<f32>;
    camera: mat4x4<f32>;
    camera_inverse: mat4x4<f32>;
    time: f32;
    levels: array<u32, 8>;
    offsets: array<u32, 8>;
    texture_size: u32;
    show_ray_steps: bool;
    freeze: bool;
    misc_bool: bool;
    misc_float: f32;
};

struct GH {
    data: [[stride(4)]] array<u32>;
};

[[group(2), binding(0)]]
var<uniform> u: Uniforms;
[[group(2), binding(1)]]
var screen_texture: texture_storage_2d_array<rgba16float, read_write>;
[[group(2), binding(2)]]
var<storage, read_write> gh: GH; // nodes
[[group(2), binding(3)]]
var texture: texture_storage_3d<r8uint, read_write>;

fn get_value_index(index: u32) -> bool {
    return ((gh.data[index / 32u] >> (index % 32u)) & 1u) != 0u;
}

struct Voxel {
    value: u32;
    pos: vec3<f32>;
    grid_size: u32;
};

fn get_value(pos: vec3<f32>) -> Voxel {
    let scaled = pos * 0.5 + 0.5;

    let size0 = u.levels[0];
    let size1 = u.levels[1];
    let size2 = u.levels[2];
    let size3 = u.levels[3];
    let size4 = u.levels[4];
    let size5 = u.levels[5];
    let size6 = u.levels[6];
    let size7 = u.levels[7];

    let scaled0 = vec3<u32>(scaled * f32(u.levels[0]));
    let scaled1 = vec3<u32>(scaled * f32(u.levels[1]));
    let scaled2 = vec3<u32>(scaled * f32(u.levels[2]));
    let scaled3 = vec3<u32>(scaled * f32(u.levels[3]));
    let scaled4 = vec3<u32>(scaled * f32(u.levels[4]));
    let scaled5 = vec3<u32>(scaled * f32(u.levels[5]));
    let scaled6 = vec3<u32>(scaled * f32(u.levels[6]));
    let scaled7 = vec3<u32>(scaled * f32(u.levels[7]));

    let index0 = u.offsets[0] + scaled0.x * size0 * size0 + scaled0.y * size0 + scaled0.z;
    let index1 = u.offsets[1] + scaled1.x * size1 * size1 + scaled1.y * size1 + scaled1.z;
    let index2 = u.offsets[2] + scaled2.x * size2 * size2 + scaled2.y * size2 + scaled2.z;
    let index3 = u.offsets[3] + scaled3.x * size3 * size3 + scaled3.y * size3 + scaled3.z;
    let index4 = u.offsets[4] + scaled4.x * size4 * size4 + scaled4.y * size4 + scaled4.z;
    let index5 = u.offsets[5] + scaled5.x * size5 * size5 + scaled5.y * size5 + scaled5.z;
    let index6 = u.offsets[6] + scaled6.x * size6 * size6 + scaled6.y * size6 + scaled6.z;
    let index7 = u.offsets[7] + scaled7.x * size7 * size7 + scaled7.y * size7 + scaled7.z;

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
    let value = textureLoad(texture, vec3<i32>(scaled.zyx * f32(u.texture_size))).r;
    return Voxel(value, rounded_pos, u.texture_size);
}

struct HitInfo {
    hit: bool;
    value: u32;
    pos: vec3<f32>;
    normal: vec3<f32>;
    steps: u32;
};

fn shoot_ray(r: Ray) -> HitInfo {
    var pos = r.pos;
    let dir_mask = vec3<f32>(r.dir == vec3<f32>(0.0));
    var dir = r.dir + dir_mask * 0.000001;

    if (!in_bounds(r.pos)) {
        // Get position on surface of the octree
        let dist = ray_box_dist(r, vec3<f32>(-1.0), vec3<f32>(1.0));
        if (dist == 0.0) {
            return HitInfo(false, 0u, vec3<f32>(0.0), vec3<f32>(0.0), 0u);
        }

        pos = r.pos + dir * (dist + 0.00001);
    }

    let r_sign = sign(dir);

    var voxel_pos = pos;
    var steps = 0u;
    var normal = trunc(pos * 1.00001);
    var voxel = Voxel(0u, vec3<f32>(0.0), 0u);
    loop {
        voxel = get_value(voxel_pos);
        let voxel_size = 2.0 / f32(voxel.grid_size);
        if (voxel.value != 0u) {
            return HitInfo(true, voxel.value, voxel_pos, normal, steps);
        }

        let voxel_size = 2.0 / f32(voxel.grid_size);
        let t_max = (voxel.pos - pos + r_sign * voxel_size / 2.0) / dir;

        // https://www.shadertoy.com/view/4dX3zl (good old shader toy)
        let mask = vec3<f32>(t_max.xyz <= min(t_max.yzx, t_max.zxy));
        normal = mask * -r_sign;

        let t_current = min(min(t_max.x, t_max.y), t_max.z);
        voxel_pos = pos + dir * t_current - normal * 0.000002;

        if (!in_bounds(voxel_pos)) {
            return HitInfo(false, 0u, vec3<f32>(0.0), vec3<f32>(0.0), steps);
        }

        steps = steps + 1u;
        if (steps > 1000u) {
            return HitInfo(true, voxel.value, voxel_pos, normal, steps);
        }
    }

    return HitInfo(true, voxel.value, voxel_pos, normal, steps);
}

let light_dir = vec3<f32>(-1.3, -1.0, 0.8);

fn calculate_direct(material: vec4<f32>, pos: vec3<f32>, normal: vec3<f32>, seed: vec3<u32>) -> vec3<f32> {
    var lighting = 0.0;
    if (material.a == 0.0) {
        // ambient
        let ambient = 0.0;

        // diffuse
        let diffuse = max(dot(normal, -normalize(light_dir)), 0.0);

        // shadow
        let rand = hash(seed) * 2.0 - 1.0;
        let shadow_ray = Ray(pos + normal * 0.0000025, -light_dir + rand * 0.1);
        let shadow_hit = shoot_ray(shadow_ray);
        let shadow = f32(!shadow_hit.hit);

        lighting = ambient + diffuse * shadow;
    } else {
        lighting = 1.0;
    }
    return lighting * material.rgb;
}

[[stage(fragment)]]
fn fragment([[builtin(position)]] frag_pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
    // pixel jitter
    let seed = vec3<u32>(frag_pos.xyz + u.time * 240.0);
    // let jitter = vec4<f32>(hash(seed).xy - 0.5, 0.0, 0.0);
    var clip_space = get_clip_space(frag_pos, u.resolution);
    let aspect = u.resolution.x / u.resolution.y;
    clip_space.x = clip_space.x * aspect;
    var output_colour = vec3<f32>(0.0, 0.0, 0.0);

    let pos = u.camera_inverse * vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let dir = u.camera_inverse * vec4<f32>(clip_space.x, clip_space.y, -1.0, 1.0);
    let pos = pos.xyz;
    let dir = normalize(dir.xyz - pos);
    var ray = Ray(pos.xyz, dir.xyz);

    let hit = shoot_ray(ray);
    var steps = hit.steps;

    var samples = 0.0;
    if (hit.hit) {
        let material = u.pallete[hit.value].colour;
            
        // direct lighting
        let direct_lighting = calculate_direct(material, hit.pos, hit.normal, seed + 15u);

        // indirect lighting
        var indirect_lighting: vec3<f32>;
        let indirect_dir = cosine_hemisphere(hit.normal, seed + 10u);
        let indirect_hit = shoot_ray(Ray(hit.pos + hit.normal * 0.0000025, indirect_dir));
        if (indirect_hit.hit) {
            let indirect_material = u.pallete[indirect_hit.value].colour;
            indirect_lighting = calculate_direct(indirect_material, indirect_hit.pos, indirect_hit.normal, seed + 15u);
        } else {
            indirect_lighting = vec3<f32>(0.2);
        }

        // final blend
        output_colour = (direct_lighting + indirect_lighting) * material.rgb * 1.5;

        // reprojection
        let last_frame_clip_space = u.last_camera * vec4<f32>(hit.pos, 1.0);
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

        samples = min(last_frame_col.a + 1.0, u.misc_float);
        if (!u.freeze) {
            output_colour = last_frame_col.rgb + (output_colour - last_frame_col.rgb) / samples;
        } else {
            output_colour = last_frame_col.rgb;
        }
    } else {
        output_colour = vec3<f32>(0.2);
    }

    if (!u.freeze) {
        // store colour for next frame
        let texture_pos = vec2<i32>(frag_pos.xy);
        textureStore(screen_texture, texture_pos, 0, vec4<f32>(output_colour.rgb, samples));
        textureStore(screen_texture, texture_pos, 1, hit.pos.xyzz);
    }

    if (u.show_ray_steps) {
        output_colour = vec3<f32>(f32(steps) / 100.0);
    }

    let knee = 0.2;
    let power = 2.2;
    output_colour = clamp(output_colour, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>((1.0 - knee) * pow(output_colour, vec3<f32>(power)) + knee * output_colour, 1.0);
}
