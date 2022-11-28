#import bevy_core_pipeline::fullscreen_vertex_shader
#import bevy_core_pipeline::tonemapping

struct Uniforms {
    offsets: array<vec4<f32>, 25>,
    kernel: array<vec4<f32>, 25>,
}

struct PassData {
    denoise_strength: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var texture_sampler: sampler;
@group(0) @binding(2)
var normal_attachment: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(3)
var position_attachment: texture_storage_2d<rgba16float, read_write>;
@group(1) @binding(0)
var<uniform> pass_data: PassData;
@group(1) @binding(1)
var colour_attachment: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let sample_pos = vec2<i32>(in.position.xy);
    let colour = textureSample(colour_attachment, texture_sampler, in.uv).rgb;
    let normal = textureLoad(normal_attachment, sample_pos).rgb * 2.0 - 1.0;
    let position = textureLoad(position_attachment, sample_pos).rgb;
    var output_colour = vec3(0.0);

    var sum = vec3(0.0);
    var sum_w = vec3(0.0);
    let c_phi = 1.0;
    let n_phi = 0.5;
    let p_phi = 0.00004;

    let denoise_strength = pass_data.denoise_strength;

    for (var i = 0; i < 25; i += 1) {
        let colour_pos = in.uv + denoise_strength * uniforms.offsets[i].xy / vec2<f32>(textureDimensions(colour_attachment));
        let new_colour = textureSample(colour_attachment, texture_sampler, colour_pos).rgb;
        let diff = colour - new_colour;
        let dist2 = dot(diff, diff);
        let colour_weight = min(exp(-dist2 / c_phi), 1.0);

        let normal_pos = sample_pos + vec2<i32>(denoise_strength * uniforms.offsets[i].xy);
        let new_normal = textureLoad(normal_attachment, normal_pos).rgb * 2.0 - 1.0;
        let diff = normal - new_normal;
        let dist2 = dot(diff, diff);
        let normal_weight = min(exp(-dist2 / n_phi), 1.0);
        
        let position_pos = sample_pos + vec2<i32>(denoise_strength * uniforms.offsets[i].xy);
        let new_position = textureLoad(position_attachment, position_pos).rgb;
        let diff = position - new_position;
        let dist2 = dot(diff, diff);
        let position_weight = min(exp(-dist2 / p_phi), 1.0);

        // new denoised frame
        let weight = colour_weight * normal_weight * position_weight;
        sum += new_colour * weight * uniforms.kernel[i].x;
        sum_w += weight * uniforms.kernel[i].x;
    }

    output_colour = sum / sum_w;
    // output_colour = sum;
    // output_colour = colour;

    return vec4<f32>(output_colour, 1.0);
}
