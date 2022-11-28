#import bevy_core_pipeline::fullscreen_vertex_shader
#import "common.wgsl"

@group(0) @binding(0)
var<uniform> voxel_uniforms: VoxelUniforms;
@group(0) @binding(1)
var voxel_world: texture_storage_3d<r16uint, read_write>;
@group(0) @binding(2)
var<storage, read_write> gh: array<u32>;

@group(1) @binding(0)
var<uniform> trace_uniforms: TraceUniforms;
@group(1) @binding(1)
var normal_attachment: texture_storage_2d<rgba8unorm, read_write>;
@group(1) @binding(2)
var position_attachment: texture_storage_2d<rgba16float, read_write>;

// note: raytracing.wgsl requires common.wgsl and for you to define u, voxel_world and gh before you import it
#import "raytracing.wgsl"

let light_dir = vec3<f32>(0.8, -1.0, 0.8);
let light_colour = vec3<f32>(1.0, 1.0, 1.0);

fn calculate_direct(material: vec4<f32>, pos: vec3<f32>, normal: vec3<f32>, seed: vec3<u32>) -> vec3<f32> {
    var lighting = vec3(0.0);
    if (material.a == 0.0) {
        // diffuse
        let diffuse = max(dot(normal, -normalize(light_dir)), 0.0);

        // shadow
        var shadow = 1.0;
        if (trace_uniforms.shadows != 0u) {
            // let rand = hash(seed) * 2.0 - 1.0;
            let rand = vec3(0.0);
            let shadow_ray = Ray(pos, -light_dir + rand * 0.1);
            let shadow_hit = shoot_ray(shadow_ray, 0.0, 0u);
            shadow = f32(!shadow_hit.hit);
        }

        lighting = diffuse * shadow * light_colour;
    } else {
        lighting = vec3(1.0);
    }
    return lighting;
}

fn get_voxel(pos: vec3<f32>) -> f32 {
    if (any(pos < vec3(0.0)) || any(pos >= vec3(f32(voxel_uniforms.texture_size)))) {
        return 0.0;
    }

    let voxel = textureLoad(voxel_world, vec3<i32>(pos.zyx));
    return min(f32(voxel.r & 0xFFu), 1.0);
}

