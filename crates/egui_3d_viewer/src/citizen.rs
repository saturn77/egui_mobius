//! The `ViewerCitizen` — citizen wrapper around the render3d
//! primitives. Owns the camera, GPU resources, input drag state,
//! and a `Dynamic<ReactiveViewerState>` for cross-panel observability
//! of the atom toggles.
//!
//! The empty default scene is just an XYZ axes gizmo + a ground grid
//! at Z=0. Consumer apps can supply their own meshes through a
//! follow-up scene-injection API; for now the viewer is a working
//! orbit-zoom-pan playground with the standard creature comforts:
//!
//! - Mouse left-drag → orbit
//! - Mouse wheel → zoom
//! - Right-mouse-drag → zoom-to-region with a yellow box overlay
//! - Double-click → reset view
//! - `G` (canvas hovered) → toggle grid
//! - `M` (canvas hovered) → toggle measure mode; left-drag while
//!   measure-active draws a distance line on the Z=0 plane

use std::sync::{Arc, Mutex};

use egui_citizen::{Citizen, CitizenId, CitizenState};
use egui_mobius_reactive::Dynamic;
use glow::HasContext as _;
use nalgebra::Vector3;

use crate::axes::axes_vertices;
use crate::camera::{project, unproject_to_z0, Camera};
use crate::grid::grid_vertices;
use crate::mesh::ColoredMesh;
use crate::renderer::UnlitProgram;
use crate::state::ReactiveViewerState;

/// Default grid colour — a desaturated cool grey that reads as
/// scene structure without competing with rendered content.
const GRID_COLOR: [f32; 3] = [0.28, 0.30, 0.35];
/// Default axes gizmo length (world units). Sized for an empty
/// scene; consumers fitting a real bounding box should also
/// rescale this via `set_axes_length`.
const DEFAULT_AXES_LEN: f32 = 3.0;
/// Default grid extent (world units, half-width). 5.0 → a 10×10
/// region centred at origin.
const DEFAULT_GRID_EXTENT: f32 = 5.0;
/// Default grid line spacing (world units).
const DEFAULT_GRID_STEP: f32 = 1.0;

/// 3D viewer citizen. Wraps the render3d primitives in a citizen
/// pattern: persistent camera + GL handles + drag state on the
/// struct, atom toggles in a `Dynamic<ReactiveViewerState>` cell,
/// and the standard `Citizen` trait integration.
pub struct ViewerCitizen {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    state: Dynamic<ReactiveViewerState>,
    camera: Camera,
    /// Lazily created on the first frame where a glow context is
    /// available — egui_glow only hands one to us inside the paint
    /// callback, and we need to allocate buffers + compile shaders
    /// before the first draw.
    gpu: Option<Arc<Mutex<GpuResources>>>,
    /// Right-mouse-drag zoom-to-region: anchor pixel on drag start,
    /// live pixel during drag. On release we un-project both corners
    /// onto the Z=0 plane and retarget + rescale to frame the
    /// selection.
    zoom_box_start: Option<egui::Pos2>,
    zoom_box_current: Option<egui::Pos2>,
    /// Measure tool state — drag endpoints in world space, latched
    /// after release so the user can re-enter measure mode and see
    /// what was last measured.
    measure_start: Option<Vector3<f32>>,
    measure_end: Option<Vector3<f32>>,
    measure_dragging: bool,
    /// Current axes gizmo length. Tracks the scene size; consumers
    /// can override with `set_axes_length`.
    axes_len: f32,
    /// Z-coordinate the axes gizmo sits at. Default keeps it just
    /// above the grid plane (avoiding Z-fighting); when a scene is
    /// set this snaps to the scene's `max.z` so the gizmo rests
    /// flush on the top surface of the rendered geometry rather than
    /// hidden under it.
    axes_z_base: f32,
    /// Set whenever `axes_len` or `axes_z_base` changes; the next
    /// `show()` re-uploads the axes mesh and clears the flag.
    axes_dirty: bool,
    /// Pending scene-triangle upload. Stashed by `set_scene_triangles`
    /// and drained inside `show()` once a glow context is available.
    /// An empty vec clears the slot — the mesh stays allocated but
    /// the ready flag goes false so it's skipped at draw time.
    pending_scene_triangles: Option<Vec<f32>>,
    /// Pending scene-line upload. Same lifecycle as the triangle
    /// pending slot — set by `set_scene_lines`, drained in `show()`.
    pending_scene_lines: Option<Vec<f32>>,
    /// Bounding box of the most recent scene set via
    /// `set_scene_triangles` or `set_scene_lines`. Computed at set
    /// time from the xyz columns of the buffer; powers the
    /// `fit_view_to_scene` action that double-click and the
    /// `reset_view_requested` flag invoke.
    scene_bbox: Option<SceneBbox>,
}

