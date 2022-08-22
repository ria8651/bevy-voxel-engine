struct Portal {
    pos: vec3<f32>,
    other_pos: vec3<f32>,
    normal: vec3<f32>,
    other_normal: vec3<f32>,
}

struct Uniforms {
    materials: array<vec4<f32>, 256>,
    portals: array<Portal, 32>,
    resolution: vec2<f32>,
    last_camera: mat4x4<f32>,
    camera: mat4x4<f32>,
    camera_inverse: mat4x4<f32>,
    levels: array<vec4<u32>, 2>,
    offsets: array<vec4<u32>, 2>,
    time: f32,
    texture_size: u32,
    show_ray_steps: u32,
    indirect_lighting: u32,
    shadows: u32,
    accumulation_frames: f32,
    freeze: u32,
    enanble_compute: u32,
    skybox: u32,
    misc_bool: u32,
    misc_float: f32,
};

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

let pi = 3.14159265359;

fn cosine_hemisphere(n: vec3<f32>, seed: vec3<u32>) -> vec3<f32> {
    let u = hash(seed);

    let r = sqrt(u.x);
    let theta = 2.0 * pi * u.y;
 
    let  b = normalize(cross(n, vec3<f32>(0.0, 1.0, 1.0)));
	let  t = cross(b, n);
    
    return normalize(r * sin(theta) * b + sqrt(1.0 - u.x) * n + r * cos(theta) * t);
}

struct Ray {
    pos: vec3<f32>,
    dir: vec3<f32>,
};

// returns the closest intersection and the furthest intersection
fn ray_box_dist(r: Ray, vmin: vec3<f32>, vmax: vec3<f32>) -> vec2<f32> {
    let v1 = (vmin.x - r.pos.x) / r.dir.x;
    let v2 = (vmax.x - r.pos.x) / r.dir.x;
    let v3 = (vmin.y - r.pos.y) / r.dir.y;
    let v4 = (vmax.y - r.pos.y) / r.dir.y;
    let v5 = (vmin.z - r.pos.z) / r.dir.z;
    let v6 = (vmax.z - r.pos.z) / r.dir.z;
    let v7 = max(max(min(v1, v2), min(v3, v4)), min(v5, v6));
    let v8 = min(min(max(v1, v2), max(v3, v4)), max(v5, v6));
    if (v8 < 0.0 || v7 > v8) {
        return vec2(0.0);
    }

    return vec2(v7, v8);
}

fn in_bounds(v: vec3<f32>) -> bool {
    let s = step(vec3<f32>(-1.0), v) - step(vec3<f32>(1.0), v);
    return (s.x * s.y * s.z) > 0.5;
}

