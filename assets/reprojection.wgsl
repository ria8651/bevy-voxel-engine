#import bevy_core_pipeline::fullscreen_vertex_shader
#import "common.wgsl"

@group(0) @binding(0)
var<uniform> trace_uniforms: TraceUniforms;
@group(0) @binding(1)
var accumulation_attachment: texture_storage_2d<rgba16float, read_write>;
@group(0) @binding(2)
var normal_attachment: texture_storage_2d<rgba16float, read_write>;
@group(0) @binding(3)
var position_attachment: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(0)
var colour_attachment: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let resolution = vec2<f32>(textureDimensions(colour_attachment));
    let sample_pos = vec2<i32>(in.position.xy);
    let colour = textureLoad(colour_attachment, sample_pos, 0).rgb;
    let position = textureLoad(position_attachment, sample_pos).rgb;
    var output_colour = colour;

    if (trace_uniforms.indirect_lighting != 0u && trace_uniforms.reprojection_factor > 0.0) {
        let sample0 = colour;
        let sample1 = textureLoad(colour_attachment, sample_pos + vec2(0, 1), 0).rgb;
        let sample2 = textureLoad(colour_attachment, sample_pos + vec2(1, 1), 0).rgb;
        let sample3 = textureLoad(colour_attachment, sample_pos + vec2(1, 0), 0).rgb;
        let sample4 = textureLoad(colour_attachment, sample_pos + vec2(1, -1), 0).rgb;
        let sample5 = textureLoad(colour_attachment, sample_pos + vec2(0, -1), 0).rgb;
        let sample6 = textureLoad(colour_attachment, sample_pos + vec2(-1, -1), 0).rgb;
        let sample7 = textureLoad(colour_attachment, sample_pos + vec2(-1, 0), 0).rgb;
        let sample8 = textureLoad(colour_attachment, sample_pos + vec2(-1, 1), 0).rgb;

        let min_col_plus = min(min(min(sample1, sample3), min(sample5, sample7)), sample0);
        let max_col_plus = max(max(max(sample1, sample3), max(sample5, sample7)), sample0);
        let min_col_square = min(min(min(min(sample1, sample2), min(sample3, sample4)), min(min(sample5, sample6), min(sample7, sample8))), sample0);
        let max_col_square = max(max(max(max(sample1, sample2), max(sample3, sample4)), max(max(sample5, sample6), max(sample7, sample8))), sample0);
        let min_col = (min_col_plus + min_col_square) / 2.0;
        let max_col = (max_col_plus + max_col_square) / 2.0;
        // let min_col = min(min(min(sample1, sample3), min(sample5, sample7)), sample0);
        // let max_col = max(max(max(sample1, sample3), max(sample5, sample7)), sample0);

        // mix with samples from last frame
        let last_clip = trace_uniforms.last_camera * vec4(position, 1.0);
        let last_clip = vec2(1.0, -1.0) * last_clip.xy / last_clip.w;
        if (all(last_clip > vec2(-1.0)) && all(last_clip < vec2(1.0))) {
            let last_texture = (last_clip * 0.5 + 0.5) * resolution;
            let accumulation = textureLoad(accumulation_attachment, vec2<i32>(last_texture)).rgb;

            let mix_ammount = trace_uniforms.reprojection_factor;
            let mixed = colour.rgb * (1.0 - mix_ammount) + accumulation * mix_ammount;
            output_colour = clip_aabb(mixed, min_col, max_col);
        }
    }

    return vec4(output_colour, 1.0);
}

@fragment
fn accumulation(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let sample_pos = vec2<i32>(in.position.xy);
    let colour = textureLoad(colour_attachment, sample_pos, 0);
    textureStore(accumulation_attachment, sample_pos, colour);
    return vec4<f32>(colour.rgb, 1.0);
}