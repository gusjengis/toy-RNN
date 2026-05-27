use std::{collections::HashMap, error::Error};

use bytemuck::{Pod, Zeroable};
use serde::Deserialize;
use wgpu::util::DeviceExt;

type RenderResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Copy)]
pub enum TextAnchor {
    Left,
    Center,
    Right,
}

pub struct TextLabel {
    pub text: String,
    pub position: [f32; 2],
    pub font_size: f32,
    pub color: [f32; 4],
    pub anchor: TextAnchor,
}

impl TextLabel {
    pub fn left(
        text: impl Into<String>,
        position: [f32; 2],
        font_size: f32,
        color: [f32; 4],
    ) -> Self {
        Self {
            text: text.into(),
            position,
            font_size,
            color,
            anchor: TextAnchor::Left,
        }
    }

    pub fn center(
        text: impl Into<String>,
        position: [f32; 2],
        font_size: f32,
        color: [f32; 4],
    ) -> Self {
        Self {
            text: text.into(),
            position,
            font_size,
            color,
            anchor: TextAnchor::Center,
        }
    }

    pub fn right(
        text: impl Into<String>,
        position: [f32; 2],
        font_size: f32,
        color: [f32; 4],
    ) -> Self {
        Self {
            text: text.into(),
            position,
            font_size,
            color,
            anchor: TextAnchor::Right,
        }
    }
}

#[derive(Debug, Deserialize)]
struct FontAtlas {
    atlas: AtlasInfo,
    metrics: FontMetrics,
    glyphs: Vec<Glyph>,
    #[serde(default)]
    kerning: Vec<KerningPair>,
}

#[derive(Debug, Deserialize)]
struct AtlasInfo {
    #[serde(rename = "distanceRange")]
    distance_range: f32,
    width: u32,
    height: u32,
}

#[derive(Debug, Deserialize)]
struct FontMetrics {
    #[serde(rename = "emSize")]
    em_size: f32,
    ascender: f32,
    descender: f32,
}

#[derive(Debug, Deserialize)]
struct Glyph {
    unicode: u32,
    advance: f32,
    #[serde(default, rename = "planeBounds")]
    plane_bounds: Option<Bounds>,
    #[serde(default, rename = "atlasBounds")]
    atlas_bounds: Option<Bounds>,
}

#[derive(Debug, Deserialize)]
struct Bounds {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
}

#[derive(Debug, Deserialize)]
struct KerningPair {
    unicode1: u32,
    unicode2: u32,
    advance: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GlyphInstance {
    position: [f32; 2],
    size: [f32; 2],
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    color: [f32; 4],
}

impl GlyphInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GlyphInstance>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MsdfParams {
    px_range: f32,
    padding: [f32; 3],
}

pub struct TextRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    glyph_buffer: wgpu::Buffer,
    glyph_count: u32,
    _atlas_texture: wgpu::Texture,
    _atlas_view: wgpu::TextureView,
    _atlas_sampler: wgpu::Sampler,
    _params_buffer: wgpu::Buffer,
}

impl TextRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        labels: &[TextLabel],
    ) -> RenderResult<Self> {
        let font_atlas = FontAtlas::from_embedded()?;
        let (atlas_texture, atlas_view, atlas_sampler) = create_font_texture(device, queue)?;
        let params = MsdfParams {
            px_range: font_atlas.atlas.distance_range,
            padding: [0.0; 3],
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("MSDF Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let text_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("MSDF Text Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("MSDF Text Bind Group"),
            layout: &text_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline = create_text_pipeline(
            device,
            surface_format,
            camera_bind_group_layout,
            &text_bind_group_layout,
        );
        let glyph_instances = font_atlas.build_glyph_instances(labels);
        let glyph_buffer = if glyph_instances.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Empty MSDF Glyph Instance Buffer"),
                size: 1,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("MSDF Glyph Instance Buffer"),
                contents: bytemuck::cast_slice(&glyph_instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
        };

        Ok(Self {
            pipeline,
            bind_group,
            glyph_buffer,
            glyph_count: glyph_instances.len() as u32,
            _atlas_texture: atlas_texture,
            _atlas_view: atlas_view,
            _atlas_sampler: atlas_sampler,
            _params_buffer: params_buffer,
        })
    }

    pub fn render<'pass>(
        &'pass self,
        render_pass: &mut wgpu::RenderPass<'pass>,
        camera_bind_group: &'pass wgpu::BindGroup,
    ) {
        if self.glyph_count == 0 {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.glyph_buffer.slice(..));
        render_pass.draw(0..6, 0..self.glyph_count);
    }
}