fn skybox(dir: vec3<f32>, time_of_day: f32) -> vec3<f32> {
    var dir1: vec3<f32>;
    var time_of_day1: f32;
    var t: f32;
    var sun_pos: vec3<f32>;
    var col: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    var p_sunset_dark: array<vec3<f32>,4u> = array<vec3<f32>,4u>(vec3<f32>(0.3720705509185791, 0.3037080764770508, 0.26548632979393005), vec3<f32>(0.4461638331413269, 0.3940589129924774, 0.42567673325538635), vec3<f32>(0.16514907777309418, 0.4046129286289215, 0.8799446225166321), vec3<f32>(-0.00000000000000007057075514128395, -0.08647964149713516, -0.26904296875));
    var p_sunset_bright: array<vec3<f32>,4u> = array<vec3<f32>,4u>(vec3<f32>(0.38976746797561646, 0.3156035840511322, 0.27932655811309814), vec3<f32>(1.2874523401260376, 1.0100154876708984, 0.8623254299163818), vec3<f32>(0.1260504275560379, 0.23134452104568481, 0.5261799693107605), vec3<f32>(-0.09298685193061829, -0.0733446329832077, -0.19287726283073425));
    var p_day: array<vec3<f32>,4u> = array<vec3<f32>,4u>(vec3<f32>(0.05101049691438675, 0.0975874736905098, 0.14233364164829254), vec3<f32>(0.721604585647583, 0.8130766749382019, 0.9907063245773315), vec3<f32>(0.23738746345043182, 0.6037047505378723, 1.279274582862854), vec3<f32>(-0.000000000000000483417267228435, 0.13545893132686615, -0.0000000000000014694301099188));
    var brightness_a: f32;
    var brightness_d: f32;
    var p_sunset: array<vec3<f32>,4u>;
    var sun_a: f32;
    var sun_d: f32;
    var a: vec3<f32>;
    var b: vec3<f32>;
    var c: vec3<f32>;
    var d: vec3<f32>;
    var sky_a: f32;
    var sky_d: f32;
    var sun_a1: f32;
    var sun_col: vec3<f32>;

    dir1 = dir;
    time_of_day1 = time_of_day;
    let e4: f32 = time_of_day1;
    t = ((e4 + 4.0) * ((360.0 / 24.0) * 0.017453292519943295));
    let e19: f32 = t;
    let e23: f32 = t;
    let e28: f32 = t;
    let e32: f32 = t;
    sun_pos = normalize(vec3<f32>(0.0, -(sin(e28)), cos(e32)));
    {
        let e104: vec3<f32> = dir1;
        let e105: vec3<f32> = sun_pos;
        let e109: vec3<f32> = dir1;
        let e110: vec3<f32> = sun_pos;
        brightness_a = acos(dot(e109, e110));
        let e132: f32 = brightness_a;
        brightness_d = ((1.5 * smoothstep((80.0 * 0.017453292519943295), (0.0 * 0.017453292519943295), e132)) - 0.5);
        let e139: array<vec3<f32>,4u> = p_sunset_dark;
        let e142: array<vec3<f32>,4u> = p_sunset_bright;
        let e146: array<vec3<f32>,4u> = p_sunset_dark;
        let e149: array<vec3<f32>,4u> = p_sunset_bright;
        let e151: f32 = brightness_d;
        let e155: array<vec3<f32>,4u> = p_sunset_dark;
        let e158: array<vec3<f32>,4u> = p_sunset_bright;
        let e162: array<vec3<f32>,4u> = p_sunset_dark;
        let e165: array<vec3<f32>,4u> = p_sunset_bright;
        let e167: f32 = brightness_d;
        let e171: array<vec3<f32>,4u> = p_sunset_dark;
        let e174: array<vec3<f32>,4u> = p_sunset_bright;
        let e178: array<vec3<f32>,4u> = p_sunset_dark;
        let e181: array<vec3<f32>,4u> = p_sunset_bright;
        let e183: f32 = brightness_d;
        let e187: array<vec3<f32>,4u> = p_sunset_dark;
        let e190: array<vec3<f32>,4u> = p_sunset_bright;
        let e194: array<vec3<f32>,4u> = p_sunset_dark;
        let e197: array<vec3<f32>,4u> = p_sunset_bright;
        let e199: f32 = brightness_d;
        p_sunset = array<vec3<f32>,4u>(mix(e146[0], e149[0], vec3<f32>(e151)), mix(e162[1], e165[1], vec3<f32>(e167)), mix(e178[2], e181[2], vec3<f32>(e183)), mix(e194[3], e197[3], vec3<f32>(e199)));
        let e209: vec3<f32> = sun_pos;
        let e220: vec3<f32> = sun_pos;
        sun_a = acos(dot(e220, vec3<f32>(0.0, 1.0, 0.0)));
        let e245: f32 = sun_a;
        sun_d = smoothstep((100.0 * 0.017453292519943295), (60.0 * 0.017453292519943295), e245);
        let e249: array<vec3<f32>,4u> = p_sunset;
        let e252: array<vec3<f32>,4u> = p_day;
        let e256: array<vec3<f32>,4u> = p_sunset;
        let e259: array<vec3<f32>,4u> = p_day;
        let e261: f32 = sun_d;
        a = mix(e256[0], e259[0], vec3<f32>(e261));
        let e266: array<vec3<f32>,4u> = p_sunset;
        let e269: array<vec3<f32>,4u> = p_day;
        let e273: array<vec3<f32>,4u> = p_sunset;
        let e276: array<vec3<f32>,4u> = p_day;
        let e278: f32 = sun_d;
        b = mix(e273[1], e276[1], vec3<f32>(e278));
        let e283: array<vec3<f32>,4u> = p_sunset;
        let e286: array<vec3<f32>,4u> = p_day;
        let e290: array<vec3<f32>,4u> = p_sunset;
        let e293: array<vec3<f32>,4u> = p_day;
        let e295: f32 = sun_d;
        c = mix(e290[2], e293[2], vec3<f32>(e295));
        let e300: array<vec3<f32>,4u> = p_sunset;
        let e303: array<vec3<f32>,4u> = p_day;
        let e307: array<vec3<f32>,4u> = p_sunset;
        let e310: array<vec3<f32>,4u> = p_day;
        let e312: f32 = sun_d;
        d = mix(e307[3], e310[3], vec3<f32>(e312));
        let e321: vec3<f32> = dir1;
        let e332: vec3<f32> = dir1;
        sky_a = acos(dot(e332, vec3<f32>(0.0, 1.0, 0.0)));
        let e357: f32 = sky_a;
        sky_d = smoothstep((90.0 * 0.017453292519943295), (60.0 * 0.017453292519943295), e357);
        let e360: vec3<f32> = col;
        let e361: vec3<f32> = b;
        let e362: vec3<f32> = d;
        let e365: f32 = sky_d;
        let e367: vec3<f32> = c;
        let e377: vec3<f32> = a;
        let e382: f32 = sky_d;
        let e384: vec3<f32> = c;
        let e394: vec3<f32> = a;
        let e400: vec3<f32> = d;
        col = (e360 + (((e361 - e362) * sin((vec3<f32>(1.0) / (((vec3<f32>(e382) / e384) + vec3<f32>((2.0 / (180.0 * 0.017453292519943295)))) - e394)))) + e400));
    }
    let e405: vec3<f32> = sun_pos;
    let e406: vec3<f32> = dir1;
    let e410: vec3<f32> = sun_pos;
    let e411: vec3<f32> = dir1;
    sun_a1 = acos(dot(e410, e411));
    let e421: f32 = sun_a1;
    sun_col = ((0.009999999776482582 * vec3<f32>(1.0, 0.949999988079071, 0.949999988079071)) / vec3<f32>(e421));
    let e425: vec3<f32> = col;
    let e427: vec3<f32> = sun_col;
    let e431: vec3<f32> = col;
    let e433: vec3<f32> = sun_col;
    let e436: vec3<f32> = sun_col;
    col = max((e431 + (0.5 * e433)), e436);
    let e438: vec3<f32> = col;
    return e438;
}

