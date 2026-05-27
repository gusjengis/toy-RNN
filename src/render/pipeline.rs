use std::error::Error;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{network::Network, neuron::Neuron};

use super::text::{TextLabel, TextRenderer};

type RenderResult<T> = Result<T, Box<dyn Error>>;

const BASE_LAYER_SPACING: f32 = 340.0;
const CONNECTION_SPACING_SCALE: f32 = 18.0;
const NEURON_SPACING: f32 = 92.0;
const NEURON_RADIUS: f32 = 28.0;
const INPUT_HALF_SIZE: f32 = 26.0;
const TEXT_LABEL_GAP: f32 = 14.0;
const TEXT_LABEL_FONT_SIZE: f32 = 15.0;
const TEXT_VALUE_FONT_SIZE: f32 = 10.5;
const TEXT_SUMMARY_FONT_SIZE: f32 = 18.0;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 8.0;
const MIN_CONNECTION_THICKNESS: f32 = 1.0;
const MAX_CONNECTION_THICKNESS: f32 = 7.0;
const TEXT_PRIMARY_COLOR: [f32; 4] = [0.95, 0.97, 1.0, 1.0];
const TEXT_MUTED_COLOR: [f32; 4] = [0.62, 0.68, 0.80, 1.0];
const TEXT_ACCENT_COLOR: [f32; 4] = [1.0, 0.78, 0.36, 1.0];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct NeuronInstance {
    position: [f32; 2],
    radius: f32,
    output: f32,
}

impl NeuronInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<NeuronInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ConnectionInstance {
    start: [f32; 2],
    end: [f32; 2],
    thickness: f32,
    weight: f32,
}

impl ConnectionInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ConnectionInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() + std::mem::size_of::<f32>())
                        as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InputInstance {
    position: [f32; 2],
    half_size: f32,
    value: f32,
}

impl InputInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InputInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    pan_and_viewport: [f32; 4],
    zoom_and_padding: [f32; 4],
}

struct Camera {
    pan: [f32; 2],
    viewport: [f32; 2],
    zoom: f32,
}

impl Camera {
    fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            pan: [0.0, 0.0],
            viewport: [size.width as f32, size.height as f32],
            zoom: 1.0,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.viewport = [size.width as f32, size.height as f32];
    }

    fn pan_by_screen_delta(&mut self, delta_x: f32, delta_y: f32) {
        self.pan[0] += delta_x / self.zoom;
        self.pan[1] -= delta_y / self.zoom;
    }

    fn zoom_at_screen_position(&mut self, scroll_delta: f32, screen_position: [f32; 2]) {
        let old_zoom = self.zoom;
        let multiplier = 1.12_f32.powf(scroll_delta);
        self.zoom = (self.zoom * multiplier).clamp(MIN_ZOOM, MAX_ZOOM);

        let screen_from_center = [
            screen_position[0] - self.viewport[0] * 0.5,
            -(screen_position[1] - self.viewport[1] * 0.5),
        ];
        self.pan[0] += screen_from_center[0] * (1.0 / self.zoom - 1.0 / old_zoom);
        self.pan[1] += screen_from_center[1] * (1.0 / self.zoom - 1.0 / old_zoom);
    }

    fn uniform(&self) -> CameraUniform {
        CameraUniform {
            pan_and_viewport: [self.pan[0], self.pan[1], self.viewport[0], self.viewport[1]],
            zoom_and_padding: [self.zoom, 0.0, 0.0, 0.0],
        }
    }
}

pub struct GpuState {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    input_pipeline: wgpu::RenderPipeline,
    input_buffer: wgpu::Buffer,
    input_count: u32,
    neuron_pipeline: wgpu::RenderPipeline,
    neuron_buffer: wgpu::Buffer,
    neuron_count: u32,
    connection_pipeline: wgpu::RenderPipeline,
    connection_buffer: wgpu::Buffer,
    connection_count: u32,
    text_renderer: TextRenderer,
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    clear_color: wgpu::Color,
}