// https://www.shadertoy.com/view/ldl3DS
fn vertex_ao(side: vec2<f32>, corner: f32) -> f32 {
    return (side.x + side.y + max(corner, side.x * side.y)) / 3.1;
}
fn voxel_ao(pos: vec3<f32>, d1: vec3<f32>, d2: vec3<f32>) -> vec4<f32> {
    let side = vec4(get_voxel(pos + d1), get_voxel(pos + d2), get_voxel(pos - d1), get_voxel(pos - d2));
    let corner = vec4(get_voxel(pos + d1 + d2), get_voxel(pos - d1 + d2), get_voxel(pos - d1 - d2), get_voxel(pos + d1 - d2));

    var ao: vec4<f32>;
    ao.x = vertex_ao(side.xy, corner.x);
    ao.y = vertex_ao(side.yz, corner.y);
    ao.z = vertex_ao(side.zw, corner.z);
    ao.w = vertex_ao(side.wx, corner.w);

    return 1.0 - ao;
}
fn glmod(x: vec2<f32>, y: vec2<f32>) -> vec2<f32> {
    return x - y * floor(x / y);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // pixel jitter
    let seed = vec3<u32>(in.position.xyz + trace_uniforms.time * 240.0);
    let jitter = vec4(hash(seed).xy - 0.5, 0.0, 0.0) / 1.1;
    var clip_space = get_clip_space(in.position, trace_uniforms.resolution);
    let aspect = trace_uniforms.resolution.x / trace_uniforms.resolution.y;
    clip_space.x = clip_space.x * aspect;
    var output_colour = vec3(0.0);

    let pos = trace_uniforms.camera_inverse * vec4(0.0, 0.0, 0.0, 1.0);
    let dir = trace_uniforms.camera_inverse * vec4(clip_space.x * trace_uniforms.fov, clip_space.y * trace_uniforms.fov, -1.0, 1.0);
    let pos = pos.xyz;
    let dir = normalize(dir.xyz - pos);
    var ray = Ray(pos, dir);

    let hit = shoot_ray(ray, 0.0, 0u);
    var steps = hit.steps;

    var samples = 0.0;
    if (hit.hit) {
        // direct lighting
        let direct_lighting = calculate_direct(hit.material, hit.pos, hit.normal, seed + 15u);

        // indirect lighting
        var indirect_lighting = vec3(0.0);
        if (trace_uniforms.indirect_lighting != 0u) {
            // raytraced indirect lighting
            let indirect_dir = cosine_hemisphere(hit.normal, seed + 10u);
            let indirect_hit = shoot_ray(Ray(hit.pos, indirect_dir), 0.0, 0u);
            if (indirect_hit.hit) {
                indirect_lighting = calculate_direct(indirect_hit.material, indirect_hit.pos, indirect_hit.normal, seed + 20u);
            } else {
                indirect_lighting = vec3(0.2);
                // indirect_lighting = skybox(indirect_dir, 10.0);
            }
        } else {
            // aproximate indirect with ambient and voxel ao
            let texture_coords = (hit.pos * 0.5 + 0.5) * f32(voxel_uniforms.texture_size);
            let ao = voxel_ao(texture_coords, hit.normal.zxy, hit.normal.yzx);
            let uv = glmod(vec2(dot(hit.normal * texture_coords.yzx, vec3(1.0)), dot(hit.normal * texture_coords.zxy, vec3(1.0))), vec2(1.0));

            let interpolated_ao = mix(mix(ao.z, ao.w, uv.x), mix(ao.y, ao.x, uv.x), uv.y);
            let interpolated_ao = pow(interpolated_ao, 1.0 / 3.0);

            indirect_lighting = vec3(interpolated_ao * 0.3);
        }

        // final blend
        output_colour = (indirect_lighting + direct_lighting) * hit.material.rgb;

        // // reprojection
        // let last_frame_clip_space = u.last_camera * vec4<f32>(hit.pos + hit.portal_offset, 1.0);
        // var last_frame_pos = vec2<f32>(-1.0, 1.0) * (last_frame_clip_space.xy / last_frame_clip_space.z / u.fov);
        // last_frame_pos.x = last_frame_pos.x / aspect;
        // let texture_pos = vec2<i32>((last_frame_pos.xy * 0.5 + 0.5) * u.resolution);

        // var last_frame_col = textureLoad(screen_texture, texture_pos, 0);
        // var last_frame_pos = textureLoad(screen_texture, texture_pos, 1);

        // let last_frame_clip_space_from_texture = u.last_camera * vec4<f32>(last_frame_pos.xyz, 1.0);
        // if (length(last_frame_clip_space.z - last_frame_clip_space_from_texture.z) > 0.001) {
        //     last_frame_col = vec4<f32>(0.0);
        //     last_frame_pos = vec4<f32>(0.0);
        // }
        // if (last_frame_clip_space.z > 0.0) {
        //     last_frame_col = vec4<f32>(0.0);
        //     last_frame_pos = vec4<f32>(0.0);
        // }

        // samples = min(last_frame_col.a + 1.0, u.accumulation_frames);
        // if (u.freeze == 0u) {
        //     output_colour = last_frame_col.rgb + (output_colour - last_frame_col.rgb) / samples;
        // } else {
        //     output_colour = last_frame_col.rgb;
        // }
    } else {
        // output_colour = vec3<f32>(0.2);
        output_colour = skybox(ray.dir, 10.0);
    }

    if (trace_uniforms.freeze == 0u) {
        // store colour for next frame
        // let texture_pos = vec2<i32>(frag_pos.xy);
        // textureStore(screen_texture, texture_pos, 0, vec4(output_colour.rgb, samples));
        // textureStore(screen_texture, texture_pos, 1, vec4(hit.pos + hit.portal_offset, 0.0));
    }

    if (trace_uniforms.show_ray_steps != 0u) {
        output_colour = vec3<f32>(f32(steps) / 100.0);
    }

    // output_colour = (hit.pos + hit.portal_offset) * 2.0;
    // output_colour = hit.pos * 2.0;

    // output_colour = vec3(u.time);

    textureStore(normal_attachment, vec2<i32>(in.uv * trace_uniforms.resolution), vec4(hit.normal * 0.5 + 0.5, 0.0));
    textureStore(position_attachment, vec2<i32>(in.uv * trace_uniforms.resolution), vec4(hit.pos, 0.0));
    return vec4<f32>(max(output_colour, vec3(0.0)), 1.0);
}