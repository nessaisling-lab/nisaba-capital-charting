//! GPU shader effects for the grimoire dashboard.
//!
//! - `VignetteProgram` (v7.4) — radial vignette, noise grain, dust motes, gold glow
//! - `NatalWheel3DProgram` (v8.0) — perspective-tilted zodiac chart with SDF planets

use iced::widget::shader;
use iced::mouse;
use iced::Rectangle;

use pursuit_week4_automation::models::{DailyTransit, NatalPosition};
use crate::state::Message;

// ── Uniform buffer (64 bytes, 16-byte aligned) ──────────────────

/// GPU uniform data passed to the vignette fragment shader each frame.
/// v9.0: added mouse_pos for cursor-reactive dust motes (replaces 2 pad floats).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VignetteUniforms {
    pub resolution: [f32; 2],
    pub time: f32,
    pub vignette_strength: f32,
    pub bg_color: [f32; 4],
    pub gold_color: [f32; 4],
    pub page_alpha: f32,
    pub _pad0: f32,
    pub mouse_pos: [f32; 2],  // v9.0: cursor UV position [0,1]
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
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        // Convert cursor to UV [0,1] space for dust mote repulsion (v9.0)
        let mouse_uv = cursor
            .position_in(bounds)
            .map(|p| [p.x / bounds.width, p.y / bounds.height])
            .unwrap_or([0.5, 0.5]);  // center when cursor outside

        VignettePrimitive {
            uniforms: VignetteUniforms {
                resolution: [bounds.width, bounds.height],
                time: self.time,
                vignette_strength: 0.7,
                bg_color: self.bg_color,
                gold_color: self.gold_color,
                page_alpha: self.page_alpha,
                _pad0: 0.0,
                mouse_pos: mouse_uv,
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

// ════════════════════════════════════════════════════════════════════
// 3D Natal Chart — "The Observatory" (v8.0)
// ════════════════════════════════════════════════════════════════════

// ── Uniform buffer (496 bytes, 16-byte aligned) ────────────────────

/// GPU uniform data for the 3D natal chart fragment shader (512 bytes).
/// Planet slots: each `[f32; 4]` = `[longitude_deg, is_retrograde, planet_idx, 0]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NatalWheel3DUniforms {
    pub resolution: [f32; 2],
    pub time: f32,
    pub camera_tilt: f32,
    pub bg_color: [f32; 4],
    pub gold_color: [f32; 4],
    pub transit_color: [f32; 4],
    pub natal_planets: [[f32; 4]; 13],
    pub transit_planets: [[f32; 4]; 13],
    pub natal_count: f32,
    pub transit_count: f32,
    pub retro_r: f32,
    pub retro_g: f32,
    // v9.0: active zodiac sign (0-11) + padding to 512 bytes
    pub active_sign: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

// ── Pipeline ───────────────────────────────────────────────────────

struct NatalWheel3DPipeline {
    pipeline: shader::wgpu::RenderPipeline,
    uniform_buffer: shader::wgpu::Buffer,
    bind_group: shader::wgpu::BindGroup,
}

impl NatalWheel3DPipeline {
    fn new(device: &shader::wgpu::Device, format: shader::wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(shader::wgpu::ShaderModuleDescriptor {
            label: Some("natal_wheel_3d"),
            source: shader::wgpu::ShaderSource::Wgsl(
                std::borrow::Cow::Borrowed(include_str!("natal_wheel_3d.wgsl")),
            ),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&shader::wgpu::BindGroupLayoutDescriptor {
                label: Some("natal_wheel_3d_bind_group_layout"),
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
                label: Some("natal_wheel_3d_pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&shader::wgpu::RenderPipelineDescriptor {
                label: Some("natal_wheel_3d_pipeline"),
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
            label: Some("natal_wheel_3d_uniforms"),
            size: std::mem::size_of::<NatalWheel3DUniforms>() as u64,
            usage: shader::wgpu::BufferUsages::UNIFORM | shader::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&shader::wgpu::BindGroupDescriptor {
            label: Some("natal_wheel_3d_bind_group"),
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

// ── Primitive ──────────────────────────────────────────────────────

#[derive(Debug)]
pub struct NatalWheel3DPrimitive {
    uniforms: NatalWheel3DUniforms,
}

impl shader::Primitive for NatalWheel3DPrimitive {
    fn prepare(
        &self,
        device: &shader::wgpu::Device,
        queue: &shader::wgpu::Queue,
        format: shader::wgpu::TextureFormat,
        storage: &mut shader::Storage,
        _bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        if !storage.has::<NatalWheel3DPipeline>() {
            storage.store(NatalWheel3DPipeline::new(device, format));
        }
        let pipeline = storage.get_mut::<NatalWheel3DPipeline>().unwrap();
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
        let pipeline = storage.get::<NatalWheel3DPipeline>().unwrap();

        let mut pass = encoder.begin_render_pass(&shader::wgpu::RenderPassDescriptor {
            label: Some("natal_wheel_3d_pass"),
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
        pass.draw(0..3, 0..1);
    }
}

// ── Program (Iced shader widget interface) ─────────────────────────

/// 3D natal chart shader program. Passed to `Shader::new()` in the view.
/// Packs planet positions into GPU-friendly uniform arrays in `draw()`.
#[derive(Debug)]
pub struct NatalWheel3DProgram {
    pub time: f32,
    pub natal_positions: Vec<NatalPosition>,
    pub transit_positions: Vec<DailyTransit>,
    pub bg_color: [f32; 4],
    pub gold_color: [f32; 4],
    pub transit_color: [f32; 4],
    pub retro_color: [f32; 4],
    pub active_sign: f32,  // v9.0: 0-11, zodiac sign with current Sun transit
}

impl shader::Program<Message> for NatalWheel3DProgram {
    type State = ();
    type Primitive = NatalWheel3DPrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        // Pack natal planets into [[f32; 4]; 13]
        let mut natal_planets = [[0.0f32; 4]; 13];
        for (i, pos) in self.natal_positions.iter().take(13).enumerate() {
            natal_planets[i] = [
                pos.longitude as f32,
                if pos.retrograde { 1.0 } else { 0.0 },
                i as f32,
                0.0,
            ];
        }

        // Pack transit planets
        let mut transit_planets = [[0.0f32; 4]; 13];
        for (i, pos) in self.transit_positions.iter().take(13).enumerate() {
            transit_planets[i] = [
                pos.longitude as f32,
                if pos.retrograde { 1.0 } else { 0.0 },
                i as f32,
                0.0,
            ];
        }

        NatalWheel3DPrimitive {
            uniforms: NatalWheel3DUniforms {
                resolution: [bounds.width, bounds.height],
                time: self.time,
                camera_tilt: 0.32,
                bg_color: self.bg_color,
                gold_color: self.gold_color,
                transit_color: self.transit_color,
                natal_planets,
                transit_planets,
                natal_count: self.natal_positions.len().min(13) as f32,
                transit_count: self.transit_positions.len().min(13) as f32,
                retro_r: self.retro_color[0],
                retro_g: self.retro_color[1],
                active_sign: self.active_sign,
                _pad1: 0.0,
                _pad2: 0.0,
                _pad3: 0.0,
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
