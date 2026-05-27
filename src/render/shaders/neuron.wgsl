struct Camera {
    pan_and_viewport: vec4<f32>,
    zoom_and_padding: vec4<f32>,
};

struct VertexInput {
    @location(0) instance_position: vec2<f32>,
    @location(1) instance_radius: f32,
    @location(2) instance_output: f32,
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_position: vec2<f32>,
    @location(1) neuron_output: f32,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var quad = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );

    let local_position = quad[input.vertex_index];
    let world_position = input.instance_position + local_position * input.instance_radius;
    let pan = camera.pan_and_viewport.xy;
    let viewport = max(camera.pan_and_viewport.zw, vec2<f32>(1.0, 1.0));
    let zoom = camera.zoom_and_padding.x;
    let clip_position = (world_position + pan) * zoom / (viewport * 0.5);

    var out: VertexOutput;
    out.position = vec4<f32>(clip_position, 0.0, 1.0);
    out.local_position = local_position;
    out.neuron_output = input.instance_output;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let distance_from_center = length(input.local_position);
    if (distance_from_center > 1.0) {
        discard;
    }

    let magnitude = clamp(abs(input.neuron_output) * 0.2, 0.0, 1.0);
    let positive_color = vec3<f32>(0.95, 0.38, 0.18);
    let negative_color = vec3<f32>(0.24, 0.48, 1.0);
    let neutral_color = vec3<f32>(0.22, 0.25, 0.32);
    let active_color = select(negative_color, positive_color, input.neuron_output >= 0.0);
    let fill_color = mix(neutral_color, active_color, magnitude);
    let border_color = vec3<f32>(0.72, 0.78, 0.92);
    let color = select(fill_color, border_color, distance_from_center > 0.86);

    return vec4<f32>(color, 1.0);
}
