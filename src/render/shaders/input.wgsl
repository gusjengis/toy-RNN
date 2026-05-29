struct Camera {
    pan_and_viewport: vec4<f32>,
    zoom_and_padding: vec4<f32>,
};

struct VertexInput {
    @location(0) instance_position: vec2<f32>,
    @location(1) half_size: f32,
    @location(2) value: f32,
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_position: vec2<f32>,
    @location(1) value: f32,
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
    let world_position = input.instance_position + local_position * input.half_size;
    let pan = camera.pan_and_viewport.xy;
    let viewport = max(camera.pan_and_viewport.zw, vec2<f32>(1.0, 1.0));
    let zoom = camera.zoom_and_padding.x;
    let clip_position = (world_position + pan) * zoom / (viewport * 0.5);

    var out: VertexOutput;
    out.position = vec4<f32>(clip_position, 0.0, 1.0);
    out.local_position = local_position;
    out.value = clamp(input.value, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // version with borders, not desirable for rendering inputs in a pixel grid
    // let border = max(abs(input.local_position.x), abs(input.local_position.y)) > 0.84;
    // let body_color = vec3<f32>(input.value, input.value, input.value);
    // let border_color = vec3<f32>(0.58, 0.62, 0.70);
    // let color = select(body_color, border_color, border);

    let color = vec3<f32>(input.value, input.value, input.value);
    return vec4<f32>(color, 1.0);
}
