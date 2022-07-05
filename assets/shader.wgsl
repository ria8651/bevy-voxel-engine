struct Uniforms {
    resolution: vec2<f32>;
};

[[group(1), binding(0)]]
var<uniform> u: Uniforms;

fn get_clip_space(frag_pos: vec4<f32>, dimensions: vec2<f32>) -> vec2<f32> {
    var clip_space = frag_pos.xy / dimensions * 2.0;
    clip_space = clip_space - 1.0;
    clip_space = clip_space * vec2<f32>(1.0, -1.0);
    return clip_space;
}

[[stage(fragment)]]
fn fragment([[builtin(position)]] frag_pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
    var output_colour = vec3<f32>(0.0, 0.0, 0.0);
    let clip_space = get_clip_space(frag_pos, u.resolution);

    output_colour = vec3<f32>(clip_space, 0.0);

    return vec4<f32>(output_colour, 1.0);
}
