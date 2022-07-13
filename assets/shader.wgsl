#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

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

struct Uniforms {
    resolution: vec2<f32>;
    camera: mat4x4<f32>;
    camera_inverse: mat4x4<f32>;
};

struct GH {
    data: [[stride(4)]] array<u32>;
};

[[group(2), binding(0)]]
var<uniform> u: Uniforms;
[[group(2), binding(1)]]
var<storage, read_write> gh: GH; // nodes

let LEVELS: vec4<u32> = vec4<u32>(128u, 64u, 32u, 16u);
let OFFSETS: vec4<u32> = vec4<u32>(0u, 2097152u, 2359296u, 2392064u);

fn get_clip_space(frag_pos: vec4<f32>, dimensions: vec2<f32>) -> vec2<f32> {
    var clip_space = frag_pos.xy / dimensions * 2.0;
    clip_space = clip_space - 1.0;
    clip_space = clip_space * vec2<f32>(1.0, -1.0);
    return clip_space;
}

fn get_value_index(index: u32) -> bool {
    return ((gh.data[index / 32u] >> (index % 32u)) & 1u) != 0u;
}

struct Voxel {
    value: bool;
    pos: vec3<f32>;
};

fn get_value(pos: vec3<f32>, level: u32) -> Voxel {
    let size = LEVELS[level];
    let scaled = pos * 0.5 + 0.5;
    let scaled = scaled * vec3<f32>(f32(size));
    let scaled = vec3<u32>(scaled);
    let index = scaled.x * size * size + scaled.y * size + scaled.z;

    let value = get_value_index(index + OFFSETS[level]);
    let pos = (floor(pos * f32(size) * 0.5) + 0.5) / (f32(size) * 0.5);
    return Voxel(value, pos);
}

struct Ray {
    pos: vec3<f32>;
    dir: vec3<f32>;
};

fn ray_box_dist(r: Ray, vmin: vec3<f32>, vmax: vec3<f32>) -> f32 {
    let v1 = (vmin.x - r.pos.x) / r.dir.x;
    let v2 = (vmax.x - r.pos.x) / r.dir.x;
    let v3 = (vmin.y - r.pos.y) / r.dir.y;
    let v4 = (vmax.y - r.pos.y) / r.dir.y;
    let v5 = (vmin.z - r.pos.z) / r.dir.z;
    let v6 = (vmax.z - r.pos.z) / r.dir.z;
    let v7 = max(max(min(v1, v2), min(v3, v4)), min(v5, v6));
    let v8 = min(min(max(v1, v2), max(v3, v4)), max(v5, v6));
    if (v8 < 0.0 || v7 > v8) {
        return 0.0;
    }
    
    return v7;
}

fn in_bounds(v: vec3<f32>) -> bool {
    let s = step(vec3<f32>(-1.0), v) - step(vec3<f32>(1.0), v);
    return (s.x * s.y * s.z) > 0.5; 
}

struct HitInfo {
    hit: bool;
    pos: vec3<f32>;
    normal: vec3<f32>;
    steps: u32;
};

fn shoot_ray(r: Ray, cs: vec2<f32>) -> HitInfo {
    var pos = r.pos;
    let dir_mask = vec3<f32>(r.dir == vec3<f32>(0.0));
    var dir = r.dir + dir_mask * 0.000001;

    // return HitInfo(false, vec3<f32>(0.0), pos, 0u);

    if (!in_bounds(r.pos)) {
        // Get position on surface of the octree
        let dist = ray_box_dist(r, vec3<f32>(-1.0), vec3<f32>(1.0));
        if (dist == 0.0){
            return HitInfo(false, vec3<f32>(0.0), vec3<f32>(0.0), 0u);
        }

        pos = r.pos + dir * (dist + 0.00001);
        // return HitInfo(false, vec3<f32>(0.0), vec3<f32>(0.0, 0.0, 1.0), 0u);
    }

    let r_sign = sign(dir);

    var voxel_pos = pos;
    var steps = 0u;
    var normal = trunc(pos * 1.00001);
    loop {
        var level = u32((cs.x * 0.5 + 0.5) * 4.0);

        let voxel = get_value(voxel_pos, level);
        if (voxel.value) {
            break;
        }

        let voxel_size = 2.0 / f32(LEVELS[level]);
        let t_max = (voxel.pos - pos + r_sign * voxel_size / 2.0) / dir;

        // https://www.shadertoy.com/view/4dX3zl (good old shader toy)
        let mask = vec3<f32>(t_max.xyz <= min(t_max.yzx, t_max.zxy));
        normal = mask * -r_sign;

        let t_current = min(min(t_max.x, t_max.y), t_max.z);
        voxel_pos = pos + dir * t_current - normal * 0.000002;

        if (!in_bounds(voxel_pos)) {
            return HitInfo(false, vec3<f32>(0.0), vec3<f32>(0.0), steps);
        }

        steps = steps + 1u;
        if (steps > 1000u) {
            return HitInfo(true, voxel_pos, normal, steps);
        }
    }

    return HitInfo(true, voxel_pos, normal, steps);
}

[[stage(fragment)]]
fn fragment([[builtin(position)]] frag_pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
    var output_colour = vec3<f32>(0.0, 0.0, 0.0);
    let clip_space = get_clip_space(frag_pos, u.resolution);

    let pos = u.camera_inverse * vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let dir = u.camera_inverse * vec4<f32>(clip_space.x, clip_space.y, -1.0, 1.0);
    let pos = pos.xyz;
    let dir = normalize(dir.xyz - pos);
    var ray = Ray(pos.xyz, dir.xyz);

    output_colour = shoot_ray(ray, clip_space).normal * 0.5 + 0.5;
    // output_colour = vec3<f32>(f32(shoot_ray(ray, 512u).steps));

    // let screen = floor((clip_space / 2.0 + 0.5) * 8.0);
    // let index = screen.y * 8.0 + screen.x;
    // output_colour = vec3<f32>(f32(get_value(u32(index))));

    let knee = 0.2;
    let power = 2.2;
    output_colour = clamp(output_colour, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>((1.0 - knee) * pow(output_colour, vec3<f32>(power)) + knee * output_colour, 1.0);
}
