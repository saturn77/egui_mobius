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
//! - Each frame, [`crate::citizen::CanvasCitizen`] calls [`paint_canvas`],
//!   which adds a [`CanvasCallback`] to the egui painter. Its `prepare`
//!   updates GPU buffers; its `paint` issues draws into egui's render
//!   pass.
//!
//! ## Status
//!
//! On the GPU: the canvas background, the procedural grid, and node
//! bodies (rect / circle / ellipse, instanced). Edges, node text,
//! ports, waypoints, and selection highlights still go through
//! [`crate::render`] on the egui painter.

use egui_wgpu::{CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};

use crate::model::{NodeKind, Scene};
use crate::render::{background_color, fill_to_color, parse_color, Viewport};

const FLAG_SHOW_GRID: u32 = 1;
const FLAG_SRGB_TARGET: u32 = 2;

/// Instance-buffer slots allocated up front; grown on demand.
const INITIAL_NODE_CAPACITY: u32 = 64;

// =============================================================================
// GPU data layouts
// =============================================================================

/// View + grid parameters handed to both canvas shaders. Mirrors the
/// `Viewport` struct in `canvas.wgsl` / `nodes.wgsl` — field order and
/// sizes must match, and the total size must stay a multiple of 16.
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
    /// Grid ink, linear RGBA (the shader applies the tier alpha).
    grid_color: [f32; 4],
    /// Full window size, egui points.
    screen_size: [f32; 2],
    grid_spacing: f32,
    dot_size: f32,
    /// 0 = lines, 1 = dots.
    grid_style: u32,
    /// Bit 0 = show grid, bit 1 = sRGB-format target.
    flags: u32,
    _pad0: u32,
    _pad1: u32,
}

/// One scene node, as uploaded to the instance buffer. Mirrors the
/// per-instance vertex attributes of `nodes.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct NodeInstance {
    /// World-space top-left corner.
    pos: [f32; 2],
    /// World-space width and height.
    size: [f32; 2],
    /// Fill, linear premultiplied RGBA.
    fill: [f32; 4],
    /// Border, linear premultiplied RGBA.
    border: [f32; 4],
    /// Border stroke width, world units.
    border_width: f32,
    /// 0 = rect, 1 = circle, 2 = ellipse.
    kind: u32,
}

/// Per-instance vertex attributes for the node pipeline.
const NODE_ATTRS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
    0 => Float32x2,  // pos
    1 => Float32x2,  // size
    2 => Float32x4,  // fill
    3 => Float32x4,  // border
    4 => Float32,    // border_width
    5 => Uint32,     // kind
];

fn node_instance(node: &crate::model::Node) -> NodeInstance {
    let fill: egui::Rgba = fill_to_color(&node.overlay.fill).into();
    let border: egui::Rgba = parse_color(&node.overlay.border.color).into();
    NodeInstance {
        pos: [node.transform.position.0, node.transform.position.1],
        size: [node.transform.size.0, node.transform.size.1],
        fill: fill.to_array(),
        border: border.to_array(),
        border_width: node.overlay.border.width,
        kind: match node.kind {
            NodeKind::Circle => 1,
            NodeKind::Ellipse => 2,
            // Path / Group fall back to a rectangle, as the CPU renderer does.
            NodeKind::Rect | NodeKind::Path(_) | NodeKind::Group(_) => 0,
        },
    }
}

// =============================================================================
// Renderer resources
// =============================================================================

/// GPU resources for the canvas. Created once by [`init`] and stored in
/// `egui_wgpu`'s callback-resources map; looked up by [`CanvasCallback`]
/// each frame.
pub struct GraficaRenderer {
    /// Fullscreen background + grid pipeline.
    canvas_pipeline: wgpu::RenderPipeline,
    /// Instanced node-body pipeline.
    node_pipeline: wgpu::RenderPipeline,
    /// Shared viewport uniform, bound by both pipelines.
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    /// Node instance buffer and its capacity, in instances.
    node_instances: wgpu::Buffer,
    node_capacity: u32,
    /// True when the render target is an sRGB texture format — the GPU
    /// then converts linear → sRGB on store, so the shader must not.
    srgb_target: bool,
}

/// Premultiplied-alpha "over" blend — the shaders emit premultiplied
/// colors, matching egui's own convention.
fn premultiplied_blend() -> wgpu::BlendState {
    let component = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    wgpu::BlendState { color: component, alpha: component }
}

