use eframe::egui_wgpu::{CallbackResources, CallbackTrait};
use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;

use crate::audio::SpectrogramBins;

#[derive(Clone)]
pub struct AuroraResources {
    pub background_pipeline: Arc<wgpu::RenderPipeline>,
    pub ring_pipeline: Arc<wgpu::RenderPipeline>,
    pub background_uniform_buffer: Arc<wgpu::Buffer>,
    pub ring_uniform_buffer: Arc<wgpu::Buffer>,
    pub background_bind_group: Arc<wgpu::BindGroup>,
    pub ring_bind_group: Arc<wgpu::BindGroup>,
}

#[derive(Clone)]
pub struct AuroraCallback {
    pub fft_data: SpectrogramBins,
    pub current_time: f64,
}

impl AuroraCallback {
    pub fn new(fft_data: SpectrogramBins, current_time: f64) -> Self {
        Self {
            fft_data,
            current_time,
        }
    }

    fn extract_frequency_bands(fft_data: &SpectrogramBins) -> [f32; 3] {
        let bins = &fft_data.bins;
        let sample_rate = fft_data.sample_rate_hz as f32;
        let num_bins = bins.len();

        // Calculate Nyquist frequency (half the sample rate)
        let nyquist = sample_rate / 2.0;

        // Helper to convert frequency (Hz) to bin index
        let freq_to_bin = |freq: f32| -> usize {
            ((freq * num_bins as f32) / nyquist).floor() as usize
        };

        // Helper to average bins in a range
        let average_bins = |start: usize, end: usize| -> f32 {
            let start = start.min(num_bins);
            let end = end.min(num_bins);
            if start >= end {
                return 0.0;
            }
            let sum: f32 = bins[start..end].iter().sum();
            sum / (end - start) as f32
        };

        // Proper frequency ranges based on musical perception
        // Bass: 20-250 Hz (sub-bass + bass fundamentals)
        let bass_start = freq_to_bin(20.0);
        let bass_end = freq_to_bin(250.0);

        // Mids: 250-2000 Hz (fundamentals + lower harmonics)
        let mid_start = bass_end;
        let mid_end = freq_to_bin(2000.0);

        // Highs: 2000-20000 Hz (upper harmonics + presence)
        let high_start = mid_end;
        let high_end = freq_to_bin(20000.0);

        let bass = average_bins(bass_start, bass_end);
        let mids = average_bins(mid_start, mid_end);
        let highs = average_bins(high_start, high_end);

        // Normalize and boost
        // Bass has more energy naturally, so less boost needed
        // Highs have less energy, so more boost needed
        [
            (bass * 2.0).clamp(0.0, 1.0),
            (mids * 3.0).clamp(0.0, 1.0),
            (highs * 8.0).clamp(0.0, 1.0),
        ]
    }

    fn prepare_shader(
        device: &wgpu::Device,
        shader_source: &str,
        shader_label: &str,
        buffer_label: &str,
        layout_label: &str,
        bind_group_label: &str,
        pipeline_label: &str,
        pipeline_layout_label: &str,
        blend_state: Option<wgpu::BlendState>,
    ) -> (Arc<wgpu::RenderPipeline>, Arc<wgpu::Buffer>, Arc<wgpu::BindGroup>) {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(shader_label),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(buffer_label),
            contents: bytemuck::cast_slice(&[0.0f32; 4]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(layout_label),
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
            label: Some(bind_group_label),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(pipeline_layout_label),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(pipeline_label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: blend_state,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
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

        (Arc::new(pipeline), Arc::new(uniform_buffer), Arc::new(bind_group))
    }
}

impl CallbackTrait for AuroraCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if !callback_resources.contains::<AuroraResources>() {
            let (background_pipeline, background_uniform_buffer, background_bind_group) =
                Self::prepare_shader(
                    device,
                    include_str!("background.wgsl"),
                    "Aurora Background Shader",
                    "Aurora Background Uniform Buffer",
                    "Aurora Background Bind Group Layout",
                    "Aurora Background Bind Group",
                    "Aurora Background Pipeline",
                    "Aurora Background Pipeline Layout",
                    Some(wgpu::BlendState::REPLACE),
                );

            let (ring_pipeline, ring_uniform_buffer, ring_bind_group) = Self::prepare_shader(
                device,
                include_str!("ring.wgsl"),
                "Aurora Ring Shader",
                "Aurora Ring Uniform Buffer",
                "Aurora Ring Bind Group Layout",
                "Aurora Ring Bind Group",
                "Aurora Ring Pipeline",
                "Aurora Ring Pipeline Layout",
                Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
            );

            let resources = AuroraResources {
                background_pipeline,
                ring_pipeline,
                background_uniform_buffer,
                ring_uniform_buffer,
                background_bind_group,
                ring_bind_group,
            };

            callback_resources.insert(resources);
        }

        if let Some(resources) = callback_resources.get_mut::<AuroraResources>() {
            let bands = Self::extract_frequency_bands(&self.fft_data);
            let uniform_data = [bands[0], bands[1], bands[2], self.current_time as f32];

            queue.write_buffer(
                &resources.background_uniform_buffer,
                0,
                bytemuck::cast_slice(&[uniform_data]),
            );
            queue.write_buffer(
                &resources.ring_uniform_buffer,
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
        let resources: &AuroraResources = callback_resources
            .get::<AuroraResources>()
            .expect("Aurora resources not found");

        render_pass.set_scissor_rect(
            info.clip_rect.min.x as u32,
            info.clip_rect.min.y as u32,
            info.clip_rect.width() as u32,
            info.clip_rect.height() as u32,
        );

        render_pass.set_pipeline(&resources.background_pipeline);
        render_pass.set_bind_group(0, &*resources.background_bind_group, &[]);
        render_pass.draw(0..4, 0..1);

        render_pass.set_pipeline(&resources.ring_pipeline);
        render_pass.set_bind_group(0, &*resources.ring_bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}