impl GpuState {
    pub async fn new(
        window: &Window,
        size: PhysicalSize<u32>,
        network: &Network,
        inputs: &[f32],
        character_labels: &[char],
    ) -> RenderResult<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(window)?)?
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Toy RNN GPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::default(),
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::AutoVsync)
        {
            wgpu::PresentMode::AutoVsync
        } else if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Fifo)
        {
            wgpu::PresentMode::Fifo
        } else {
            surface_caps.present_modes[0]
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera = Camera::new(size);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::bytes_of(&camera.uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let neuron_pipeline =
            create_neuron_pipeline(&device, config.format, &camera_bind_group_layout);
        let neuron_instances = build_neuron_instances(network, inputs.len());
        let neuron_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Neuron Instance Buffer"),
            contents: bytemuck::cast_slice(&neuron_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let input_pipeline =
            create_input_pipeline(&device, config.format, &camera_bind_group_layout);
        let input_instances = build_input_instances(network, inputs);
        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input Instance Buffer"),
            contents: bytemuck::cast_slice(&input_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let connection_pipeline =
            create_connection_pipeline(&device, config.format, &camera_bind_group_layout);
        let connection_instances = build_connection_instances(network, inputs.len());
        let connection_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Connection Instance Buffer"),
            contents: bytemuck::cast_slice(&connection_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let text_labels = build_text_labels(network, inputs, character_labels);
        let text_renderer = TextRenderer::new(
            &device,
            &queue,
            config.format,
            &camera_bind_group_layout,
            &text_labels,
        )?;
        let clear_color = wgpu::Color {
            r: 0.018,
            g: 0.02,
            b: 0.032,
            a: 1.0,
        };

        Ok(Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            size,
            input_pipeline,
            input_buffer,
            input_count: input_instances.len() as u32,
            neuron_pipeline,
            neuron_buffer,
            neuron_count: neuron_instances.len() as u32,
            connection_pipeline,
            connection_buffer,
            connection_count: connection_instances.len() as u32,
            text_renderer,
            camera,
            camera_buffer,
            camera_bind_group,
            clear_color,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.camera.resize(size);
        self.write_camera();
    }

    pub fn pan_by_screen_delta(&mut self, delta_x: f32, delta_y: f32) {
        self.camera.pan_by_screen_delta(delta_x, delta_y);
        self.write_camera();
    }

    pub fn zoom_at_screen_position(&mut self, scroll_delta: f32, screen_position: [f32; 2]) {
        self.camera
            .zoom_at_screen_position(scroll_delta, screen_position);
        self.write_camera();
    }

    pub fn refresh_network(
        &mut self,
        network: &Network,
        inputs: &[f32],
        character_labels: &[char],
    ) {
        let neuron_instances = build_neuron_instances(network, inputs.len());
        self.queue.write_buffer(
            &self.neuron_buffer,
            0,
            bytemuck::cast_slice(&neuron_instances),
        );
        self.neuron_count = neuron_instances.len() as u32;

        let input_instances = build_input_instances(network, inputs);
        self.queue.write_buffer(
            &self.input_buffer,
            0,
            bytemuck::cast_slice(&input_instances),
        );
        self.input_count = input_instances.len() as u32;

        let text_labels = build_text_labels(network, inputs, character_labels);
        self.text_renderer
            .replace_labels(&self.device, &text_labels);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Neuron Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Neuron Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.connection_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.connection_buffer.slice(..));
            render_pass.draw(0..6, 0..self.connection_count);

            render_pass.set_pipeline(&self.input_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.input_buffer.slice(..));
            render_pass.draw(0..6, 0..self.input_count);

            render_pass.set_pipeline(&self.neuron_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.neuron_buffer.slice(..));
            render_pass.draw(0..6, 0..self.neuron_count);

            self.text_renderer
                .render(&mut render_pass, &self.camera_bind_group);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }

    fn write_camera(&self) {
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&self.camera.uniform()),
        );
    }
}

fn build_neuron_instances(network: &Network, input_count: usize) -> Vec<NeuronInstance> {
    let layers = network.neuron_layers().collect::<Vec<_>>();
    let layer_x_positions = layer_x_positions(input_count, &layers);
    let mut instances = Vec::new();

    for (layer_index, layer) in layers.iter().enumerate() {
        let x = layer_x_positions[layer_index + 1];
        for (neuron_index, neuron) in layer.iter().enumerate() {
            let y = -centered_position(neuron_index, layer.len(), NEURON_SPACING);
            instances.push(NeuronInstance {
                position: [x, y],
                radius: NEURON_RADIUS,
                output: neuron.output,
            });
        }
    }

    instances
}

fn build_input_instances(network: &Network, inputs: &[f32]) -> Vec<InputInstance> {
    let layers = network.neuron_layers().collect::<Vec<_>>();
    let layer_x_positions = layer_x_positions(inputs.len(), &layers);
    let x = layer_x_positions[0];

    inputs
        .iter()
        .enumerate()
        .map(|(input_index, input)| InputInstance {
            position: [
                x,
                -centered_position(input_index, inputs.len(), NEURON_SPACING),
            ],
            half_size: INPUT_HALF_SIZE,
            value: input.clamp(0.0, 1.0),
        })
        .collect()
}

fn build_connection_instances(network: &Network, input_count: usize) -> Vec<ConnectionInstance> {
    let layers = network.neuron_layers().collect::<Vec<_>>();
    let layer_count = layers.len();
    let layer_x_positions = layer_x_positions(input_count, &layers);
    let mut instances = Vec::new();

    if let Some(first_layer) = layers.first() {
        let start_x = layer_x_positions[0];
        let end_x = layer_x_positions[1];

        for (to_index, to_neuron) in first_layer.iter().enumerate() {
            let end_y = -centered_position(to_index, first_layer.len(), NEURON_SPACING);

            for (from_index, weight) in to_neuron.weights().iter().take(input_count).enumerate() {
                let start_y = -centered_position(from_index, input_count, NEURON_SPACING);
                instances.push(connection_instance(
                    [start_x + INPUT_HALF_SIZE, start_y],
                    [end_x - NEURON_RADIUS, end_y],
                    *weight,
                ));
            }
        }
    }

    for layer_index in 1..layer_count {
        let previous_layer = layers[layer_index - 1];
        let current_layer = layers[layer_index];
        let start_x = layer_x_positions[layer_index];
        let end_x = layer_x_positions[layer_index + 1];

        for (to_index, to_neuron) in current_layer.iter().enumerate() {
            let end_y = -centered_position(to_index, current_layer.len(), NEURON_SPACING);

            for (from_index, weight) in to_neuron
                .weights()
                .iter()
                .take(previous_layer.len())
                .enumerate()
            {
                let start_y = -centered_position(from_index, previous_layer.len(), NEURON_SPACING);
                instances.push(connection_instance(
                    [start_x + NEURON_RADIUS, start_y],
                    [end_x - NEURON_RADIUS, end_y],
                    *weight,
                ));
            }
        }
    }

    instances
}

fn build_text_labels(
    network: &Network,
    inputs: &[f32],
    character_labels: &[char],
) -> Vec<TextLabel> {
    let layers = network.neuron_layers().collect::<Vec<_>>();
    let layer_x_positions = layer_x_positions(inputs.len(), &layers);
    let mut labels = Vec::new();

    if layer_x_positions.is_empty() {
        return labels;
    }

    let input_x = layer_x_positions[0];
    for (input_index, input) in inputs.iter().enumerate() {
        let y = -centered_position(input_index, inputs.len(), NEURON_SPACING);
        let label = character_labels
            .get(input_index)
            .map(|character| character_label(*character))
            .unwrap_or_else(|| format!("x{input_index}"));
        let label_color = if *input > 0.5 {
            TEXT_ACCENT_COLOR
        } else {
            TEXT_MUTED_COLOR
        };

        labels.push(TextLabel::right(
            label,
            [input_x - INPUT_HALF_SIZE - TEXT_LABEL_GAP, y],
            TEXT_LABEL_FONT_SIZE,
            label_color,
        ));
        labels.push(TextLabel::left(
            format_value(*input),
            [input_x + INPUT_HALF_SIZE + TEXT_LABEL_GAP, y],
            TEXT_VALUE_FONT_SIZE,
            TEXT_PRIMARY_COLOR,
        ));
    }

    for (layer_index, layer) in layers.iter().enumerate() {
        let x = layer_x_positions[layer_index + 1];
        let is_output_layer = layer_index == layers.len().saturating_sub(1);

        for (neuron_index, neuron) in layer.iter().enumerate() {
            let y = -centered_position(neuron_index, layer.len(), NEURON_SPACING);
            labels.push(TextLabel::center(
                format_value(neuron.output),
                [x, y],
                TEXT_VALUE_FONT_SIZE,
                TEXT_PRIMARY_COLOR,
            ));

            if is_output_layer {
                let label = character_labels
                    .get(neuron_index)
                    .map(|character| character_label(*character))
                    .unwrap_or_else(|| format!("y{neuron_index}"));
                labels.push(TextLabel::left(
                    label,
                    [x + NEURON_RADIUS + TEXT_LABEL_GAP, y],
                    TEXT_LABEL_FONT_SIZE,
                    TEXT_MUTED_COLOR,
                ));
            }
        }
    }

    if let Some(output_layer) = layers.last() {
        if let Some((prediction_index, _)) = output_layer
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.output.total_cmp(&right.output))
        {
            let output_x = layer_x_positions[layer_x_positions.len() - 1];
            let top_y = -centered_position(0, output_layer.len(), NEURON_SPACING)
                + NEURON_RADIUS
                + TEXT_SUMMARY_FONT_SIZE
                + TEXT_LABEL_GAP;
            let prediction = character_labels
                .get(prediction_index)
                .map(|character| character_label(*character))
                .unwrap_or_else(|| format!("y{prediction_index}"));

            labels.push(TextLabel::center(
                format!("Predicted: {prediction}"),
                [output_x, top_y],
                TEXT_SUMMARY_FONT_SIZE,
                TEXT_ACCENT_COLOR,
            ));
        }
    }

    labels
}

