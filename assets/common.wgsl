fn get_clip_space(frag_pos: vec4<f32>, dimensions: vec2<f32>) -> vec2<f32> {
    var clip_space = frag_pos.xy / dimensions * 2.0;
    clip_space = clip_space - 1.0;
    clip_space = clip_space * vec2<f32>(1.0, -1.0);
    return clip_space;
}

let k: u32 = 1103515245u;

fn hash(x: vec3<u32>) -> vec3<f32> {
    let x = ((x >> vec3<u32>(8u)) ^ x.yzx) * k;
    let x = ((x >> vec3<u32>(8u)) ^ x.yzx) * k;
    let x = ((x >> vec3<u32>(8u)) ^ x.yzx) * k;
    
    return vec3<f32>(x) * (1.0 / f32(0xffffffffu));
}

// fn rand(n: vec2<f32>) -> f32 {
// 	// return fract(sin(dot(n.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453) * 2.0 - 1.0;
//     var v = 0.0;
//     for (var k = 0; k < 9; k = k + 1) {
//         v = v + hash(n + vec2<f32>(f32(k) % 3.0 - 1.0, f32(k) / 3.0 - 1.0));
//     }
//     return 0.9 * (1.125 * hash(n) - v / 8.0) + 0.5;
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

fn in_bounds(v: vec3<f32>) -> bool {
    let s = step(vec3<f32>(-1.0), v) - step(vec3<f32>(1.0), v);
    return (s.x * s.y * s.z) > 0.5;
}