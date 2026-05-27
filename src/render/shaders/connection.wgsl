struct Camera {
    pan_and_viewport: vec4<f32>,
    zoom_and_padding: vec4<f32>,
};

struct VertexInput {
    @location(0) start: vec2<f32>,
    @location(1) end: vec2<f32>,
    @location(2) thickness: f32,
    @location(3) weight: f32,
    @location(4) contribution: f32,
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) weight: f32,
    @location(1) contribution: f32,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

fn world_to_clip(world_position: vec2<f32>) -> vec4<f32> {
    let pan = camera.pan_and_viewport.xy;
    let viewport = max(camera.pan_and_viewport.zw, vec2<f32>(1.0, 1.0));
    let zoom = camera.zoom_and_padding.x;
    let clip_position = (world_position + pan) * zoom / (viewport * 0.5);
    return vec4<f32>(clip_position, 0.0, 1.0);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );

    let corner = corners[input.vertex_index];
    let delta = input.end - input.start;
    let direction = normalize(delta);
    let normal = vec2<f32>(-direction.y, direction.x);
    let center = mix(input.start, input.end, corner.x);
    let world_position = center + normal * corner.y * input.thickness * 0.5;

    var out: VertexOutput;
    out.position = world_to_clip(world_position);
    out.weight = input.weight;
    out.contribution = input.contribution;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let magnitude = clamp(abs(input.weight), 0.0, 1.0);
    let contribution = abs(input.contribution);
    let contribution_intensity = pow(clamp(1.0 - exp(-contribution * 3.0), 0.0, 1.0), 0.55) + 0.08;
    let negative_color = vec3<f32>(1.0, 0.08, 0.04);
    let positive_color = vec3<f32>(0.15, 1.0, 0.24);
    let color = select(negative_color, positive_color, input.weight >= 0.0);
    let brightness = mix(0.04, 1.0, contribution_intensity) * mix(0.35, 1.0, magnitude);
    let alpha = mix(0.025, 0.9, contribution_intensity);
    return vec4<f32>(color * brightness, alpha);
}