fn character_label(character: char) -> String {
    match character {
        ' ' => "space".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        character if character.is_ascii_graphic() => character.to_string(),
        character if character.is_ascii() => format!("U+{:02X}", character as u32),
        character => format!("U+{:04X}", character as u32),
    }
}

fn format_value(value: f32) -> String {
    let magnitude = value.abs();
    if magnitude >= 100.0 {
        format!("{value:.0}")
    } else if magnitude >= 10.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    }
}

fn connection_instance(start: [f32; 2], end: [f32; 2], weight: f32) -> ConnectionInstance {
    let magnitude = weight.abs().min(1.0);
    let thickness = MIN_CONNECTION_THICKNESS
        + magnitude * (MAX_CONNECTION_THICKNESS - MIN_CONNECTION_THICKNESS);

    ConnectionInstance {
        start,
        end,
        thickness,
        weight,
    }
}

fn layer_x_positions(input_count: usize, layers: &[&[Neuron]]) -> Vec<f32> {
    if layers.is_empty() && input_count == 0 {
        return Vec::new();
    }

    let mut layer_sizes = Vec::with_capacity(layers.len() + 1);
    layer_sizes.push(input_count);
    layer_sizes.extend(layers.iter().map(|layer| layer.len()));

    let mut positions = Vec::with_capacity(layer_sizes.len());
    positions.push(0.0);

    for layer_index in 1..layer_sizes.len() {
        let connection_count = layer_sizes[layer_index - 1] * layer_sizes[layer_index];
        let gap = BASE_LAYER_SPACING + (connection_count as f32).sqrt() * CONNECTION_SPACING_SCALE;
        positions.push(positions[layer_index - 1] + gap);
    }

    let midpoint = (positions[0] + positions[positions.len() - 1]) * 0.5;
    for position in positions.iter_mut() {
        *position -= midpoint;
    }

    positions
}

fn centered_position(index: usize, count: usize, spacing: f32) -> f32 {
    if count <= 1 {
        return 0.0;
    }

    (index as f32 - (count as f32 - 1.0) * 0.5) * spacing
}

fn create_neuron_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Neuron Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/neuron.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Neuron Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Neuron Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[NeuronInstance::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

fn create_input_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Input Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/input.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Input Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Input Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[InputInstance::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

fn create_connection_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Connection Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/connection.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Connection Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Connection Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[ConnectionInstance::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
