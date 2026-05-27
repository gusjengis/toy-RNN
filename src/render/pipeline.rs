use std::error::Error;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::network::Network;

type RenderResult<T> = Result<T, Box<dyn Error>>;

const LAYER_SPACING: f32 = 260.0;
const NEURON_SPACING: f32 = 92.0;
const NEURON_RADIUS: f32 = 28.0;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 8.0;

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
    render_pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
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

        let render_pipeline =
            create_neuron_pipeline(&device, config.format, &camera_bind_group_layout);
        let neuron_instances = build_neuron_instances(network);
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Neuron Instance Buffer"),
            contents: bytemuck::cast_slice(&neuron_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
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
            render_pipeline,
            instance_buffer,
            instance_count: neuron_instances.len() as u32,
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
            render_pass.draw(0..6, 0..self.instance_count);
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

fn build_neuron_instances(network: &Network) -> Vec<NeuronInstance> {
    let layers = network.neuron_layers().collect::<Vec<_>>();
    let layer_count = layers.len();
    let mut instances = Vec::new();

    for (layer_index, layer) in layers.iter().enumerate() {
        let x = centered_position(layer_index, layer_count, LAYER_SPACING);
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