#[derive(Clone, Copy)]
struct SceneBbox {
    min: Vector3<f32>,
    max: Vector3<f32>,
}

impl SceneBbox {
    fn from_xyz_rgb(verts: &[f32]) -> Option<Self> {
        if verts.is_empty() {
            return None;
        }
        let mut min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for chunk in verts.chunks_exact(6) {
            min.x = min.x.min(chunk[0]);
            min.y = min.y.min(chunk[1]);
            min.z = min.z.min(chunk[2]);
            max.x = max.x.max(chunk[0]);
            max.y = max.y.max(chunk[1]);
            max.z = max.z.max(chunk[2]);
        }
        Some(Self { min, max })
    }

    fn merge(self, other: Self) -> Self {
        Self {
            min: Vector3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vector3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    fn size(self) -> Vector3<f32> {
        self.max - self.min
    }

    fn center(self) -> Vector3<f32> {
        (self.min + self.max) * 0.5
    }
}

struct GpuResources {
    unlit: UnlitProgram,
    axes: ColoredMesh,
    grid: ColoredMesh,
    /// User-supplied scene triangles. Pre-allocated empty so uploads
    /// have a slot to land in without re-creating GpuResources.
    scene_triangles: ColoredMesh,
    scene_triangles_ready: bool,
    /// User-supplied scene lines. Same shape as `scene_triangles`
    /// but with the `LINES` primitive — useful for wireframe overlays
    /// or vector-style scene content.
    scene_lines: ColoredMesh,
    scene_lines_ready: bool,
}

impl ViewerCitizen {
    /// Build a viewer with a fresh default state cell.
    pub fn new(id: impl Into<String>, citizen_state: CitizenState) -> Self {
        Self::with_state(
            id,
            citizen_state,
            Dynamic::new(ReactiveViewerState::default()),
        )
    }

    /// Build a viewer wrapped around an existing reactive state cell —
    /// useful when the consuming app's `SharedState` already carries
    /// the `Dynamic<ReactiveViewerState>` for cross-panel observation.
    pub fn with_state(
        id: impl Into<String>,
        citizen_state: CitizenState,
        state: Dynamic<ReactiveViewerState>,
    ) -> Self {
        Self {
            citizen_id: CitizenId::new(id.into()),
            citizen_state,
            state,
            camera: Camera::default(),
            gpu: None,
            zoom_box_start: None,
            zoom_box_current: None,
            measure_start: None,
            measure_end: None,
            measure_dragging: false,
            axes_len: DEFAULT_AXES_LEN,
            axes_z_base: 0.001,
            axes_dirty: false,
            pending_scene_triangles: None,
            pending_scene_lines: None,
            scene_bbox: None,
        }
    }

    /// Borrow the reactive atom state cell — observe or mutate it
    /// from anywhere in the consuming app.
    pub fn state(&self) -> &Dynamic<ReactiveViewerState> {
        &self.state
    }

    /// Borrow the camera. Useful for consumers who want to drive
    /// the view programmatically — set `target`, call `fit_to_bbox`
    /// after loading a scene, etc.
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Mutably borrow the camera.
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Override the axes gizmo length. Consumers loading a scene
    /// should call this with ~15 % of the scene's largest dimension
    /// so the gizmo reads as a reference instead of a stub.
    ///
    /// Re-upload happens lazily on the next frame: we drop the cached
    /// GPU resources so the next `show()` recompiles with the new
    /// length. Cheap because the cached resources are a shader plus
    /// two small line meshes.
    pub fn set_axes_length(&mut self, len: f32) {
        self.axes_len = len.max(0.1);
        self.axes_dirty = true;
    }

    /// Replace the scene triangle mesh — `vertices` is a flat buffer
    /// in `xyz rgb` stride (six floats per vertex). Pass an empty
    /// vec to clear the slot. Upload is deferred until the next
    /// `show()` call, since a glow context is only available there.
    ///
    /// Length must be a multiple of 6; non-multiple slices panic
    /// at upload time inside `ColoredMesh::upload`.
    ///
    /// The scene bounding box is recomputed at set time so
    /// `fit_view_to_scene` and double-click reset can frame the
    /// new geometry without the consumer doing it manually.
    pub fn set_scene_triangles(&mut self, vertices: Vec<f32>) {
        let new_bbox = SceneBbox::from_xyz_rgb(&vertices);
        self.scene_bbox = match (self.scene_bbox, new_bbox) {
            (Some(_existing_lines_only), Some(tris)) => {
                // If lines have already populated the bbox we conservatively
                // overwrite — triangles are typically the dominant scene.
                Some(tris)
            }
            (None, Some(tris)) => Some(tris),
            (existing, None) => existing,
        };
        // Snap the axes gizmo to the top of the scene so it rests on
        // the top surface of the rendered geometry instead of being
        // hidden inside it. Falls back to the default tiny offset
        // when no scene is loaded.
        if let Some(bbox) = &self.scene_bbox {
            let new_base = bbox.max.z.max(0.001);
            if (new_base - self.axes_z_base).abs() > 1e-4 {
                self.axes_z_base = new_base;
                self.axes_dirty = true;
            }
        }
        self.pending_scene_triangles = Some(vertices);
    }

    /// Replace the scene line mesh — same `xyz rgb` stride as
    /// `set_scene_triangles`, rendered with the `LINES` primitive.
    /// Pass an empty vec to clear. Bbox handling mirrors
    /// `set_scene_triangles`: if a triangle bbox already exists the
    /// line bbox is merged into it.
    pub fn set_scene_lines(&mut self, vertices: Vec<f32>) {
        if let Some(lines_bbox) = SceneBbox::from_xyz_rgb(&vertices) {
            self.scene_bbox = Some(match self.scene_bbox {
                Some(existing) => existing.merge(lines_bbox),
                None => lines_bbox,
            });
        }
        self.pending_scene_lines = Some(vertices);
    }

    /// Clear both scene mesh slots and forget the cached bbox — the
    /// next frame draws axes + grid only.
    pub fn clear_scene(&mut self) {
        self.pending_scene_triangles = Some(Vec::new());
        self.pending_scene_lines = Some(Vec::new());
        self.scene_bbox = None;
    }

    /// Reset the camera to a tilted top-down orientation centred on
    /// the current scene's bounding box and zoom-fit so the entire
    /// scene fills the viewport. No-op on the rotation/target/zoom
    /// pieces that depend on the bbox if no scene has been set —
    /// falls back to `Camera::reset_top_down`.
    pub fn fit_view_to_scene(&mut self) {
        self.camera.reset_top_down();
        if let Some(bbox) = self.scene_bbox {
            let size = bbox.size();
            let centre = bbox.center();
            self.camera.target = Vector3::new(centre.x, centre.y, 0.0);
            self.camera.fit_to_bbox(size.x.abs().max(0.1), size.y.abs().max(0.1));
        }
    }

    /// Render one frame of the viewer. Allocates the canvas, handles
    /// input, queues the paint callback, and draws screen-space
    /// overlays — measure-tool line, axis labels, zoom-box rectangle.
    ///
    /// Pass `None` for `gl` when the eframe wgpu backend is active —
    /// the canvas falls back to a "requires glow backend" message.
    pub fn show(&mut self, ui: &mut egui::Ui, gl: Option<&Arc<glow::Context>>) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

        let bg = self.state.get().background_color;
        ui.painter().rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgb(bg[0], bg[1], bg[2]),
        );

        // Compute MVP early so the same matrix drives input handling
        // (un-project for measure + zoom-box) and the paint callback.
        let mvp = self.camera.mvp(rect);
        let mvp_inv = mvp.try_inverse();

        // Consume the reset_view command flag — reset orientation
        // AND zoom-fit to the scene bbox if there is one.
        if self.state.get().reset_view_requested {
            self.fit_view_to_scene();
            self.state.get().reset_view_requested = false;
        }

        let measure_active = self.state.get().measure_active;

        // ── Input handling ────────────────────────────────────────
        // Primary-button: orbit (default) OR draw measure line
        // (when measure mode is active). Double-click always resets.
        if measure_active {
            if response.drag_started_by(egui::PointerButton::Primary) {
                if let (Some(p), Some(inv)) = (response.interact_pointer_pos(), mvp_inv) {
                    if let Some(w) = unproject_to_z0(&inv, rect, p) {
                        self.measure_start = Some(w);
                        self.measure_end = Some(w);
                        self.measure_dragging = true;
                    }
                }
            }
            if self.measure_dragging && response.dragged_by(egui::PointerButton::Primary) {
                if let (Some(p), Some(inv)) = (response.interact_pointer_pos(), mvp_inv) {
                    if let Some(w) = unproject_to_z0(&inv, rect, p) {
                        self.measure_end = Some(w);
                    }
                }
            }
            if response.drag_stopped_by(egui::PointerButton::Primary) {
                self.measure_dragging = false;
            }
        } else if response.dragged_by(egui::PointerButton::Primary) {
            self.camera.orbit(response.drag_delta());
        }

        // Double-click → reset orientation AND zoom-fit to the scene
        // bbox (so the whole object frames in the viewport, not just
        // the rotation axis going through its centre).
        if response.double_clicked_by(egui::PointerButton::Primary) {
            self.fit_view_to_scene();
        }

        // Right-mouse-drag → zoom-to-region. Anchor on start, track
        // live position during drag, commit on release.
        if response.drag_started_by(egui::PointerButton::Secondary) {
            if let Some(p) = response.interact_pointer_pos() {
                self.zoom_box_start = Some(p);
                self.zoom_box_current = Some(p);
            }
        }
        if response.dragged_by(egui::PointerButton::Secondary) {
            if let Some(p) = response.interact_pointer_pos() {
                self.zoom_box_current = Some(p);
            }
        }
        let zoom_box_released = response.drag_stopped_by(egui::PointerButton::Secondary);

        // Hover-gated input: scroll-wheel zoom + G/M hotkeys. Gating
        // on hover means typing G or M in another panel doesn't flip
        // settings under the user's nose.
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 {
                self.camera.zoom_by(1.0 + scroll * 0.001);
            }
            if ui.input(|i| i.key_pressed(egui::Key::G)) {
                let mut s = self.state.get();
                s.show_grid = !s.show_grid;
                self.state.set(s);
            }
            if ui.input(|i| i.key_pressed(egui::Key::M)) {
                let mut s = self.state.get();
                s.measure_active = !s.measure_active;
                let now_active = s.measure_active;
                self.state.set(s);
                self.measure_dragging = false;
                if now_active {
                    // Entering: clear previous endpoints.
                    self.measure_start = None;
                    self.measure_end = None;
                }
            }
        }