fn rotate_axis(p: vec3<f32>, axis: vec3<f32>, angle: f32) -> vec3<f32> {
    return mix(dot(axis, p) * axis, p, cos(angle)) + cross(axis, p) * sin(angle);
}

fn create_rot_mat(axis: vec3<f32>, angle: f32) -> mat3x3<f32> {
    var axis1: vec3<f32>;
    var angle1: f32;
    var s: f32;
    var c: f32;
    var oc: f32;

    axis1 = axis;
    angle1 = angle;
    let e5: vec3<f32> = axis1;
    axis1 = normalize(e5);
    let e8: f32 = angle1;
    s = sin(e8);
    let e12: f32 = angle1;
    c = cos(e12);
    let e16: f32 = c;
    oc = (1.0 - e16);
    let e19: f32 = oc;
    let e20: vec3<f32> = axis1;
    let e23: vec3<f32> = axis1;
    let e26: f32 = c;
    let e28: f32 = oc;
    let e29: vec3<f32> = axis1;
    let e32: vec3<f32> = axis1;
    let e35: vec3<f32> = axis1;
    let e37: f32 = s;
    let e40: f32 = oc;
    let e41: vec3<f32> = axis1;
    let e44: vec3<f32> = axis1;
    let e47: vec3<f32> = axis1;
    let e49: f32 = s;
    let e52: f32 = oc;
    let e53: vec3<f32> = axis1;
    let e56: vec3<f32> = axis1;
    let e59: vec3<f32> = axis1;
    let e61: f32 = s;
    let e64: f32 = oc;
    let e65: vec3<f32> = axis1;
    let e68: vec3<f32> = axis1;
    let e71: f32 = c;
    let e73: f32 = oc;
    let e74: vec3<f32> = axis1;
    let e77: vec3<f32> = axis1;
    let e80: vec3<f32> = axis1;
    let e82: f32 = s;
    let e85: f32 = oc;
    let e86: vec3<f32> = axis1;
    let e89: vec3<f32> = axis1;
    let e92: vec3<f32> = axis1;
    let e94: f32 = s;
    let e97: f32 = oc;
    let e98: vec3<f32> = axis1;
    let e101: vec3<f32> = axis1;
    let e104: vec3<f32> = axis1;
    let e106: f32 = s;
    let e109: f32 = oc;
    let e110: vec3<f32> = axis1;
    let e113: vec3<f32> = axis1;
    let e116: f32 = c;
    return mat3x3<f32>(vec3<f32>((((e19 * e20.x) * e23.x) + e26), (((e28 * e29.x) * e32.y) - (e35.z * e37)), (((e40 * e41.z) * e44.x) + (e47.y * e49))), vec3<f32>((((e52 * e53.x) * e56.y) + (e59.z * e61)), (((e64 * e65.y) * e68.y) + e71), (((e73 * e74.y) * e77.z) - (e80.x * e82))), vec3<f32>((((e85 * e86.z) * e89.x) - (e92.y * e94)), (((e97 * e98.y) * e101.z) + (e104.x * e106)), (((e109 * e110.z) * e113.z) + e116)));
}

