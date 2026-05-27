struct Camera {
    pan_and_viewport: vec4<f32>,
    zoom_and_padding: vec4<f32>,
};

struct MsdfParams {
    px_range_and_padding: vec4<f32>,
};

struct VertexInput {
    @location(0) instance_position: vec2<f32>,
    @location(1) instance_size: vec2<f32>,
    @location(2) uv_min: vec2<f32>,
    @location(3) uv_max: vec2<f32>,
    @location(4) color: vec4<f32>,
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var msdf_tex: texture_2d<f32>;

@group(1) @binding(1)
var msdf_sampler: sampler;

@group(1) @binding(2)
var<uniform> msdf_params: MsdfParams;

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
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(-0.5, 0.5),
        vec2<f32>(-0.5, 0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(0.5, 0.5),
    );

    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(input.uv_min.x, input.uv_max.y),
        vec2<f32>(input.uv_max.x, input.uv_max.y),
        vec2<f32>(input.uv_min.x, input.uv_min.y),
        vec2<f32>(input.uv_min.x, input.uv_min.y),
        vec2<f32>(input.uv_max.x, input.uv_max.y),
        vec2<f32>(input.uv_max.x, input.uv_min.y),
    );

    let world_position = input.instance_position + corners[input.vertex_index] * input.instance_size;

    var out: VertexOutput;
    out.position = world_to_clip(world_position);
    out.tex_coord = uvs[input.vertex_index];
    out.color = input.color;
    return out;
}

fn median(a: f32, b: f32, c: f32) -> f32 {
    return max(min(a, b), min(max(a, b), c));
}

fn screen_px_range(tex_coord: vec2<f32>) -> f32 {
    let px_range = msdf_params.px_range_and_padding.x;
    let texture_size = textureDimensions(msdf_tex, 0u);
    let tex_size = vec2<f32>(f32(texture_size.x), f32(texture_size.y));
    let unit_range = vec2<f32>(px_range, px_range) / tex_size;
    let fw = fwidth(tex_coord);
    let fw_safe = select(vec2<f32>(1.0, 1.0), fw, all(fw != vec2<f32>(0.0, 0.0)));
    let screen_tex_size = vec2<f32>(1.0, 1.0) / fw_safe;
    return max(0.5 * dot(unit_range, screen_tex_size), 1.0);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let msd = textureSample(msdf_tex, msdf_sampler, input.tex_coord).rgb;
    let signed_distance = median(msd.r, msd.g, msd.b);
    let range = screen_px_range(input.tex_coord);
    let screen_px_distance = range * (signed_distance - 0.5);
    let opacity = smoothstep(-0.5, 0.5, screen_px_distance);
    let corrected_opacity = pow(opacity, 1.0 / 2.2);

    return vec4<f32>(input.color.rgb, input.color.a * corrected_opacity);
}
