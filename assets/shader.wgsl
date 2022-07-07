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
    out.clip_position = view.view_proj * world_position;
    out.uv = vertex.uv;
    return out;
}

struct Uniforms {
    resolution: vec2<f32>;
};

struct GH {
    data: [[stride(4)]] array<u32>;
};

[[group(2), binding(0)]]
var<uniform> u: Uniforms;
// [[group(1), binding(1)]]
// var<storage, read_write> gh: GH; // nodes

fn get_clip_space(frag_pos: vec4<f32>, dimensions: vec2<f32>) -> vec2<f32> {
    var clip_space = frag_pos.xy / dimensions * 2.0;
    clip_space = clip_space - 1.0;
    clip_space = clip_space * vec2<f32>(1.0, -1.0);
    return clip_space;
}

// fn get_value(index: u32) -> bool {
//     return ((gh.data[index / 32u] >> (index % 32u)) & 1u) != 0u;
// }

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

[[stage(fragment)]]
fn fragment([[builtin(position)]] frag_pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
    var output_colour = vec3<f32>(0.0, 0.0, 0.0);
    let clip_space = get_clip_space(frag_pos, u.resolution);

    // let screen = floor((clip_space / 2.0 + 0.5) * 8.0);
    // let index = screen.y * 8.0 + screen.x;
    // output_colour = vec3<f32>(f32(get_value(u32(index))));

    output_colour = vec3<f32>(clip_space, 0.5);

    let knee = 0.2;
    let power = 2.2;
    output_colour = clamp(output_colour, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>((1.0 - knee) * pow(output_colour, vec3<f32>(power)) + knee * output_colour, 1.0);
}
