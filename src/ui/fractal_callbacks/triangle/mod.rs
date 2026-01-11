use eframe::egui_wgpu::{CallbackResources, CallbackTrait};
use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;

use crate::ui::fractal_callbacks::common::*;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 1.0],
        color: [1.0, 0.0, 0.0],
    }, // Top left - Red
    Vertex {
        position: [-1.0, -1.0],
        color: [0.0, 1.0, 0.0],
    }, // Bottom left - Green
    Vertex {
        position: [1.0, -1.0],
        color: [0.0, 0.0, 1.0],
    }, // Bottom right - Blue
];

#[derive(Clone)]
pub struct TriangleResources {
    pipeline: Arc<wgpu::RenderPipeline>,
    vertex_buffer: Arc<wgpu::Buffer>,
}

#[derive(Clone)]
pub struct TriangleCallback;

impl TriangleCallback {
    pub fn new() -> Self {
        Self
    }
}

impl CallbackTrait for TriangleCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
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

            // Create pipeline
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Triangle Pipeline Layout"),
                bind_group_layouts: &[],
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
            };

            callback_resources.insert(resources);
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
        render_pass.set_vertex_buffer(0, resources.vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);
    }
}