        // ── Bail with a friendly note if egui_glow isn't active ──
        let Some(gl) = gl else {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "3D view requires the eframe `glow` backend",
                egui::FontId::proportional(14.0),
                egui::Color32::YELLOW,
            );
            return;
        };

        // ── Lazy GPU init ─────────────────────────────────────────
        let axes_len = self.axes_len;
        let axes_z_base = self.axes_z_base;
        let gpu = self
            .gpu
            .get_or_insert_with(|| {
                let resources = unsafe {
                    let unlit = UnlitProgram::new(gl);

                    let mut axes = ColoredMesh::new(gl, glow::LINES);
                    axes.upload(gl, &axes_vertices(axes_len, axes_z_base));

                    let mut grid = ColoredMesh::new(gl, glow::LINES);
                    grid.upload(
                        gl,
                        &grid_vertices(
                            DEFAULT_GRID_EXTENT,
                            DEFAULT_GRID_STEP,
                            GRID_COLOR,
                            // Drop the grid 0.001 below the axes so the
                            // axis lines sit cleanly above the grid's
                            // centre lines without Z-fighting.
                            axes_z_base - 0.001,
                        ),
                    );

                    let scene_triangles = ColoredMesh::new(gl, glow::TRIANGLES);
                    let scene_lines = ColoredMesh::new(gl, glow::LINES);

                    GpuResources {
                        unlit,
                        axes,
                        grid,
                        scene_triangles,
                        scene_triangles_ready: false,
                        scene_lines,
                        scene_lines_ready: false,
                    }
                };
                Arc::new(Mutex::new(resources))
            })
            .clone();

        // ── Drain pending scene uploads ───────────────────────────
        // gl is in scope here, so we can upload directly without
        // waiting for the paint callback. Stays None on subsequent
        // frames until the consumer calls set_scene_* again.
        if let Some(verts) = self.pending_scene_triangles.take() {
            if let Ok(mut g) = gpu.lock() {
                let has_data = !verts.is_empty();
                unsafe { g.scene_triangles.upload(gl, &verts); }
                g.scene_triangles_ready = has_data;
            }
        }
        if let Some(verts) = self.pending_scene_lines.take() {
            if let Ok(mut g) = gpu.lock() {
                let has_data = !verts.is_empty();
                unsafe { g.scene_lines.upload(gl, &verts); }
                g.scene_lines_ready = has_data;
            }
        }

        // ── Re-upload axes + grid when length or z-base changed ───
        // Triggered by set_axes_length and by the axes_z_base
        // auto-snap that fires inside set_scene_triangles when a
        // new scene's bbox has a different top. Grid sits 0.001 below
        // the axes so the axes' centre lines win the depth test.
        if self.axes_dirty {
            if let Ok(mut g) = gpu.lock() {
                unsafe {
                    g.axes.upload(gl, &axes_vertices(self.axes_len, self.axes_z_base));
                    g.grid.upload(
                        gl,
                        &grid_vertices(
                            DEFAULT_GRID_EXTENT,
                            DEFAULT_GRID_STEP,
                            GRID_COLOR,
                            self.axes_z_base - 0.001,
                        ),
                    );
                }
            }
            self.axes_dirty = false;
        }

        // ── Commit zoom-box on release ────────────────────────────
        if zoom_box_released {
            if let (Some(s), Some(e)) = (self.zoom_box_start, self.zoom_box_current) {
                // Ignore accidental clicks (< 8 px on both axes).
                if (s.x - e.x).abs() >= 8.0 || (s.y - e.y).abs() >= 8.0 {
                    if let Some(inv) = mvp_inv {
                        if let (Some(ws), Some(we)) = (
                            unproject_to_z0(&inv, rect, s),
                            unproject_to_z0(&inv, rect, e),
                        ) {
                            let min_x = ws.x.min(we.x);
                            let max_x = ws.x.max(we.x);
                            let min_y = ws.y.min(we.y);
                            let max_y = ws.y.max(we.y);
                            let w = max_x - min_x;
                            let h = max_y - min_y;
                            if w > 0.0 && h > 0.0 {
                                self.camera.target = Vector3::new(
                                    (min_x + max_x) * 0.5,
                                    (min_y + max_y) * 0.5,
                                    0.0,
                                );
                                self.camera.fit_to_bbox(w, h);
                            }
                        }
                    }
                }
            }
            self.zoom_box_start = None;
            self.zoom_box_current = None;
        }

        // ── Paint callback (the actual GL draw) ───────────────────
        let show_grid = self.state.get().show_grid;
        let show_axes = self.state.get().show_axes;
        let callback = egui_glow::CallbackFn::new(move |_info, painter| {
            let gl = painter.gl();
            let Ok(g) = gpu.lock() else { return };
            unsafe {
                gl.enable(glow::DEPTH_TEST);
                gl.depth_func(glow::LEQUAL);
                gl.depth_mask(true);
                gl.clear(glow::DEPTH_BUFFER_BIT);
                g.unlit.bind(gl, &mvp);
                // Scene triangles first — depth-write on so subsequent
                // grid/axes lines occlude correctly behind them.
                if g.scene_triangles_ready {
                    g.scene_triangles.draw(gl);
                }
                if show_grid {
                    gl.line_width(1.0);
                    g.grid.draw(gl);
                }
                if g.scene_lines_ready {
                    gl.line_width(1.5);
                    g.scene_lines.draw(gl);
                }
                if show_axes {
                    gl.line_width(2.5);
                    g.axes.draw(gl);
                }
                gl.depth_mask(false);
                gl.disable(glow::DEPTH_TEST);
            }
        });
        ui.painter().add(egui::PaintCallback {
            rect,
            callback: Arc::new(callback),
        });

        // ── XYZ axis labels (HUD, drawn after the GL pass) ────────
        if show_axes {
            self.draw_axis_labels(ui, rect, &mvp);
        }

        // ── Measure tool overlay ──────────────────────────────────
        if let (Some(ws), Some(we)) = (self.measure_start, self.measure_end) {
            self.draw_measure_overlay(ui, rect, &mvp, ws, we, measure_active);
        }
        if measure_active {
            ui.painter().text(
                egui::Pos2::new(rect.min.x + 10.0, rect.min.y + 10.0),
                egui::Align2::LEFT_TOP,
                "MEASURE  (M to exit)",
                egui::FontId::monospace(12.0),
                egui::Color32::from_rgb(255, 220, 90),
            );
        }

        // ── Right-drag zoom-box overlay ───────────────────────────
        if let (Some(s), Some(e)) = (self.zoom_box_start, self.zoom_box_current) {
            let box_rect = egui::Rect::from_two_pos(s, e).intersect(rect);
            ui.painter().rect_filled(
                box_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(255, 200, 60, 40),
            );
            ui.painter().rect_stroke(
                box_rect,
                0.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 200, 60)),
                egui::StrokeKind::Inside,
            );
        }

        ui.ctx().request_repaint();
    }

    fn draw_axis_labels(
        &self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        mvp: &nalgebra::Matrix4<f32>,
    ) {
        let l = self.axes_len;
        let label_offset_px = 14.0_f32;
        let axes = [
            (Vector3::new(l, 0.0, 0.0), "X", egui::Color32::from_rgb(255, 90, 90)),
            (Vector3::new(0.0, l, 0.0), "Y", egui::Color32::from_rgb(90, 220, 90)),
            (Vector3::new(0.0, 0.0, l), "Z", egui::Color32::from_rgb(110, 150, 255)),
        ];
        let font = egui::FontId::monospace(13.0);
        let origin_screen = project(mvp, rect, Vector3::zeros());
        for (end, text, color) in axes {
            let Some(tip) = project(mvp, rect, end) else { continue };
            let pos = match origin_screen {
                Some(origin) => {
                    let dir = tip - origin;
                    let len = (dir.x * dir.x + dir.y * dir.y).sqrt();
                    if len > 0.5 {
                        egui::Pos2::new(
                            tip.x + dir.x / len * label_offset_px,
                            tip.y + dir.y / len * label_offset_px,
                        )
                    } else {
                        tip
                    }
                }
                None => tip,
            };
            if rect.contains(pos) {
                ui.painter().text(pos, egui::Align2::CENTER_CENTER, text, font.clone(), color);
            }
        }
    }

    fn draw_measure_overlay(
        &self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        mvp: &nalgebra::Matrix4<f32>,
        ws: Vector3<f32>,
        we: Vector3<f32>,
        active: bool,
    ) {
        let (Some(ps), Some(pe)) = (project(mvp, rect, ws), project(mvp, rect, we)) else {
            return;
        };
        let color = if active {
            egui::Color32::from_rgb(255, 220, 90)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 220, 90, 180)
        };
        ui.painter().line_segment([ps, pe], egui::Stroke::new(2.0, color));
        ui.painter().circle_filled(ps, 3.5, color);
        ui.painter().circle_filled(pe, 3.5, color);
        let dist = (we - ws).norm();
        let midpoint = egui::Pos2::new((ps.x + pe.x) * 0.5, (ps.y + pe.y) * 0.5 - 10.0);
        let text = format!("{:.3}", dist);
        let galley = ui.painter().layout_no_wrap(
            text.clone(),
            egui::FontId::monospace(13.0),
            color,
        );
        let bg_rect = egui::Rect::from_center_size(midpoint, galley.size() + egui::vec2(8.0, 4.0));
        ui.painter().rect_filled(
            bg_rect,
            3.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
        );
        ui.painter().text(
            midpoint,
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::monospace(13.0),
            color,
        );
    }
}

impl Citizen for ViewerCitizen {
    fn id(&self) -> &CitizenId {
        &self.citizen_id
    }
    fn citizen_state(&self) -> &CitizenState {
        &self.citizen_state
    }
    fn citizen_state_mut(&mut self) -> &mut CitizenState {
        &mut self.citizen_state
    }
}
