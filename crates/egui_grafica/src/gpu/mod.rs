//! Retained wgpu rendering pipeline — the `gpu` cargo feature.
//!
//! This module is the wgpu counterpart of [`crate::render`]. Where
//! `render.rs` re-tessellates the scene on the CPU every frame, this
//! module uploads geometry to VRAM and lets pan / zoom be a uniform
//! update. See `develop/gpu_rendering_plan.md` for the staged plan.
//!
//! ## Lifecycle
//!
//! - The application calls [`init`] once, from `eframe::CreationContext`,
//!   passing the `wgpu` render state. It constructs a [`GraficaRenderer`]
//!   and stores it in `egui_wgpu`'s callback-resources type map.
//! - Each frame, [`crate::citizen::CanvasCitizen`] adds a [`CanvasCallback`]
//!   to the egui painter. Its `prepare` updates GPU buffers; its `paint`
//!   issues draws into egui's render pass.
//!
//! ## Status
//!
//! The canvas background and the grid are drawn on the GPU — a single
//! fullscreen quad whose fragment shader computes the grid per-pixel
//! from the viewport transform. Text, edges, selection, and nodes still
//! go through [`crate::render`].

use egui_wgpu::{CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};

use crate::model::{CanvasSettings, GridStyle};
use crate::render::{background_color, Viewport};

const FLAG_SHOW_GRID: u32 = 1;
const FLAG_SRGB_TARGET: u32 = 2;

/// View + grid parameters handed to the canvas shader. Mirrors the
/// `Viewport` struct in `canvas.wgsl` — field order and sizes must match.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewportUniform {
    /// Screen point where world `(0, 0)` lands.
    origin: [f32; 2],
    /// Pixels per world unit.
    zoom: f32,
    /// egui points-to-physical-pixels ratio.
    pixels_per_point: f32,
    /// Canvas background, linear premultiplied RGBA.
    bg_color: [f32; 4],
    /// Grid ink, linear premultiplied RGBA (Phase 1).
    grid_color: [f32; 4],
    grid_spacing: f32,
    dot_size: f32,
    /// 0 = lines, 1 = dots.
    grid_style: u32,
    /// Bit 0 = show grid, bit 1 = sRGB-format target.
    flags: u32,
}

/// GPU resources for the canvas pipeline. Created once by [`init`] and
/// stored in `egui_wgpu`'s callback-resources map; looked up by
/// [`CanvasCallback`] each frame.
pub struct GraficaRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    /// True when the render target is an sRGB texture format — the GPU
    /// then converts linear → sRGB on store, so the shader must not.
    srgb_target: bool,
}

impl GraficaRenderer {
    fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("grafica.canvas.shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("canvas.wgsl").into()),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("grafica.canvas.bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grafica.canvas.uniform"),
            size: std::mem::size_of::<ViewportUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("grafica.canvas.bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("grafica.canvas.layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        // Premultiplied-alpha blend — the shader emits premultiplied
        // colors, matching egui's own convention.
        let blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grafica.canvas.pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            // eframe's wgpu egui pass is single-sampled by default. If an
            // app enables MSAA this count must follow it.
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            uniform_buffer,
            bind_group,
            srgb_target: target_format.is_srgb(),
        }
    }
}

/// Construct the [`GraficaRenderer`] and register it with `egui_wgpu`.
///
/// Call once at startup, from `eframe::CreationContext`:
///
/// ```ignore
/// Box::new(|cc| {
///     if let Some(rs) = cc.wgpu_render_state.as_ref() {
///         egui_grafica::gpu::init(rs);
///     }
///     Ok(Box::new(MyApp::new(cc)))
/// });
/// ```
///
/// A no-op-safe call: invoking it twice simply replaces the renderer.
pub fn init(render_state: &RenderState) {
    let renderer = GraficaRenderer::new(&render_state.device, render_state.target_format);
    render_state
        .renderer
        .write()
        .callback_resources
        .insert(renderer);
}

/// Per-frame paint callback for the canvas. Carries the view state by
/// value; the GPU resources live in [`GraficaRenderer`].
struct CanvasCallback {
    uniform: ViewportUniform,
}

impl CallbackTrait for CanvasCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen: &ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(renderer) = resources.get::<GraficaRenderer>() {
            let mut uniform = self.uniform;
            if renderer.srgb_target {
                uniform.flags |= FLAG_SRGB_TARGET;
            }
            queue.write_buffer(&renderer.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        if let Some(renderer) = resources.get::<GraficaRenderer>() {
            render_pass.set_pipeline(&renderer.pipeline);
            render_pass.set_bind_group(0, &renderer.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}

/// Paint the canvas background and grid on the GPU, over `rect`.
///
/// Adds a paint callback to `painter`. If [`init`] was never called the
/// callback finds no [`GraficaRenderer`] and silently draws nothing —
/// callers that need a guaranteed fill should keep a CPU fallback.
pub fn paint_canvas(
    painter: &egui::Painter,
    rect: egui::Rect,
    viewport: &Viewport,
    settings: &CanvasSettings,
    pixels_per_point: f32,
) {
    let bg: egui::Rgba = background_color(settings.background).into();
    // Grid ink contrasts with the background — light ink on dark canvases,
    // dark ink on light ones. Mirrors `render::paint_grid`.
    let ink: egui::Rgba = if settings.background.is_dark() {
        egui::Color32::WHITE
    } else {
        egui::Color32::BLACK
    }
    .into();

    // The grid auto-hides once lines would be closer than 4 points apart,
    // matching the CPU renderer's noise cutoff.
    let show_grid = settings.show_grid
        && settings.grid_spacing > 0.0
        && settings.grid_spacing * viewport.zoom >= 4.0;

    let uniform = ViewportUniform {
        origin: [viewport.origin.x, viewport.origin.y],
        zoom: viewport.zoom,
        pixels_per_point,
        bg_color: bg.to_array(),
        grid_color: ink.to_array(),
        grid_spacing: settings.grid_spacing,
        dot_size: settings.dot_size,
        grid_style: match settings.grid_style {
            GridStyle::Lines => 0,
            GridStyle::Dots => 1,
        },
        flags: if show_grid { FLAG_SHOW_GRID } else { 0 },
    };
    painter.add(egui_wgpu::Callback::new_paint_callback(
        rect,
        CanvasCallback { uniform },
    ));
}
