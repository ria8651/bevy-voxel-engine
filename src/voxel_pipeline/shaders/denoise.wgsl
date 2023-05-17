#import bevy_core_pipeline::fullscreen_vertex_shader

struct Uniforms {
    offsets: array<vec4<f32>, 25>,
    kernel: array<vec4<f32>, 25>,
}

struct PassData {
    denoise_strength: f32,
    colour_phi: f32,
    normal_phi: f32,
    position_phi: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(0) @binding(1)
var accumulation_attachment: texture_storage_2d<rgba16float, read_write>;
@group(0) @binding(2)
var normal_attachment: texture_storage_2d<rgba16float, read_write>;
@group(0) @binding(3)
var position_attachment: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(0)
var<uniform> pass_data: PassData;
@group(1) @binding(1)
var colour_attachment: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let sample_pos = vec2<i32>(in.position.xy);
    let colour = textureLoad(colour_attachment, sample_pos, 0).rgb;
    let normal = textureLoad(normal_attachment, sample_pos).rgb;
    let position = textureLoad(position_attachment, sample_pos).rgb;
    var output_colour = colour;

    var sum = vec3(0.0);
    var sum_w = vec3(0.0);
    let denoise_strength = pass_data.denoise_strength;
    let c_phi = pass_data.colour_phi;
    let n_phi = pass_data.normal_phi;
    let p_phi = pass_data.position_phi;

    for (var i = 0; i < 25; i += 1) {
        let new_sample_pos = sample_pos + vec2<i32>(denoise_strength * uniforms.offsets[i].xy);

        let new_colour = textureLoad(colour_attachment, new_sample_pos, 0).rgb;
        let diff = colour - new_colour;
        let dist2 = dot(diff, diff);
        let colour_weight = min(exp(-dist2 / c_phi), 1.0);

        let new_normal = textureLoad(normal_attachment, new_sample_pos).rgb;
        let diff_normal = normal - new_normal;
        let dist2_normal = dot(diff_normal, diff_normal);
        let normal_weight = min(exp(-dist2_normal / n_phi), 1.0);
        
        let new_position = textureLoad(position_attachment, new_sample_pos).rgb;
        let diff_pos = position - new_position;
        let dist2_pos = dot(diff_pos, diff_pos);
        let position_weight = min(exp(-dist2_pos / p_phi), 1.0);

        // new denoised frame
        let weight = colour_weight * normal_weight * position_weight;
        sum += new_colour * weight * uniforms.kernel[i].x;
        sum_w += weight * uniforms.kernel[i].x;
    }

    output_colour = sum / sum_w;
    // output_colour = position;
    // output_colour = sum;
    // output_colour = abs(vec3(uniforms.offsets[5 * i32(in.uv.x * 5.0) + i32(in.uv.y * 5.0)].xyz));

    textureStore(accumulation_attachment, sample_pos, vec4(sum / sum_w, 0.0));
    return vec4<f32>(output_colour, 1.0);
}
