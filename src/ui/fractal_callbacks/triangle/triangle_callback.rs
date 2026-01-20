use eframe::egui_wgpu::{CallbackResources, CallbackTrait};
use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;

use crate::audio::SpectrogramBins;
use crate::ui::fractal_callbacks::common::*;

// Triangle vertex data
// Equilateral triangle vertices
// Center at (0,0), height of 1.5, base of ~1.732 (equilateral)
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.75],  // Top vertex
        color: [1.0, 1.0, 1.0], // White
    },
    Vertex {
        position: [-0.866, -0.75], // Bottom left
        color: [1.0, 1.0, 1.0],    // White
    },
    Vertex {
        position: [0.866, -0.75], // Bottom right
        color: [1.0, 1.0, 1.0],   // White
    },
];

#[derive(Clone)]
pub struct TriangleResources {
    pub pipeline: Arc<wgpu::RenderPipeline>,
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub uniform_buffer: Arc<wgpu::Buffer>,
    pub bind_group: Arc<wgpu::BindGroup>,
}

#[derive(Clone)]
pub struct TriangleCallback {
    pub fft_data: SpectrogramBins,
    pub current_time: f64,
}

impl TriangleCallback {
    pub fn new(fft_data: SpectrogramBins, current_time: f64) -> Self {
        Self {
            fft_data,
            current_time,
        }
    }

    /// Extract 3 frequency bands from FFT data
    fn extract_frequency_bands(fft_data: &SpectrogramBins) -> [f32; 3] {
        let bins = &fft_data.bins;
        let band_size = bins.len() / 3;

        // Calculate average for each frequency band
        let low = bins[0..band_size].iter().sum::<f32>() / band_size as f32;
        let mid = bins[band_size..2 * band_size].iter().sum::<f32>() / band_size as f32;
        let high = bins[2 * band_size..].iter().sum::<f32>() / band_size as f32;

        // Amplify mid/high frequencies for better visibility, clamp to [0,1]
        [
            low.clamp(0.0, 1.0),
            (mid * 5.0).clamp(0.0, 1.0),
            (high * 10.0).clamp(0.0, 1.0),
        ]
    }
}

impl CallbackTrait for TriangleCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let _resources_key = std::any::TypeId::of::<TriangleResources>();

        // Only create resources once
        if !callback_resources.contains::<TriangleResources>() {
            // Create shader module
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Triangle Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("triangle.wgsl"))),
            });

            // Create vertex buffer
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Triangle Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

            // Create uniform buffer for frequency bands [low, mid, high, time]
            let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Triangle Uniform Buffer"),
                contents: bytemuck::cast_slice(&[0.0f32; 4]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            // Create bind group
            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Triangle Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Triangle Bind Group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            });

            // Create pipeline
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Triangle Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Triangle Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

            let resources = TriangleResources {
                pipeline: Arc::new(pipeline),
                vertex_buffer: Arc::new(vertex_buffer),
                uniform_buffer: Arc::new(uniform_buffer),
                bind_group: Arc::new(bind_group),
            };

            callback_resources.insert(resources);
        }

        // Update uniform buffer with frequency data
        if let Some(resources) = callback_resources.get_mut::<TriangleResources>() {
            let bands = Self::extract_frequency_bands(&self.fft_data);
            let uniform_data = [bands[0], bands[1], bands[2], self.current_time as f32];
            queue.write_buffer(
                &resources.uniform_buffer,
                0,
                bytemuck::cast_slice(&[uniform_data]),
            );
        }

        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        let resources: &TriangleResources = callback_resources
            .get::<TriangleResources>()
            .expect("Triangle resources not found");

        // Set scissor to the widget area to ensure we don't render outside
        render_pass.set_scissor_rect(
            info.clip_rect.min.x as u32,
            info.clip_rect.min.y as u32,
            info.clip_rect.width() as u32,
            info.clip_rect.height() as u32,
        );

        // Render the triangle
        render_pass.set_pipeline(&resources.pipeline);
        render_pass.set_bind_group(0, &*resources.bind_group, &[]);
        render_pass.set_vertex_buffer(0, resources.vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);
    }
}
