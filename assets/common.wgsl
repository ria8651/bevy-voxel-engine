//note: uniformly distributed, normalized rand, [-1, 1]
fn rand(n: vec2<f32>) -> f32 {
	return fract(sin(dot(n.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453) * 2.0 - 1.0;
}