// let DEPOLARIZATION_FACTOR: f32 = 0.035;
// let MIE_COEFFICIENT: f32 = 0.005;
// let MIE_DIRECTIONAL_G: f32 = 0.8;
// let MIE_K_COEFFICIENT: vec3<f32> = vec3<f32>(0.686, 0.678, 0.666);
// let MIE_V: f32 = 4.0;
// let MIE_ZENITH_LENGTH: f32 = 1.25e3;
// let NUM_MOLECULES: f32 = 2.542e25;
// let PRIMARIES: vec3<f32> = vec3<f32>(6.8e-7, 5.5e-7, 4.5e-7);
// let RAYLEIGH: f32 = 1.0;
// let RAYLEIGH_ZENITH_LENGTH: f32 = 8.4e3;
// let REFRACTIVE_INDEX: f32 = 1.0003;
// let SUN_ANGULAR_DIAMETER_DEGREES: f32 = 0.0093333;
// let SUN_INTENSITY_FACTOR: f32 = 1000.0;
// let SUN_INTENSITY_FALLOFF_STEEPNESS: f32 = 1.5;
// let TURBIDITY: f32 = 2.0;
// let PI: f32 = 3.141592653589793;

// fn total_rayleigh(lambda: vec3<f32>) -> vec3<f32> {
//     return (8.0 * pow(PI, 3.0)
//         * pow(pow(REFRACTIVE_INDEX, 2.0) - 1.0, 2.0)
//         * (6.0 + 3.0 * DEPOLARIZATION_FACTOR))
//         / (3.0 * NUM_MOLECULES * pow(lambda, vec3(4.0)) * (6.0 - 7.0 * DEPOLARIZATION_FACTOR));
// }

