struct ColorData {
		tint: vec4<f32>
};
@group(0) @binding(0)
var<uniform> u_color: ColorData;

struct VertexOut {
		@builtin(position) position: vec4<f32>,
		@location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
		var positions = array<vec2<f32>, 3>(
				vec2<f32>(0.0,  0.8),
				vec2<f32>(-0.8, -0.8),
				vec2<f32>(0.8,  -0.8)
		);
		var colors = array<vec3<f32>, 3>(
				vec3<f32>(1.0, 0.0, 0.0),
				vec3<f32>(0.0, 1.0, 0.0),
				vec3<f32>(0.0, 0.0, 1.0)
		);

		var out: VertexOut;
		out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
		out.color = colors[vertex_index];
		return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
		return vec4<f32>(in.color * u_color.tint.rgb, u_color.tint.a);
}
