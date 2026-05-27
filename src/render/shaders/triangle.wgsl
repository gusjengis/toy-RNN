struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.55),
        vec2<f32>(-0.55, -0.45),
        vec2<f32>(0.55, -0.45),
    );
    var colors = array<vec3<f32>, 3>(
        vec3<f32>(0.95, 0.25, 0.25),
        vec3<f32>(0.25, 0.85, 0.45),
        vec3<f32>(0.25, 0.45, 0.95),
    );

    var out: VertexOutput;
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.color = colors[vertex_index];
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