// fn total_mie(lambda: vec3<f32>, k: vec3<f32>, t: f32) -> vec3<f32> {
//     let c = 0.2 * t * 10e-18;
//     return vec3(0.434 * c * PI * pow((2.0 * PI) / lambda, vec3(MIE_V - 2.0)) * k);
// }

// fn rayleigh_phase(cos_theta: f32) -> f32 {
//     return (3.0 / (16.0 * PI)) * (1.0 + pow(cos_theta, 2.0));
// }

// fn henyey_greenstein_phase(cos_theta: f32, g: f32) -> f32 {
//     return (1.0 / (4.0 * PI)) * ((1.0 - pow(g, 2.0))) / pow(1.0 - 2.0 * g * cos_theta + pow(g, 2.0), 1.5);
// }

// fn sun_intensity(zenith_angle_cos: f32) -> f32 {
//     let cutoff_angle = PI / 1.95; // Earth shadow hack
//     return SUN_INTENSITY_FACTOR
//         * max(0.0,
//             1.0 - exp(-((cutoff_angle - acos(zenith_angle_cos))
//                 / SUN_INTENSITY_FALLOFF_STEEPNESS))
//         );
// }

// fn saturate(x: f32) -> f32 {
//     return clamp(x, 0.0, 1.0);
// }

// fn sky(dir: vec3<f32>, sun_position: vec3<f32>) -> vec3<f32> {
//     let up = vec3(0.0, 1.0, 0.0);
//     let sunfade = 1.0 - exp(1.0 - saturate(sun_position.y / 450000.0));
//     let rayleigh_coefficient = RAYLEIGH - (1.0 * (1.0 - sunfade));
//     let beta_r = total_rayleigh(PRIMARIES) * rayleigh_coefficient;

//     // Mie coefficient
//     let beta_m = total_mie(PRIMARIES, MIE_K_COEFFICIENT, TURBIDITY) * MIE_COEFFICIENT;

//     // Optical length, cutoff angle at 90 to avoid singularity
//     let zenith_angle = acos(max(dot(up, dir), 0.0));
//     let denom = cos(zenith_angle) + 0.15 * pow(93.885 - ((zenith_angle * 180.0) / PI), -1.253);

//     let s_r = RAYLEIGH_ZENITH_LENGTH / denom;
//     let s_m = MIE_ZENITH_LENGTH / denom;

//     // Combined extinction factor
//     let fex = exp(-(beta_r * s_r + beta_m * s_m));

//     // In-scattering
//     let sun_direction = normalize(sun_position);
//     let cos_theta = dot(dir, sun_direction);
//     let beta_r_theta = beta_r * rayleigh_phase(cos_theta * 0.5 + 0.5);

//     let beta_m_theta = beta_m * henyey_greenstein_phase(cos_theta, MIE_DIRECTIONAL_G);
//     let sun_e = sun_intensity(dot(sun_direction, up));
//     var lin = pow(
//         sun_e * ((beta_r_theta + beta_m_theta) / (beta_r + beta_m)) * (vec3(1.0) - fex),
//         vec3(1.5)
//     );

//     let t = saturate(pow(1.0 - dot(up, sun_direction), 5.0));
//     lin *= vec3(1.0) * (vec3(1.0) - t) + pow(
//             sun_e * ((beta_r_theta + beta_m_theta) / (beta_r + beta_m)) * fex,
//             vec3(0.5),
//         ) * t;

//     // Composition + solar disc
//     let sun_angular_diameter_cos = cos(SUN_ANGULAR_DIAMETER_DEGREES);
//     let sundisk = smoothstep(
//         sun_angular_diameter_cos,
//         sun_angular_diameter_cos + 0.00002,
//         cos_theta,
//     );
//     var l0 = 0.1 * fex;
//     l0 = l0 + sun_e * 19000.0 * fex * sundisk;

//     return lin + l0;
// }