impl GraficaRenderer {
    fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let canvas_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("grafica.canvas.shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("canvas.wgsl").into()),
        });
        let node_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("grafica.nodes.shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("nodes.wgsl").into()),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("grafica.uniform.bgl"),
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

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grafica.viewport.uniform"),
            size: std::mem::size_of::<ViewportUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("grafica.uniform.bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("grafica.pipeline.layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let blend = premultiplied_blend();
        let target = wgpu::ColorTargetState {
            format: target_format,
            blend: Some(blend),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let canvas_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grafica.canvas.pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &canvas_shader,
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
                module: &canvas_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(target.clone())],
            }),
            multiview_mask: None,
            cache: None,
        });

        let node_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grafica.nodes.pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &node_shader,
                entry_point: Some("vs_node"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<NodeInstance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &NODE_ATTRS,
                }],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &node_shader,
                entry_point: Some("fs_node"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(target)],
            }),
            multiview_mask: None,
            cache: None,
        });

        let node_instances = new_node_buffer(device, INITIAL_NODE_CAPACITY);

        Self {
            canvas_pipeline,
            node_pipeline,
            uniform_buffer,
            bind_group,
            node_instances,
            node_capacity: INITIAL_NODE_CAPACITY,
            srgb_target: target_format.is_srgb(),
        }
    }

    /// Grow the node instance buffer if `needed` exceeds its capacity.
    fn reserve_nodes(&mut self, device: &wgpu::Device, needed: u32) {
        if needed <= self.node_capacity {
            return;
        }
        let capacity = needed.next_power_of_two();
        self.node_instances = new_node_buffer(device, capacity);
        self.node_capacity = capacity;
    }
}

fn new_node_buffer(device: &wgpu::Device, capacity: u32) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("grafica.nodes.instances"),
        size: capacity as u64 * std::mem::size_of::<NodeInstance>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
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
/// Calling it twice simply replaces the renderer.
pub fn init(render_state: &RenderState) {
    let renderer = GraficaRenderer::new(&render_state.device, render_state.target_format);
    render_state
        .renderer
        .write()
        .callback_resources
        .insert(renderer);
}

// =============================================================================
// Per-frame callback
// =============================================================================

/// Per-frame paint callback for the canvas. Carries the frame's view
/// state and node instances by value; the GPU resources live in
/// [`GraficaRenderer`].
struct CanvasCallback {
    uniform: ViewportUniform,
    nodes: Vec<NodeInstance>,
}

impl CallbackTrait for CanvasCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: &ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(renderer) = resources.get_mut::<GraficaRenderer>() {
            let mut uniform = self.uniform;
            // egui's render pass covers the whole surface — take its size
            // from the authoritative screen descriptor, in egui points.
            uniform.pixels_per_point = screen.pixels_per_point;
            uniform.screen_size = [
                screen.size_in_pixels[0] as f32 / screen.pixels_per_point,
                screen.size_in_pixels[1] as f32 / screen.pixels_per_point,
            ];
            if renderer.srgb_target {
                uniform.flags |= FLAG_SRGB_TARGET;
            }
            queue.write_buffer(&renderer.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

            if !self.nodes.is_empty() {
                renderer.reserve_nodes(device, self.nodes.len() as u32);
                queue.write_buffer(
                    &renderer.node_instances,
                    0,
                    bytemuck::cast_slice(&self.nodes),
                );
            }
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        let Some(renderer) = resources.get::<GraficaRenderer>() else {
            return;
        };
        render_pass.set_bind_group(0, &renderer.bind_group, &[]);

        // Background + grid: one fullscreen triangle.
        render_pass.set_pipeline(&renderer.canvas_pipeline);
        render_pass.draw(0..3, 0..1);

        // Node bodies: one instanced quad per node.
        if !self.nodes.is_empty() {
            render_pass.set_pipeline(&renderer.node_pipeline);
            render_pass.set_vertex_buffer(0, renderer.node_instances.slice(..));
            render_pass.draw(0..6, 0..self.nodes.len() as u32);
        }
    }
}

/// Paint the canvas — background, grid, and node bodies — on the GPU,
/// over `rect`.
///
/// Adds a paint callback to `painter`. If [`init`] was never called the
/// callback finds no [`GraficaRenderer`] and silently draws nothing —
/// callers that need a guaranteed result keep the CPU fallback.
///
/// The surface size and pixels-per-point are resolved later, in the
/// callback's `prepare`, from `egui_wgpu`'s screen descriptor.
pub fn paint_canvas(
    painter: &egui::Painter,
    rect: egui::Rect,
    viewport: &Viewport,
    scene: &Scene,
) {
    let settings = &scene.settings;
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
        // Resolved in the callback's `prepare` from the screen descriptor.
        pixels_per_point: 1.0,
        bg_color: bg.to_array(),
        grid_color: ink.to_array(),
        screen_size: [0.0, 0.0],
        grid_spacing: settings.grid_spacing,
        dot_size: settings.dot_size,
        grid_style: match settings.grid_style {
            crate::model::GridStyle::Lines => 0,
            crate::model::GridStyle::Dots => 1,
        },
        flags: if show_grid { FLAG_SHOW_GRID } else { 0 },
        _pad0: 0,
        _pad1: 0,
    };

    let nodes: Vec<NodeInstance> = scene.nodes.iter().map(node_instance).collect();

    painter.add(egui_wgpu::Callback::new_paint_callback(
        rect,
        CanvasCallback { uniform, nodes },
    ));
}
