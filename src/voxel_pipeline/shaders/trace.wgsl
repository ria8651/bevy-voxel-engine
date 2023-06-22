#import bevy_core_pipeline::fullscreen_vertex_shader
#import bevy_voxel_engine::common

@group(0) @binding(0)
var<uniform> voxel_uniforms: VoxelUniforms;
@group(0) @binding(1)
var voxel_world: texture_storage_3d<r16uint, read_write>;
@group(0) @binding(2)
var<storage, read_write> gh: array<u32>;
@group(0) @binding(3)
var mip: texture_3d<f32>;
@group(0) @binding(4)
var texture_sampler: sampler;

@group(1) @binding(0)
var<uniform> trace_uniforms: TraceUniforms;
@group(1) @binding(2)
var normal: texture_storage_2d<rgba16float, read_write>;
@group(1) @binding(3)
var position: texture_storage_2d<rgba32float, read_write>;

// note: raytracing.wgsl requires common.wgsl and for you to define u, voxel_world and gh before you import it
#import bevy_voxel_engine::raytracing

const light_dir = vec3<f32>(0.8, -1.0, 0.8);
const light_colour = vec3<f32>(1.0, 1.0, 1.0);

fn calculate_direct(material: vec4<f32>, pos: vec3<f32>, normal: vec3<f32>, seed: vec3<u32>, shadow_samples: u32) -> vec3<f32> {
    var lighting = vec3(0.0);
    if material.a == 0.0 {
        // diffuse
        let diffuse = max(dot(normal, -normalize(light_dir)), 0.0);

        // shadow
        var shadow = 1.0;
        if trace_uniforms.shadows != 0u {
            if trace_uniforms.indirect_lighting != 0u {
                for (var i = 0u; i < shadow_samples; i += 1u) {
                    let rand = hash(seed + i) * 2.0 - 1.0;
                    let shadow_ray = Ray(pos, -light_dir + rand * 0.1);
                    let shadow_hit = shoot_ray(shadow_ray, 0.0, 0u);
                    shadow -= f32(shadow_hit.hit) / f32(shadow_samples);
                }
            } else {
                // let shadow_ray = Ray(pos, -light_dir);
                // let shadow_hit = shoot_ray(shadow_ray, 0.0, 0u);
                // shadow = f32(!shadow_hit.hit);

                let shadow_ray = Ray(pos, -light_dir);
                let col = vct(shadow_ray, 0.1);
                shadow = 1.0 - col.a;
            }
        }

        lighting = diffuse * shadow * light_colour;
    } else {
        lighting = vec3(material.rgb);
    }
    return lighting;
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

fn vct(ray: Ray, angle: f32) -> vec4<f32> {
    var color = vec4(0.0);
    var steps = 0u;
    // var distance = 0.025;
    var distance = 0.3 * angle;
    var tcpotr = ray.pos + ray.dir * distance;
    tcpotr = tcpotr * VOXELS_PER_METER / f32(voxel_uniforms.texture_size) + 0.5;
    loop {
        let size = distance * tan(angle);
        let mip_level = log2(size * f32(voxel_uniforms.texture_size));

        let col = textureSampleLevel(mip, texture_sampler, tcpotr.zyx, mip_level);
        color += col;
        if color.a > 1.0 {
            color.a = 1.0;
            break;
        }

        tcpotr += ray.dir * size;
        distance += size;
        if any(tcpotr < vec3(0.0)) || any(tcpotr >= vec3(1.0)) {
            break;
        }

        steps += 1u;
        if steps > 200u {
            break;
        }
    }

    return color;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let seed = vec3<u32>(in.position.xyz) * 100u + u32(trace_uniforms.time * 120.0) * 15236u;
    let resolution = vec2<f32>(textureDimensions(normal));
    var jitter = vec2(0.0);
    // if (trace_uniforms.indirect_lighting != 0u) {
    //     jitter = (hash(seed).xy - 0.5) / resolution;
    // }
    var clip_space = vec2(1.0, -1.0) * ((in.uv + jitter) * 2.0 - 1.0);
    var output_colour = vec3(0.0);

    let pos1 = trace_uniforms.camera_inverse * vec4(clip_space.x, clip_space.y, 1.0, 1.0);
    let dir1 = trace_uniforms.camera_inverse * vec4(clip_space.x, clip_space.y, 0.01, 1.0);
    let pos = pos1.xyz / pos1.w;
    let dir = normalize(dir1.xyz / dir1.w - pos);
    var ray = Ray(pos, dir);

    let hit = shoot_ray(ray, 0.0, 0u);
    var steps = hit.steps;

    var samples = 0.0;
    if hit.hit {
        // direct lighting
        let direct_lighting = calculate_direct(hit.material, hit.pos, hit.normal, seed + 1u, trace_uniforms.samples);

        // indirect lighting
        var indirect_lighting = vec3(0.0);
        if trace_uniforms.indirect_lighting != 0u {
            // raytraced indirect lighting
            for (var i = 0u; i < trace_uniforms.samples; i += 1u) {
                let indirect_dir = cosine_hemisphere(hit.normal, seed + i);
                let indirect_hit = shoot_ray(Ray(hit.pos, indirect_dir), 0.0, 0u);
                var lighting = vec3(0.0);
                if indirect_hit.hit {
                    lighting = calculate_direct(indirect_hit.material, indirect_hit.pos, indirect_hit.normal, seed + 3u, 1u);
                } else {
                    lighting = vec3(0.2);
                    // lighting = skybox(indirect_dir, 10.0);
                }
                indirect_lighting += lighting / f32(trace_uniforms.samples);
            }
        } else {
            // // aproximate indirect with ambient and voxel ao
            // let texture_coords = hit.pos * VOXELS_PER_METER + f32(voxel_uniforms.texture_size) / 2.0;
            // let ao = voxel_ao(texture_coords, hit.normal.zxy, hit.normal.yzx);
            // let uv = glmod(vec2(dot(hit.normal * texture_coords.yzx, vec3(1.0)), dot(hit.normal * texture_coords.zxy, vec3(1.0))), vec2(1.0));

            // let interpolated_ao_pweig = mix(mix(ao.z, ao.w, uv.x), mix(ao.y, ao.x, uv.x), uv.y);
            // let interpolated_ao = pow(interpolated_ao_pweig, 1.0 / 3.0);

            // indirect_lighting = vec3(interpolated_ao * 0.3);
        
            var color = vct(Ray(hit.pos, vec3(0.0, 1.0, 0.0)), 0.5);
            color += vct(Ray(hit.pos, vec3(cos(0.5) * cos(1.257 * 0.0), sin(0.5), sin(1.257 * 0.0))), 0.5);
            color += vct(Ray(hit.pos, vec3(cos(0.5) * cos(1.257 * 1.0), sin(0.5), sin(1.257 * 1.0))), 0.5);
            color += vct(Ray(hit.pos, vec3(cos(0.5) * cos(1.257 * 2.0), sin(0.5), sin(1.257 * 2.0))), 0.5);
            color += vct(Ray(hit.pos, vec3(cos(0.5) * cos(1.257 * 3.0), sin(0.5), sin(1.257 * 3.0))), 0.5);
            color += vct(Ray(hit.pos, vec3(cos(0.5) * cos(1.257 * 4.0), sin(0.5), sin(1.257 * 4.0))), 0.5);
            color /= 6.0;
            // indirect_lighting = color.rgb / color.a / 100.0;
            indirect_lighting = vec3(0.3 * (1.0 - color.a));
        }

        // final blend
        output_colour = (indirect_lighting + direct_lighting) * hit.material.rgb;

        // let posing = (hit.pos - hit.normal * 0.01) * VOXELS_PER_METER / f32(voxel_uniforms.texture_size) + 0.5;
        // output_colour = textureSampleLevel(mip, texture_sampler, posing.zyx, (1.0 - trace_uniforms.misc_float) * f32(textureNumLevels(mip))).rgb;
    } else {
        // output_colour = vec3<f32>(0.2);
        output_colour = skybox(ray.dir, 10.0);
    }

    if trace_uniforms.show_ray_steps != 0u {
        output_colour = vec3<f32>(f32(steps) / 100.0);
    }

    output_colour = max(output_colour, vec3(0.0));
    textureStore(normal, vec2<i32>(in.position.xy), vec4(hit.normal, 0.0));
    textureStore(position, vec2<i32>(in.position.xy), vec4(hit.reprojection_pos, 0.0));
    return vec4<f32>(output_colour, 1.0);
}