impl FontAtlas {
    fn from_embedded() -> RenderResult<Self> {
        Ok(serde_json::from_str(include_str!(
            "assets/commit_mono_mtsdf_64.json"
        ))?)
    }

    fn build_glyph_instances(&self, labels: &[TextLabel]) -> Vec<GlyphInstance> {
        let glyph_map = self
            .glyphs
            .iter()
            .map(|glyph| (glyph.unicode, glyph))
            .collect::<HashMap<_, _>>();
        let atlas_width = self.atlas.width as f32;
        let atlas_height = self.atlas.height as f32;
        let mut instances = Vec::new();

        for label in labels {
            if label.font_size <= 0.0 || self.metrics.em_size == 0.0 {
                continue;
            }

            let scale = label.font_size / self.metrics.em_size;
            let text_width = self.line_width(&label.text, scale, &glyph_map);
            let baseline_x = match label.anchor {
                TextAnchor::Left => label.position[0],
                TextAnchor::Center => label.position[0] - text_width * 0.5,
                TextAnchor::Right => label.position[0] - text_width,
            };
            let vertical_center = (self.metrics.ascender + self.metrics.descender) * 0.5 * scale;
            let baseline_y = label.position[1] - vertical_center;
            let mut pen_x = 0.0;
            let mut previous_char = None;

            for character in label.text.chars() {
                let unicode = character as u32;
                if let Some(previous_unicode) = previous_char {
                    pen_x += self.kerning(previous_unicode, unicode) * scale;
                }

                let Some(glyph) = glyph_map.get(&unicode) else {
                    pen_x += 0.25 * scale;
                    previous_char = Some(unicode);
                    continue;
                };

                if let (Some(plane_bounds), Some(atlas_bounds)) =
                    (&glyph.plane_bounds, &glyph.atlas_bounds)
                {
                    let width = (plane_bounds.right - plane_bounds.left) * scale;
                    let height = (plane_bounds.top - plane_bounds.bottom) * scale;
                    let center_x = (plane_bounds.left + plane_bounds.right) * 0.5 * scale;
                    let center_y = (plane_bounds.bottom + plane_bounds.top) * 0.5 * scale;
                    let u0 = atlas_bounds.left / atlas_width;
                    let u1 = atlas_bounds.right / atlas_width;
                    let v_top = 1.0 - atlas_bounds.top / atlas_height;
                    let v_bottom = 1.0 - atlas_bounds.bottom / atlas_height;

                    instances.push(GlyphInstance {
                        position: [baseline_x + pen_x + center_x, baseline_y + center_y],
                        size: [width, height],
                        uv_min: [u0, v_top],
                        uv_max: [u1, v_bottom],
                        color: label.color,
                    });
                }

                pen_x += glyph.advance * scale;
                previous_char = Some(unicode);
            }
        }

        instances
    }

    fn line_width(&self, text: &str, scale: f32, glyph_map: &HashMap<u32, &Glyph>) -> f32 {
        let mut width = 0.0;
        let mut previous_char = None;

        for character in text.chars() {
            let unicode = character as u32;
            if let Some(previous_unicode) = previous_char {
                width += self.kerning(previous_unicode, unicode) * scale;
            }

            width += glyph_map
                .get(&unicode)
                .map(|glyph| glyph.advance * scale)
                .unwrap_or(0.25 * scale);
            previous_char = Some(unicode);
        }

        width
    }

    fn kerning(&self, unicode1: u32, unicode2: u32) -> f32 {
        self.kerning
            .iter()
            .find(|pair| pair.unicode1 == unicode1 && pair.unicode2 == unicode2)
            .map(|pair| pair.advance)
            .unwrap_or(0.0)
    }
}

fn create_font_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> RenderResult<(wgpu::Texture, wgpu::TextureView, wgpu::Sampler)> {
    let image =
        image::load_from_memory(include_bytes!("assets/commit_mono_mtsdf_64.png"))?.to_rgba8();
    let (width, height) = image.dimensions();
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("MSDF Font Atlas"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        image.as_raw(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("MSDF Font Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    Ok((texture, view, sampler))
}

fn create_text_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    text_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("MSDF Text Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/msdf_text.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("MSDF Text Pipeline Layout"),
        bind_group_layouts: &[camera_bind_group_layout, text_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("MSDF Text Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[GlyphInstance::desc()],
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
