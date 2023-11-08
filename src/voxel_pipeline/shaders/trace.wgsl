#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_voxel_engine::common::{
    VOXELS_PER_METER,
    PI,
    VoxelUniforms,
    TraceUniforms,
    Ray,
    skybox
}
#import bevy_voxel_engine::raytracing::{
    shoot_ray,
}
#import bevy_voxel_engine::bindings::{
    voxel_world,
    voxel_uniforms,
    gh
}

@group(0) @binding(3)
var texture_sampler: sampler;

@group(1) @binding(0)
var<uniform> trace_uniforms: TraceUniforms;
@group(1) @binding(1)
var normal: texture_storage_2d<rgba16float, read_write>;
@group(1) @binding(2)
var position: texture_storage_2d<rgba32float, read_write>;

struct DirectLightningInfo {
    color: vec3<f32>,
    shadow: f32,
};

fn calculate_direct(sun_dir: vec3<f32>, sky_color: vec3<f32>, material: vec4<f32>, pos: vec3<f32>, normal: vec3<f32>, seed: vec3<u32>, shadow_samples: u32) -> DirectLightningInfo {
    // Diffuse
    let diffuse = max(dot(normal, -normalize(sun_dir)), 0.0);

    // Shadow
    var shadow = 1.0;

    if trace_uniforms.shadows != 0u {
        let shadow_ray = Ray(pos, -sun_dir);
        let shadow_hit = shoot_ray(shadow_ray, 0.0, 0u);
        shadow = f32(!shadow_hit.hit);
    }

    // Emissive
    var emissive = vec3(0.0);
    if material.a != 0.0 {
        emissive = vec3(material.rgb);
    }

    let color = diffuse * shadow + emissive;

    return DirectLightningInfo(color, shadow);
}

fn get_voxel(pos: vec3<f32>) -> f32 {
    if any(pos < vec3(0.0)) || any(pos >= vec3(f32(voxel_uniforms.texture_size))) {
        return 0.0;
    }

    let voxel = textureLoad(voxel_world, vec3<i32>(pos.zyx));
    return min(f32(voxel.r & 0xFFu), 1.0);
}

// https://www.shadertoy.com/view/ldl3DS
fn vertex_ao(side: vec2<f32>, corner: f32) -> f32 {
    return (side.x + side.y + max(corner, side.x * side.y)) / 10.0;
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

// Calculate the angle between two vectors
fn angle_between_vectors(v1: vec3<f32>, v2: vec3<f32>) -> f32 {
    return acos(dot(v1, v2) / (length(v1) * length(v2)));
}

// Calculate the progressive value based on sun angle
fn calculate_sun_progress(sun_dir: vec3<f32>) -> f32 {
    let up = vec3<f32>(0.0, 1.0, 0.0);

    // Calculate the angle between sun_dir and the "up" vector
    let angle = angle_between_vectors(sun_dir, up);

    // If the sun is below the horizon or exactly at the horizon, return 0
    if (angle > PI / 2.0) {
        return mix(0.2, 1.0, -sun_dir.y);
    }

    return 0.2;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let seed = vec3<u32>(in.position.xyz) * 100u + u32(trace_uniforms.time * 120.0) * 15236u;
    let resolution = vec2<f32>(textureDimensions(normal));
    var clip_space = vec2(1.0, -1.0) * (in.uv * 2.0 - 1.0);
    var output_color = vec3(0.0);

    let pos1 = trace_uniforms.camera_inverse * vec4(clip_space.x, clip_space.y, 1.0, 1.0);
    let dir1 = trace_uniforms.camera_inverse * vec4(clip_space.x, clip_space.y, 0.01, 1.0);
    let pos = pos1.xyz / pos1.w;
    let dir = normalize(dir1.xyz / dir1.w - pos);
    var ray = Ray(pos, dir);

    let hit = shoot_ray(ray, 0.0, 0u);
    var steps = hit.steps;

    let timespan = 0.1;
    let w = clamp((trace_uniforms.time * timespan + 12.0) % 24.0, 0.0, 24.0);
    let skybox_info = skybox(ray.dir, w);

    var samples = 0.0;
    if hit.hit {
        // Direct lighting
        let direct_lighting = calculate_direct(skybox_info.sun_dir, skybox_info.sky_color, hit.material, hit.pos, hit.normal, seed + 1u, trace_uniforms.samples);

        // Indirect lighting
        let texture_coords = hit.pos * VOXELS_PER_METER + f32(voxel_uniforms.texture_size) / 2.0;
        let ao = voxel_ao(texture_coords, hit.normal.zxy, hit.normal.yzx);
        let uv = glmod(vec2(dot(hit.normal * texture_coords.yzx, vec3(1.0)), dot(hit.normal * texture_coords.zxy, vec3(1.0))), vec2(1.0));

        let interpolated_ao_pweig = mix(mix(ao.z, ao.w, uv.x), mix(ao.y, ao.x, uv.x), uv.y);
        let voxel_ao = pow(interpolated_ao_pweig, 1.0 / 3.0);
        let indirect_lighting_color = vec3(0.3 * voxel_ao);

        let sun_progress = calculate_sun_progress(skybox_info.sun_dir);

        output_color = (indirect_lighting_color + direct_lighting.color) * hit.material.rgb * sun_progress;
    } else {
        output_color = skybox_info.sky_color;
    }

    if trace_uniforms.show_ray_steps != 0u {
        output_color = vec3<f32>(f32(steps) / 100.0);
    }

    output_color = max(output_color, vec3(0.0));

    textureStore(normal, vec2<i32>(in.position.xy), vec4(hit.normal, 0.0));
    textureStore(position, vec2<i32>(in.position.xy), vec4(hit.reprojection_pos, 0.0));

    return vec4<f32>(output_color, 1.0);
}