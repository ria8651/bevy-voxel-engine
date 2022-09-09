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
var<storage, read> gh: array<u32>;
@group(2) @binding(2)
var texture: texture_storage_3d<r16uint, read>;
@group(2) @binding(3)
var screen_texture: texture_storage_2d_array<rgba16float, read_write>;

// note: raytracing.wgsl requires common.wgsl and for you to define u, gh and texture before you import it
#import "raytracing.wgsl"

let light_dir = vec3<f32>(0.8, -1.0, 0.8);
let light_colour = vec3<f32>(1.0, 1.0, 1.0);

fn calculate_direct(material: vec4<f32>, pos: vec3<f32>, normal: vec3<f32>, seed: vec3<u32>) -> vec3<f32> {
    var lighting = vec3(0.0);
    if (material.a == 0.0) {
        // ambient
        var ambient = vec3(0.0);
        if (!(u.indirect_lighting != 0u)) {
            ambient = vec3(0.3);
        }

        // diffuse
        let diffuse = max(dot(normal, -normalize(light_dir)), 0.0);

        // shadow
        var shadow = 1.0;
        if (u.shadows != 0u) {
            let rand = hash(seed) * 2.0 - 1.0;
            // let rand = vec3(0.0);
            let shadow_ray = Ray(pos, -light_dir + rand * 0.1);
            let shadow_hit = shoot_ray(shadow_ray, 0.0, 0u);
            shadow = f32(!(shadow_hit.hit && any(shadow_hit.material == vec4(0.0))));
        }

        lighting = ambient + diffuse * shadow * light_colour;
    } else {
        lighting = vec3(1.0);
    }
    return lighting;
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
    var ray = Ray(pos, dir);

    let hit = shoot_ray(ray, 0.0, 0u);
    var steps = hit.steps;

    var samples = 0.0;
    if (hit.hit || any(hit.material != vec4(0.0))) {
        // direct lighting
        let direct_lighting = calculate_direct(hit.material, hit.pos, hit.normal, seed + 15u);

        // indirect lighting
        var indirect_lighting = vec3(0.0);
        if (u.indirect_lighting != 0u) {
            let indirect_dir = cosine_hemisphere(hit.normal, seed + 10u);
            let indirect_hit = shoot_ray(Ray(hit.pos, indirect_dir), 0.0, 0u);
            if (indirect_hit.hit) {
                indirect_lighting = calculate_direct(indirect_hit.material, indirect_hit.pos, indirect_hit.normal, seed + 20u);
            } else {
                indirect_lighting = vec3<f32>(0.3);
                // indirect_lighting = skybox(indirect_dir, 10.0);
            }
        }

        // final blend
        output_colour = (direct_lighting + indirect_lighting) * hit.material.rgb;

        // reprojection
        let last_frame_clip_space = u.last_camera * vec4<f32>(hit.pos + hit.portal_offset, 1.0);
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
        // output_colour = vec3<f32>(0.2);
        output_colour = skybox(ray.dir, 10.0);
    }

    if (u.freeze == 0u) {
        // store colour for next frame
        let texture_pos = vec2<i32>(frag_pos.xy);
        textureStore(screen_texture, texture_pos, 0, vec4(output_colour.rgb, samples));
        textureStore(screen_texture, texture_pos, 1, vec4(hit.pos + hit.portal_offset, 0.0));
    }

    if (u.show_ray_steps != 0u) {
        output_colour = vec3<f32>(f32(steps) / 100.0);
    }

    // output_colour = (hit.pos + hit.portal_offset) * 2.0;
    // output_colour = hit.pos * 2.0;

    output_colour = max(output_colour, vec3(0.0));
    // output_colour = aces(output_colour);
    output_colour = pow(output_colour, vec3(2.2));
    return vec4<f32>(output_colour, 1.0);
}