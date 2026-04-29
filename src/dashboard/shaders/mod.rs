//! GPU shader effects for the grimoire atmospheric background (v7.4).
//!
//! Uses Iced's `Shader` widget with wgpu to render a radial vignette,
//! noise grain, floating dust motes, and gold glow during page transitions.

use iced::widget::shader;
use iced::mouse;
use iced::Rectangle;

use crate::state::Message;

// ── Uniform buffer (64 bytes, 16-byte aligned) ──────────────────

/// GPU uniform data passed to the vignette fragment shader each frame.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VignetteUniforms {
    pub resolution: [f32; 2],
    pub time: f32,
    pub vignette_strength: f32,
    pub bg_color: [f32; 4],
    pub gold_color: [f32; 4],
    pub page_alpha: f32,
    pub _pad: [f32; 3],
}

// ── Pipeline (stored in shader::Storage) ────────────────────────

/// Holds the wgpu render pipeline and uniform buffer.
/// Created once on first `prepare()`, reused thereafter.
struct VignettePipeline {
    pipeline: shader::wgpu::RenderPipeline,
    uniform_buffer: shader::wgpu::Buffer,
    bind_group: shader::wgpu::BindGroup,
}

impl VignettePipeline {
    fn new(device: &shader::wgpu::Device, format: shader::wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(shader::wgpu::ShaderModuleDescriptor {
            label: Some("grimoire_vignette"),
            source: shader::wgpu::ShaderSource::Wgsl(
                std::borrow::Cow::Borrowed(include_str!("vignette.wgsl")),
            ),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&shader::wgpu::BindGroupLayoutDescriptor {
                label: Some("vignette_bind_group_layout"),
                entries: &[shader::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: shader::wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: shader::wgpu::BindingType::Buffer {
                        ty: shader::wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&shader::wgpu::PipelineLayoutDescriptor {
                label: Some("vignette_pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&shader::wgpu::RenderPipelineDescriptor {
                label: Some("vignette_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: shader::wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: shader::wgpu::PrimitiveState {
                    topology: shader::wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: shader::wgpu::MultisampleState::default(),
                fragment: Some(shader::wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(shader::wgpu::ColorTargetState {
                        format,
                        blend: Some(shader::wgpu::BlendState::REPLACE),
                        write_mask: shader::wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        let uniform_buffer = device.create_buffer(&shader::wgpu::BufferDescriptor {
            label: Some("vignette_uniforms"),
            size: std::mem::size_of::<VignetteUniforms>() as u64,
            usage: shader::wgpu::BufferUsages::UNIFORM | shader::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&shader::wgpu::BindGroupDescriptor {
            label: Some("vignette_bind_group"),
            layout: &bind_group_layout,
            entries: &[shader::wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            uniform_buffer,
            bind_group,
        }
    }
}

// ── Primitive (implements wgpu Primitive trait) ──────────────────

/// The per-frame data sent to the GPU. Iced calls `prepare` then `render`.
#[derive(Debug)]
pub struct VignettePrimitive {
    uniforms: VignetteUniforms,
}

impl shader::Primitive for VignettePrimitive {
    fn prepare(
        &self,
        device: &shader::wgpu::Device,
        queue: &shader::wgpu::Queue,
        format: shader::wgpu::TextureFormat,
        storage: &mut shader::Storage,
        _bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        // Create pipeline on first frame
        if !storage.has::<VignettePipeline>() {
            storage.store(VignettePipeline::new(device, format));
        }

        // Upload uniforms
        let pipeline = storage.get_mut::<VignettePipeline>().unwrap();
        queue.write_buffer(
            &pipeline.uniform_buffer,
            0,
            bytemuck::bytes_of(&self.uniforms),
        );
    }

    fn render(
        &self,
        encoder: &mut shader::wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &shader::wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let pipeline = storage.get::<VignettePipeline>().unwrap();

        let mut pass = encoder.begin_render_pass(&shader::wgpu::RenderPassDescriptor {
            label: Some("vignette_pass"),
            color_attachments: &[Some(shader::wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: shader::wgpu::Operations {
                    load: shader::wgpu::LoadOp::Load,
                    store: shader::wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&pipeline.pipeline);
        pass.set_viewport(
            clip_bounds.x as f32,
            clip_bounds.y as f32,
            clip_bounds.width as f32,
            clip_bounds.height as f32,
            0.0,
            1.0,
        );
        pass.set_bind_group(0, &pipeline.bind_group, &[]);
        pass.draw(0..3, 0..1); // Full-screen triangle
    }
}

// ── Program (Iced shader widget interface) ──────────────────────

/// The shader program struct passed to `Shader::new()`.
/// Fields are set each frame from Dashboard state in `view()`.
#[derive(Debug)]
pub struct VignetteProgram {
    pub time: f32,
    pub page_alpha: f32,
    pub bg_color: [f32; 4],
    pub gold_color: [f32; 4],
}

impl shader::Program<Message> for VignetteProgram {
    type State = ();
    type Primitive = VignettePrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        VignettePrimitive {
            uniforms: VignetteUniforms {
                resolution: [bounds.width, bounds.height],
                time: self.time,
                vignette_strength: 0.7,
                bg_color: self.bg_color,
                gold_color: self.gold_color,
                page_alpha: self.page_alpha,
                _pad: [0.0; 3],
            },
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}
