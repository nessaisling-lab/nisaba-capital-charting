//! GPU shader effects for the grimoire dashboard.
//!
//! - `VignetteProgram` (v7.4) — radial vignette, noise grain, dust motes, gold glow
//! - `NatalWheel3DProgram` (v8.0) — perspective-tilted zodiac chart with SDF planets

use iced::widget::shader;
use iced::mouse;
use iced::Rectangle;

use nisaba_engine::models::{DailyTransit, NatalPosition};
use crate::state::Message;

// Re-export wgpu from iced (moved from shader::wgpu to iced::wgpu in 0.14)
use iced::wgpu;

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

// ── Pipeline (Iced 0.14: implements shader::Pipeline trait) ─────

/// Holds the wgpu render pipeline and uniform buffer.
/// Created once on first frame via Pipeline::new(), reused thereafter.
pub struct VignettePipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl shader::Pipeline for VignettePipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("grimoire_vignette"),
            source: wgpu::ShaderSource::Wgsl(
                std::borrow::Cow::Borrowed(include_str!("vignette.wgsl")),
            ),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("vignette_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("vignette_pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("vignette_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vignette_uniforms"),
            size: std::mem::size_of::<VignetteUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vignette_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
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

/// The per-frame data sent to the GPU. Iced calls `prepare` then `draw`/`render`.
#[derive(Debug)]
pub struct VignettePrimitive {
    uniforms: VignetteUniforms,
}

impl shader::Primitive for VignettePrimitive {
    type Pipeline = VignettePipeline;

    fn prepare(
        &self,
        pipeline: &mut VignettePipeline,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        queue.write_buffer(
            &pipeline.uniform_buffer,
            0,
            bytemuck::bytes_of(&self.uniforms),
        );
    }

    fn render(
        &self,
        pipeline: &VignettePipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("vignette_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
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
    // v9.0: active zodiac sign (0-11)
    pub active_sign: f32,
    // v11.1: chart layer visibility toggles (0.0 = hidden, 1.0 = visible)
    pub show_natal: f32,
    pub show_transit: f32,
    pub show_aspects: f32,
}

// ── Pipeline ───────────────────────────────────────────────────────

pub struct NatalWheel3DPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl shader::Pipeline for NatalWheel3DPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("natal_wheel_3d"),
            source: wgpu::ShaderSource::Wgsl(
                std::borrow::Cow::Borrowed(include_str!("natal_wheel_3d.wgsl")),
            ),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("natal_wheel_3d_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("natal_wheel_3d_pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("natal_wheel_3d_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("natal_wheel_3d_uniforms"),
            size: std::mem::size_of::<NatalWheel3DUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("natal_wheel_3d_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
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
    type Pipeline = NatalWheel3DPipeline;

    fn prepare(
        &self,
        pipeline: &mut NatalWheel3DPipeline,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        queue.write_buffer(
            &pipeline.uniform_buffer,
            0,
            bytemuck::bytes_of(&self.uniforms),
        );
    }

    fn render(
        &self,
        pipeline: &NatalWheel3DPipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("natal_wheel_3d_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
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
    // v11.1: chart layer visibility
    pub show_natal: bool,
    pub show_transit: bool,
    pub show_aspects: bool,
    pub show_retrogrades: bool,
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

        // Pack transit planets (v11.1: clear retrograde flag when toggle is off)
        let mut transit_planets = [[0.0f32; 4]; 13];
        for (i, pos) in self.transit_positions.iter().take(13).enumerate() {
            transit_planets[i] = [
                pos.longitude as f32,
                if pos.retrograde && self.show_retrogrades { 1.0 } else { 0.0 },
                i as f32,
                0.0,
            ];
        }

        NatalWheel3DPrimitive {
            uniforms: NatalWheel3DUniforms {
                resolution: [bounds.width, bounds.height],
                time: self.time,
                camera_tilt: 0.10, // v11.6.C — sphere not oval (was 0.32)
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
                show_natal: if self.show_natal { 1.0 } else { 0.0 },
                show_transit: if self.show_transit { 1.0 } else { 0.0 },
                show_aspects: if self.show_aspects { 1.0 } else { 0.0 },
            },
        }
    }
}
