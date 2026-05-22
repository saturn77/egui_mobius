//! `CanvasCitizen` — the dock-panel widget that hosts a [`Scene`].
//!
//! A `CanvasCitizen` owns a [`Registry`] (the backend-model demarcation) and
//! a [`Viewport`] (pan + zoom). Calling [`CanvasCitizen::show`] renders the
//! ribbon alongside the canvas.
//!
//! ## Ribbon
//!
//! Dockable to any of the four sides ([`RibbonSide`]). Contents:
//!
//! - Grid on/off, style (Lines / Dots), spacing, dot size, snap, units
//! - Routing default (Orthogonal / Bezier / Straight)
//! - View controls: Fit, Reset
//! - Keys menu: lists active hotkeys
//! - Dock menu: Top / Bottom / Left / Right
//!
//! ## Hotkeys (active when the canvas is hovered)
//!
//! - `G` — toggle grid on/off
//! - `X` — mirror scene about the X axis (flip vertically)
//! - `Y` — mirror scene about the Y axis (flip horizontally)
//! - `R` — rotate scene 90° clockwise
//!
//! ## Pointer interactions
//!
//! - Left-drag — pan
//! - Scroll — zoom (anchored on the hover point)
//! - Double-click — zoom-to-fit
//!
//! All edits to scene state flow through [`Registry`] — the widget never
//! touches `Scene` fields directly.

use std::path::{Path, PathBuf};

use egui::{Color32, Key, Sense, Stroke};
use egui_phosphor::regular as ico;

use crate::interact::{
    hit_test_edge, hit_test_edge_segment, hit_test_free_end, hit_test_node, hit_test_port,
    hit_test_waypoint, insert_pivot, nearest_perimeter_anchor, prepare_segment_drag,
    snap_to_grid, CanvasEvent, CanvasFsm, CanvasState, HitTarget, Selection,
};
use crate::lang::{self, CommentBlock, ParsedDocument};
use crate::model::{
    Border, CanvasBackground, Edge, EdgeEnd, EdgeEndSide, EdgeId, EdgeOverlay, Fill, GridStyle,
    GridUnits, LineStyle, Node, NodeId, NodeKind, Overlay, Port, PortId, PortKind, Routing, Scene,
    TextAnchor, TextLabel, Transform,
};
use crate::registry::Registry;
use crate::render::{
    paint_connection_preview, paint_selected_edges, paint_selected_segments, paint_selection,
    scene_bounds, viewport_fit_to, Viewport,
};
// CPU-path-only entry points — unused when the GPU pipeline is compiled.
#[cfg(not(feature = "gpu"))]
use crate::render::{background_color, paint_scene};
use crate::router::{edge_end_position, port_world_position};

/// Pointer-to-port grab tolerance, in screen pixels — a generous safety ring
/// so pressing near a port reliably grabs it instead of panning the scene.
const PORT_GRAB_PX: f32 = 14.0;
/// Pointer-to-wire grab tolerance, in screen pixels.
const EDGE_GRAB_PX: f32 = 10.0;

/// What the ribbon's File menu requested this frame.
#[derive(Clone, Copy, PartialEq, Eq)]
enum FileAction {
    Open,
    Save,
    SaveAs,
}

/// Which side of the citizen the ribbon docks to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RibbonSide {
    Top,
    Bottom,
    Left,
    Right,
}

/// The active shape-placement tool. `Select` is the default rest state —
/// normal selection / drag / pan. Any other tool turns a canvas click
/// into placing that primitive; the tool is sticky until you switch back
/// to `Select` or press `Escape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShapeTool {
    #[default]
    Select,
    Rect,
    Square,
    Circle,
    Ellipse,
    Text,
}

/// The canvas citizen widget.
pub struct CanvasCitizen {
    pub registry: Registry,
    pub viewport: Viewport,
    pub fit_padding: f32,
    pub routing_picker_applies_to_all: bool,
    pub ribbon_side: RibbonSide,
    /// Which side the shape-tool palette docks to.
    pub tool_ribbon_side: RibbonSide,
    /// The active shape-placement tool.
    pub active_tool: ShapeTool,
    /// Currently-selected nodes.
    pub selection: Selection,
    /// Canvas interaction state machine (pan / move / connect / re-route).
    fsm: CanvasFsm,
    /// Set by the Fit button; consumed in the canvas pass where the real
    /// canvas rect is available.
    pending_fit: bool,
    /// Path of the currently-open `.canvas` file, if any.
    current_path: Option<PathBuf>,
    /// Comments from the loaded document — carried so a save preserves them.
    loaded_comments: Vec<CommentBlock>,
    /// Last load/save outcome, shown in the ribbon.
    status: String,
    /// World position of the last right-click — what the context menu acts on.
    context_world: Option<(f32, f32)>,
}

/// An action chosen from the right-click context menu, applied after the
/// menu closure returns.
enum ContextAction {
    DeleteEdge(EdgeId),
    DeleteSegment(EdgeId, (f32, f32)),
    DeletePivot(EdgeId, usize),
    AddPort(NodeId, (f32, f32)),
    SetEdgeOverlay(EdgeId, EdgeOverlay),
}

fn hex_to_rgb(hex: &str) -> [u8; 3] {
    let s = hex.trim_start_matches('#');
    let byte = |i: usize| u8::from_str_radix(s.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    [byte(0), byte(2), byte(4)]
}

fn rgb_to_hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

fn line_style_label(s: crate::model::LineStyle) -> &'static str {
    use crate::model::LineStyle;
    match s {
        LineStyle::Solid => "Solid",
        LineStyle::Dashed => "Dashed",
        LineStyle::Dotted => "Dotted",
    }
}

impl CanvasCitizen {
    pub fn new(scene: Scene) -> Self {
        Self::from_registry(Registry::new(scene))
    }

    pub fn from_registry(registry: Registry) -> Self {
        Self {
            registry,
            viewport: Viewport::default(),
            fit_padding: 40.0,
            routing_picker_applies_to_all: true,
            ribbon_side: RibbonSide::Top,
            tool_ribbon_side: RibbonSide::Left,
            active_tool: ShapeTool::default(),
            selection: Selection::default(),
            fsm: CanvasFsm::new(),
            pending_fit: false,
            current_path: None,
            loaded_comments: Vec::new(),
            status: String::new(),
            context_world: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let ribbon_id = ui.id().with("grafica_ribbon");
        let panel = match self.ribbon_side {
            RibbonSide::Top => egui::Panel::top(ribbon_id),
            RibbonSide::Bottom => egui::Panel::bottom(ribbon_id),
            RibbonSide::Left => egui::Panel::left(ribbon_id),
            RibbonSide::Right => egui::Panel::right(ribbon_id),
        };
        let vertical = matches!(self.ribbon_side, RibbonSide::Left | RibbonSide::Right);
        panel.resizable(false).show_inside(ui, |ui| self.show_ribbon(ui, vertical));

        // Shape-tool palette — a second, independently-dockable ribbon.
        let tool_id = ui.id().with("grafica_tool_ribbon");
        let tool_panel = match self.tool_ribbon_side {
            RibbonSide::Top => egui::Panel::top(tool_id),
            RibbonSide::Bottom => egui::Panel::bottom(tool_id),
            RibbonSide::Left => egui::Panel::left(tool_id),
            RibbonSide::Right => egui::Panel::right(tool_id),
        };
        let tool_vertical = matches!(self.tool_ribbon_side, RibbonSide::Left | RibbonSide::Right);
        tool_panel
            .resizable(false)
            .show_inside(ui, |ui| self.show_tool_ribbon(ui, tool_vertical));

        egui::CentralPanel::default().show_inside(ui, |ui| self.show_canvas(ui));
    }

    /// The shape-tool palette: a select cursor plus one button per
    /// placeable primitive. Sticky — the chosen tool stays active for
    /// repeated placement until `Select` is re-chosen or `Escape` pressed.
    fn show_tool_ribbon(&mut self, ui: &mut egui::Ui, vertical: bool) {
        let lay = |ui: &mut egui::Ui, body: &mut dyn FnMut(&mut egui::Ui)| {
            if vertical {
                ui.vertical(body);
            } else {
                ui.horizontal_wrapped(body);
            }
        };
        let mut tool = self.active_tool;
        let mut dock_to: Option<RibbonSide> = None;

        lay(ui, &mut |ui| {
            ui.label(egui::RichText::new(format!("{} Tools", ico::CURSOR)).strong());
            sep(ui, vertical);

            let mut btn = |ui: &mut egui::Ui, t: ShapeTool, icon: &str, tip: &str| {
                if ui
                    .selectable_label(tool == t, icon)
                    .on_hover_text(tip)
                    .clicked()
                {
                    tool = t;
                }
            };
            btn(ui, ShapeTool::Select, ico::CURSOR, "Select / move");
            btn(ui, ShapeTool::Rect, ico::RECTANGLE, "Rectangle");
            btn(ui, ShapeTool::Square, ico::SQUARE, "Square");
            btn(ui, ShapeTool::Circle, ico::CIRCLE, "Circle");
            // Phosphor has no oval glyph; the circle glyph stands in,
            // disambiguated by the tooltip.
            btn(ui, ShapeTool::Ellipse, ico::CIRCLE, "Ellipse");
            btn(ui, ShapeTool::Text, ico::TEXT_T, "Text");

            sep(ui, vertical);
            ui.menu_button(format!("{} Dock", ico::RECTANGLE_DASHED), |ui| {
                if ui.button("Top").clicked() {
                    dock_to = Some(RibbonSide::Top);
                    ui.close();
                }
                if ui.button("Bottom").clicked() {
                    dock_to = Some(RibbonSide::Bottom);
                    ui.close();
                }
                if ui.button("Left").clicked() {
                    dock_to = Some(RibbonSide::Left);
                    ui.close();
                }
                if ui.button("Right").clicked() {
                    dock_to = Some(RibbonSide::Right);
                    ui.close();
                }
            });
        });

        self.active_tool = tool;
        if let Some(side) = dock_to {
            self.tool_ribbon_side = side;
        }
    }

    fn show_ribbon(&mut self, ui: &mut egui::Ui, vertical: bool) {
        let lay = |ui: &mut egui::Ui, body: &mut dyn FnMut(&mut egui::Ui)| {
            if vertical {
                ui.vertical(body);
            } else {
                // Wrap onto extra rows when the controls don't fit one line.
                ui.horizontal_wrapped(body);
            }
        };

        let mut settings = self.registry.with_scene(|s| s.settings.clone());
        let mut settings_changed = false;
        let mut routing_changed_to: Option<Routing> = None;
        let mut dock_to: Option<RibbonSide> = None;
        let mut reset_clicked = false;
        let mut file_action: Option<FileAction> = None;

        lay(ui, &mut |ui| {
            ui.label(egui::RichText::new(format!("{} Grafica", ico::PALETTE)).strong());
            sep(ui, vertical);

            ui.menu_button(format!("{} File", ico::FOLDER_OPEN), |ui| {
                if ui.button(format!("{} Open…", ico::FOLDER_OPEN)).clicked() {
                    file_action = Some(FileAction::Open);
                    ui.close();
                }
                if ui.button(format!("{} Save", ico::FLOPPY_DISK)).clicked() {
                    file_action = Some(FileAction::Save);
                    ui.close();
                }
                if ui.button(format!("{} Save As…", ico::FLOPPY_DISK)).clicked() {
                    file_action = Some(FileAction::SaveAs);
                    ui.close();
                }
            });
            sep(ui, vertical);

            if ui
                .checkbox(&mut settings.show_grid, format!("{} Grid", ico::GRID_FOUR))
                .changed()
            {
                settings_changed = true;
            }

            let before_style = settings.grid_style;
            egui::ComboBox::from_id_salt("grafica_grid_style")
                .selected_text(grid_style_label(settings.grid_style))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.grid_style, GridStyle::Lines, "Lines");
                    ui.selectable_value(&mut settings.grid_style, GridStyle::Dots, "Dots");
                });
            if before_style != settings.grid_style {
                settings_changed = true;
            }

            ui.label(format!("{} spacing", ico::RULER));
            if ui
                .add(
                    egui::Slider::new(&mut settings.grid_spacing, 1.0..=100.0)
                        .suffix(settings.grid_units.suffix()),
                )
                .changed()
            {
                settings_changed = true;
            }

            if settings.grid_style == GridStyle::Dots {
                ui.label(format!("{} dot", ico::DOTS_NINE));
                if ui
                    .add(
                        egui::Slider::new(&mut settings.dot_size, 0.5..=8.0)
                            .suffix(settings.grid_units.suffix()),
                    )
                    .changed()
                {
                    settings_changed = true;
                }
            }

            if ui
                .checkbox(&mut settings.snap_to_grid, format!("{} Snap", ico::MAGNET))
                .changed()
            {
                settings_changed = true;
            }

            ui.label("units");
            let before_units = settings.grid_units;
            egui::ComboBox::from_id_salt("grafica_grid_units")
                .selected_text(settings.grid_units.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.grid_units, GridUnits::Pixels, "Pixels");
                    ui.selectable_value(&mut settings.grid_units, GridUnits::Mils, "Mils");
                    ui.selectable_value(&mut settings.grid_units, GridUnits::Millimeters, "Millimeters");
                    ui.selectable_value(&mut settings.grid_units, GridUnits::Inches, "Inches");
                });
            if before_units != settings.grid_units {
                settings_changed = true;
            }

            ui.label(format!("{} Bg", ico::PALETTE));
            let before_bg = settings.background;
            egui::ComboBox::from_id_salt("grafica_background")
                .selected_text(settings.background.label())
                .show_ui(ui, |ui| {
                    use CanvasBackground::*;
                    for bg in [Light, Slate, Charcoal, Dark] {
                        ui.selectable_value(&mut settings.background, bg, bg.label());
                    }
                });
            if before_bg != settings.background {
                settings_changed = true;
            }

            sep(ui, vertical);

            ui.label(format!("{} Routing", ico::LINE_SEGMENTS));
            let before_routing = settings.default_routing.clone();
            egui::ComboBox::from_id_salt("grafica_routing_picker")
                .selected_text(routing_label(&settings.default_routing))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.default_routing, Routing::Orthogonal, "Orthogonal");
                    ui.selectable_value(&mut settings.default_routing, Routing::Bezier, "Bezier");
                    ui.selectable_value(&mut settings.default_routing, Routing::Straight, "Straight");
                });
            if before_routing != settings.default_routing {
                settings_changed = true;
                routing_changed_to = Some(settings.default_routing.clone());
            }

            sep(ui, vertical);

            if ui.button(format!("{} Fit", ico::FRAME_CORNERS)).clicked() {
                self.pending_fit = true;
            }
            if ui.button(format!("{} Reset", ico::ARROW_COUNTER_CLOCKWISE)).clicked() {
                reset_clicked = true;
            }

            sep(ui, vertical);

            ui.menu_button(format!("{} Keys", ico::KEYBOARD), |ui| {
                ui.set_min_width(220.0);
                for (key, desc) in HOTKEY_TABLE {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(" {}  ", key))
                                .monospace()
                                .background_color(Color32::from_gray(40))
                                .color(Color32::from_gray(220)),
                        );
                        ui.label(*desc);
                    });
                }
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Active when canvas is hovered")
                        .small()
                        .color(Color32::from_gray(140)),
                );
            });

            ui.menu_button(format!("{} Dock", ico::RECTANGLE_DASHED), |ui| {
                if ui.button("Top").clicked() {
                    dock_to = Some(RibbonSide::Top);
                    ui.close();
                }
                if ui.button("Bottom").clicked() {
                    dock_to = Some(RibbonSide::Bottom);
                    ui.close();
                }
                if ui.button("Left").clicked() {
                    dock_to = Some(RibbonSide::Left);
                    ui.close();
                }
                if ui.button("Right").clicked() {
                    dock_to = Some(RibbonSide::Right);
                    ui.close();
                }
            });

            if !self.status.is_empty() {
                sep(ui, vertical);
                ui.label(
                    egui::RichText::new(&self.status)
                        .small()
                        .color(Color32::from_gray(120)),
                );
            }
        });

        if settings_changed {
            self.registry.update_settings(settings);
        }
        if let Some(new_routing) = routing_changed_to
            && self.routing_picker_applies_to_all
        {
            self.apply_routing_to_all(new_routing);
        }
        if reset_clicked {
            self.viewport = Viewport::default();
        }
        if let Some(side) = dock_to {
            self.ribbon_side = side;
        }
        if let Some(action) = file_action {
            self.handle_file_action(action);
        }
    }

    // ── File I/O ─────────────────────────────────────────────────────────

    fn handle_file_action(&mut self, action: FileAction) {
        match action {
            FileAction::Open => self.open_file(),
            FileAction::Save => {
                if self.current_path.is_some() {
                    self.save_to_current();
                } else {
                    self.save_as();
                }
            }
            FileAction::SaveAs => self.save_as(),
        }
    }

    fn open_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("canvas DSL", &["canvas"])
            .pick_file()
        else {
            return;
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => match lang::parse_document(&text) {
                Ok(doc) => {
                    self.registry.set_scene(doc.scene);
                    self.loaded_comments = doc.comments;
                    self.selection.clear();
                    self.status = format!("Opened {}", file_name(&path));
                    self.current_path = Some(path);
                }
                Err(e) => self.status = format!("Parse error — {e}"),
            },
            Err(e) => self.status = format!("Read error — {e}"),
        }
    }

    fn save_as(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("canvas DSL", &["canvas"])
            .set_file_name("scene.canvas")
            .save_file()
        else {
            return;
        };
        self.current_path = Some(path);
        self.save_to_current();
    }

    fn save_to_current(&mut self) {
        let Some(path) = self.current_path.clone() else {
            return;
        };
        let doc = ParsedDocument {
            scene: self.registry.scene(),
            comments: self.loaded_comments.clone(),
        };
        match std::fs::write(&path, lang::pretty_document(&doc)) {
            Ok(()) => self.status = format!("Saved {}", file_name(&path)),
            Err(e) => self.status = format!("Write error — {e}"),
        }
    }

    fn show_canvas(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = response.rect;

        if self.pending_fit {
            self.fit_to_rect(rect);
            self.pending_fit = false;
        }

        let shift = ui.input(|i| i.modifiers.shift);

        // ── Press: classify what was hit and drive the FSM ──
        //
        // Gated to the PRIMARY button — `Sense::click_and_drag()` reports
        // drags from any button, so without this the right button also
        // panned and right/left events interleaved into the FSM.
        //
        // Hit-test at the true press origin, not `interact_pointer_pos` —
        // egui only reports a drag once the pointer has moved a few pixels,
        // and testing that drifted point misses thin wires and small ports.
        if self.active_tool == ShapeTool::Select
            && response.drag_started_by(egui::PointerButton::Primary)
            && let Some(screen) = ui.input(|i| i.pointer.press_origin())
        {
            let world = self.viewport.screen_to_world(screen);
            let port_radius = PORT_GRAB_PX / self.viewport.zoom;
            let edge_thresh = EDGE_GRAB_PX / self.viewport.zoom;

            // Priority: pivot vertex > free end > port > node body > wire > empty.
            // Free ends are draggable handles for reattachment, so they
            // win over a port that happens to sit at the same point.
            let waypoint_hit =
                self.registry.with_scene(|s| hit_test_waypoint(s, world, port_radius));
            let free_end_hit =
                self.registry.with_scene(|s| hit_test_free_end(s, world, port_radius));
            let port_hit = self.registry.with_scene(|s| hit_test_port(s, world, port_radius));
            let node_hit = self.registry.with_scene(|s| hit_test_node(s, world));
            let edge_hit = self.registry.with_scene(|s| hit_test_edge(s, world, edge_thresh));
            let target = if waypoint_hit.is_some() {
                HitTarget::Waypoint
            } else if free_end_hit.is_some() {
                HitTarget::FreeEnd
            } else if port_hit.is_some() {
                HitTarget::Port
            } else if node_hit.is_some() {
                HitTarget::NodeBody
            } else if edge_hit.is_some() {
                HitTarget::WireSegment
            } else {
                HitTarget::Empty
            };

            self.fsm.dispatch(CanvasEvent::Press, target);
            self.fsm.grab_world = world;
            self.fsm.cursor_world = world;

            match self.fsm.state {
                CanvasState::MovingNodes => {
                    let id = node_hit.expect("NodeBody target implies a node hit");
                    if !self.selection.contains(&id) {
                        if shift {
                            self.selection.toggle(id);
                        } else {
                            self.selection.select_only(id);
                        }
                    }
                    self.fsm.node_origins = self.registry.with_scene(|s| {
                        self.selection
                            .nodes
                            .iter()
                            .filter_map(|nid| {
                                s.nodes
                                    .iter()
                                    .find(|n| &n.id == nid)
                                    .map(|n| (nid.clone(), n.transform.position))
                            })
                            .collect()
                    });
                }
                CanvasState::Connecting => {
                    self.fsm.connect_from = port_hit;
                    self.fsm.connect_latched = false;
                    // Record the port's anchor so a connection-draw gesture
                    // can restore it (a reposition keeps the dragged anchor).
                    let origin = self.fsm.connect_from.as_ref().and_then(|(nid, pid)| {
                        self.registry.with_scene(|s| {
                            s.nodes
                                .iter()
                                .find(|n| &n.id == nid)
                                .and_then(|n| n.ports.iter().find(|p| &p.id == pid))
                                .map(|p| p.anchor)
                        })
                    });
                    self.fsm.connect_origin_anchor = origin;
                }
                CanvasState::DraggingWaypoint => {
                    let (eid, idx) = waypoint_hit.expect("Waypoint target implies a hit");
                    self.selection.select_only_edge(eid.clone());
                    self.fsm.drag_edge = Some(eid);
                    self.fsm.drag_waypoint = idx;
                }
                CanvasState::DraggingFreeEnd => {
                    let (eid, side) = free_end_hit.expect("FreeEnd target implies a hit");
                    self.selection.select_only_edge(eid.clone());
                    // Snapshot the Free position — kept as the new
                    // waypoint when the wire is extended on release.
                    let origin = self.registry.with_scene(|s| {
                        s.edges.iter().find(|e| e.id == eid).and_then(|e| {
                            let end = match side {
                                EdgeEndSide::From => &e.from,
                                EdgeEndSide::To => &e.to,
                            };
                            if let EdgeEnd::Free(x, y) = end {
                                Some((*x, *y))
                            } else {
                                None
                            }
                        })
                    });
                    self.fsm.drag_free_origin = origin;
                    self.fsm.drag_edge = Some(eid);
                    self.fsm.drag_free_side = Some(side);
                }
                CanvasState::DraggingSegment => {
                    let eid = edge_hit.expect("WireSegment target implies a wire hit");
                    if shift {
                        self.selection.toggle_edge(eid.clone());
                    } else {
                        self.selection.select_only_edge(eid.clone());
                    }
                    // Materialise the wire's path and prepare the segment drag
                    // — a stub is inserted if the grabbed segment touched a
                    // pinned endpoint, so it has two movable interior ends.
                    let polyline = self.registry.with_scene(|s| {
                        s.edges.iter().find(|e| e.id == eid).and_then(|e| {
                            let f = edge_end_position(s, &e.from)?;
                            let t = edge_end_position(s, &e.to)?;
                            Some(match &e.routing {
                                Routing::Manual { .. } => crate::router::edge_polyline(s, e)?,
                                _ => crate::router::route(f, t, &Routing::Orthogonal),
                            })
                        })
                    });
                    if let Some(poly) = polyline
                        && let Some(sd) = prepare_segment_drag(&poly, world)
                    {
                        self.fsm.drag_edge = Some(eid);
                        self.fsm.drag_segment = sd.segment;
                        self.fsm.drag_axis_horizontal = sd.horizontal;
                        self.fsm.drag_origin_pts = sd.points;
                    }
                }
                _ => {}
            }
        }

        // ── Drag: act on the FSM's current state (primary button only) ──
        if response.dragged_by(egui::PointerButton::Primary) {
            match self.fsm.state {
                CanvasState::Marquee => {
                    if let Some(screen) = response.interact_pointer_pos() {
                        self.fsm.cursor_world = self.viewport.screen_to_world(screen);
                    }
                }
                CanvasState::MovingNodes => {
                    if let Some(screen) = response.interact_pointer_pos() {
                        let now = self.viewport.screen_to_world(screen);
                        let (dx, dy) = (now.0 - self.fsm.grab_world.0, now.1 - self.fsm.grab_world.1);
                        let (snap, spacing) = self
                            .registry
                            .with_scene(|s| (s.settings.snap_to_grid, s.settings.grid_spacing));
                        let moves: Vec<_> = self
                            .fsm
                            .node_origins
                            .iter()
                            .map(|(id, origin)| {
                                let mut pos = (origin.0 + dx, origin.1 + dy);
                                if snap {
                                    pos = snap_to_grid(pos, spacing);
                                }
                                (id.clone(), pos)
                            })
                            .collect();
                        self.registry.move_nodes(&moves);
                    }
                }
                CanvasState::Connecting => {
                    // Spatial gesture: while the cursor stays over the source
                    // node, slide the port along its perimeter; once it
                    // leaves, latch into drawing a connection instead.
                    if let Some(screen) = response.interact_pointer_pos() {
                        let now = self.viewport.screen_to_world(screen);
                        self.fsm.cursor_world = now;
                        if !self.fsm.connect_latched
                            && let Some((nid, pid)) = self.fsm.connect_from.clone()
                        {
                            let node_box = self.registry.with_scene(|s| {
                                s.nodes
                                    .iter()
                                    .find(|n| n.id == nid)
                                    .map(|n| (n.transform.position, n.transform.size))
                            });
                            if let Some((pos, size)) = node_box {
                                let m = 24.0 / self.viewport.zoom;
                                let inside = now.0 >= pos.0 - m
                                    && now.0 <= pos.0 + size.0 + m
                                    && now.1 >= pos.1 - m
                                    && now.1 <= pos.1 + size.1 + m;
                                if inside {
                                    let anchor = nearest_perimeter_anchor(pos, size, now);
                                    self.registry.set_port_anchor(&nid, &pid, anchor);
                                } else {
                                    self.fsm.connect_latched = true;
                                    if let Some(orig) = self.fsm.connect_origin_anchor {
                                        self.registry.set_port_anchor(&nid, &pid, orig);
                                    }
                                }
                            }
                        }
                    }
                }
                CanvasState::DraggingSegment => {
                    // Move the grabbed segment perpendicular to itself:
                    // a horizontal segment in Y, a vertical segment in X.
                    if let Some(eid) = self.fsm.drag_edge.clone()
                        && let Some(screen) = response.interact_pointer_pos()
                        && !self.fsm.drag_origin_pts.is_empty()
                    {
                        let cursor = self.viewport.screen_to_world(screen);
                        let (dx, dy) =
                            (cursor.0 - self.fsm.grab_world.0, cursor.1 - self.fsm.grab_world.1);
                        let mut pts = self.fsm.drag_origin_pts.clone();
                        let k = self.fsm.drag_segment;
                        if k + 1 < pts.len() {
                            if self.fsm.drag_axis_horizontal {
                                pts[k].1 += dy;
                                pts[k + 1].1 += dy;
                            } else {
                                pts[k].0 += dx;
                                pts[k + 1].0 += dx;
                            }
                        }
                        let waypoints = pts[1..pts.len() - 1].to_vec();
                        self.registry
                            .update_edge_routing(&eid, Routing::Manual { waypoints });
                    }
                }
                CanvasState::DraggingWaypoint => {
                    // Move the pivot vertex freely (both axes).
                    if let Some(eid) = self.fsm.drag_edge.clone()
                        && let Some(screen) = response.interact_pointer_pos()
                    {
                        let cursor = self.viewport.screen_to_world(screen);
                        let idx = self.fsm.drag_waypoint;
                        let updated = self.registry.with_scene(|s| {
                            s.edges.iter().find(|e| e.id == eid).and_then(|e| {
                                if let Routing::Manual { waypoints } = &e.routing {
                                    let mut wps = waypoints.clone();
                                    if idx < wps.len() {
                                        wps[idx] = cursor;
                                        return Some(wps);
                                    }
                                }
                                None
                            })
                        });
                        if let Some(waypoints) = updated {
                            self.registry
                                .update_edge_routing(&eid, Routing::Manual { waypoints });
                        }
                    }
                }
                CanvasState::DraggingFreeEnd => {
                    // Track the cursor for the rubber-band preview only.
                    // The wire isn't mutated until release so the Free
                    // anchor stays drawn at its original position.
                    if let Some(screen) = response.interact_pointer_pos() {
                        self.fsm.cursor_world = self.viewport.screen_to_world(screen);
                    }
                }
                CanvasState::Idle => {}
            }
        }

        // ── Middle-button drag pans the viewport (independent of the FSM). ──
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = response.drag_delta();
            self.viewport.origin.x += delta.x;
            self.viewport.origin.y += delta.y;
        }

        // ── Release: finalise the gesture, return the FSM to Idle ──
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            // Only a latched (left-the-node) gesture creates an edge; an
            // unlatched one was a port reposition, already committed live.
            if self.fsm.state == CanvasState::Connecting
                && self.fsm.connect_latched
                && let Some(from) = self.fsm.connect_from.clone()
            {
                self.finish_connection(from, self.fsm.cursor_world);
            }
            // A marquee selects every node/edge it caught.
            if self.fsm.state == CanvasState::Marquee {
                let (nodes, edges) = self.registry.with_scene(|s| {
                    crate::interact::marquee_pick(s, self.fsm.grab_world, self.fsm.cursor_world)
                });
                self.selection.nodes = nodes;
                self.selection.edges = edges;
            }
            // Extend the wire from its Free end. The old free position
            // becomes a waypoint; the new end is the released point —
            // a port if released over one, else a fresh free point.
            if self.fsm.state == CanvasState::DraggingFreeEnd
                && let Some(eid) = self.fsm.drag_edge.clone()
                && let Some(side) = self.fsm.drag_free_side
            {
                let port_radius = PORT_GRAB_PX / self.viewport.zoom;
                let snap = self
                    .registry
                    .with_scene(|s| hit_test_port(s, self.fsm.cursor_world, port_radius));
                let new_end = match snap {
                    Some((nid, pid)) => EdgeEnd::Port(nid, pid),
                    None => EdgeEnd::Free(self.fsm.cursor_world.0, self.fsm.cursor_world.1),
                };
                self.registry.extend_free_end(&eid, side, new_end);
            }
            self.fsm.dispatch(CanvasEvent::Release, HitTarget::Empty);
        }

        // Click (press + release, no movement): select node, else wire, else clear.
        if response.clicked()
            && let Some(screen) = response.interact_pointer_pos()
        {
            let world = self.viewport.screen_to_world(screen);
            if self.active_tool != ShapeTool::Select {
                // A tool is armed: the click places a primitive instead
                // of selecting.
                self.place_shape(world);
            } else {
                let edge_thresh = EDGE_GRAB_PX / self.viewport.zoom;
                if let Some(id) = self.registry.with_scene(|s| hit_test_node(s, world)) {
                    if shift {
                        self.selection.toggle(id);
                    } else {
                        self.selection.select_only(id);
                    }
                } else if let Some((eid, seg)) =
                    self.registry.with_scene(|s| hit_test_edge_segment(s, world, edge_thresh))
                {
                    // A wire click selects the run under the pointer, not
                    // the whole wire — only that segment highlights.
                    if shift {
                        self.selection.toggle_segment(eid, seg);
                    } else {
                        self.selection.select_only_segment(eid, seg);
                    }
                } else if !shift {
                    self.selection.clear();
                }
            }
        }

        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.01
                && let Some(hover) = response.hover_pos()
            {
                let world_before = self.viewport.screen_to_world(hover);
                let factor = (scroll * 0.005).exp();
                self.viewport.zoom = (self.viewport.zoom * factor).clamp(0.1, 10.0);
                let world_after = self.viewport.screen_to_world(hover);
                self.viewport.origin.x += (world_after.0 - world_before.0) * self.viewport.zoom;
                self.viewport.origin.y += (world_after.1 - world_before.1) * self.viewport.zoom;
            }

            // Hotkeys — gated on canvas hover so text widgets elsewhere
            // aren't silently swallowed.
            let (g, x, y, r) = ui.input(|i| {
                (
                    i.key_pressed(Key::G),
                    i.key_pressed(Key::X),
                    i.key_pressed(Key::Y),
                    i.key_pressed(Key::R),
                )
            });
            if g {
                self.registry.toggle_grid();
            }
            if x {
                self.registry.mirror_scene_about_x();
            }
            if y {
                self.registry.mirror_scene_about_y();
            }
            if r {
                self.registry.rotate_scene_90_cw();
            }

            if ui.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
                self.delete_selection();
            }
            // Escape disarms the active shape tool back to Select.
            if ui.input(|i| i.key_pressed(Key::Escape)) {
                self.active_tool = ShapeTool::Select;
            }
        }

        // Double-click: on a pivot vertex it deletes that pivot; on a wire it
        // inserts a pivot; on empty space it zooms to fit.
        if response.double_clicked()
            && let Some(screen) = response.interact_pointer_pos()
        {
            let world = self.viewport.screen_to_world(screen);
            let port_radius = PORT_GRAB_PX / self.viewport.zoom;
            let edge_thresh = EDGE_GRAB_PX / self.viewport.zoom;

            if let Some((eid, idx)) =
                self.registry.with_scene(|s| hit_test_waypoint(s, world, port_radius))
            {
                // Delete the pivot — the two segments on either side merge.
                let routing = self.registry.with_scene(|s| {
                    s.edges.iter().find(|e| e.id == eid).map(|e| match &e.routing {
                        Routing::Manual { waypoints } => {
                            let mut wps = waypoints.clone();
                            if idx < wps.len() {
                                wps.remove(idx);
                            }
                            if wps.is_empty() {
                                Routing::Orthogonal
                            } else {
                                Routing::Manual { waypoints: wps }
                            }
                        }
                        other => other.clone(),
                    })
                });
                if let Some(routing) = routing {
                    self.registry.update_edge_routing(&eid, routing);
                }
            } else if let Some(eid) =
                self.registry.with_scene(|s| hit_test_edge(s, world, edge_thresh))
            {
                let poly = self
                    .registry
                    .with_scene(|s| s.edges.iter().find(|e| e.id == eid).cloned())
                    .and_then(|e| self.registry.with_scene(|s| crate::router::edge_polyline(s, &e)));
                if let Some(poly) = poly {
                    self.registry
                        .update_edge_routing(&eid, Routing::Manual { waypoints: insert_pivot(&poly, world) });
                }
            } else {
                self.fit_to_rect(rect);
            }
        }

        // ── Right-click context menu ──
        if response.secondary_clicked()
            && let Some(p) = response.interact_pointer_pos()
        {
            self.context_world = Some(self.viewport.screen_to_world(p));
        }
        let ctx = self.context_world;
        let port_radius = PORT_GRAB_PX / self.viewport.zoom;
        let edge_thresh = EDGE_GRAB_PX / self.viewport.zoom;
        let (hit_wp, hit_edge, hit_node) = match ctx {
            Some(w) => self.registry.with_scene(|s| {
                (
                    hit_test_waypoint(s, w, port_radius),
                    hit_test_edge(s, w, edge_thresh),
                    hit_test_node(s, w),
                )
            }),
            None => (None, None, None),
        };
        let edge_overlay = hit_edge.as_ref().and_then(|eid| {
            self.registry
                .with_scene(|s| s.edges.iter().find(|e| &e.id == eid).map(|e| e.overlay.clone()))
        });
        let mut action: Option<ContextAction> = None;
        response.context_menu(|ui| {
            if let Some((eid, idx)) = &hit_wp {
                if ui.button("Delete pivot").clicked() {
                    action = Some(ContextAction::DeletePivot(eid.clone(), *idx));
                    ui.close();
                }
            } else if let Some(eid) = &hit_edge {
                if ui.button("Delete segment").clicked() {
                    action = Some(ContextAction::DeleteSegment(eid.clone(), ctx.unwrap_or_default()));
                    ui.close();
                }
                if ui.button("Delete wire").clicked() {
                    action = Some(ContextAction::DeleteEdge(eid.clone()));
                    ui.close();
                }
                if let Some(overlay) = &edge_overlay {
                    ui.menu_button("Wire style", |ui| {
                        let mut next = overlay.clone();
                        let mut rgb = hex_to_rgb(&next.color);
                        ui.horizontal(|ui| {
                            ui.label("Color");
                            ui.color_edit_button_srgb(&mut rgb);
                        });
                        next.color = rgb_to_hex(rgb);
                        ui.add(egui::Slider::new(&mut next.width, 0.5..=6.0).text("Width"));
                        ui.separator();
                        for style in [LineStyle::Solid, LineStyle::Dashed, LineStyle::Dotted] {
                            if ui
                                .selectable_label(next.line_style == style, line_style_label(style))
                                .clicked()
                            {
                                next.line_style = style;
                            }
                        }
                        if next != *overlay {
                            action = Some(ContextAction::SetEdgeOverlay(eid.clone(), next));
                        }
                    });
                }
            } else if let Some(nid) = &hit_node {
                if ui.button("Add connection").clicked() {
                    action = Some(ContextAction::AddPort(nid.clone(), ctx.unwrap_or_default()));
                    ui.close();
                }
            } else {
                ui.label(egui::RichText::new("Nothing here").weak());
            }
        });
        if let Some(action) = action {
            self.apply_context_action(action);
        }

        // Background, grid, and — on the GPU path — node bodies are drawn
        // by the wgpu pipeline; everything else stays on the egui painter.
        // The CPU path fills and strokes the whole scene on the painter.
        #[cfg(feature = "gpu")]
        let generation = self.registry.generation();
        #[cfg(feature = "gpu")]
        self.registry.with_scene(|scene| {
            // GPU: background, grid, node bodies, edge segments.
            crate::gpu::paint_canvas(&painter, rect, &self.viewport, scene, generation);
            // Painter: everything the GPU path leaves out.
            crate::render::paint_arrowheads(&painter, scene, &self.viewport);
            crate::render::paint_node_labels(&painter, scene, &self.viewport);
            crate::render::paint_ports(&painter, scene, &self.viewport);
            crate::render::paint_waypoints(&painter, scene, &self.viewport);
            crate::render::paint_free_ends(&painter, scene, &self.viewport);
        });
        #[cfg(not(feature = "gpu"))]
        {
            let settings = self.registry.with_scene(|s| s.settings.clone());
            painter.rect_filled(rect, 0.0, background_color(settings.background));
            if settings.show_grid {
                crate::render::paint_grid(&painter, &self.viewport, &settings, rect);
            }
            self.registry.with_scene(|scene| paint_scene(&painter, scene, &self.viewport));
        }

        // Selection highlights — painter-side on both paths.
        self.registry.with_scene(|scene| {
            paint_selected_edges(&painter, scene, &self.selection.edges, &self.viewport);
            paint_selected_segments(&painter, scene, &self.selection.segments, &self.viewport);
            paint_selection(&painter, scene, &self.selection.nodes, &self.viewport);
        });

        // Rubber-band preview while a connection is being drawn.
        if self.fsm.state == CanvasState::Connecting
            && self.fsm.connect_latched
            && let Some(from) = &self.fsm.connect_from
            && let Some(from_world) =
                self.registry.with_scene(|s| port_world_position(s, &from.0, &from.1))
        {
            paint_connection_preview(&painter, from_world, self.fsm.cursor_world, &self.viewport);
        }

        // Rubber-band preview while extending a wire from a dangling end.
        if self.fsm.state == CanvasState::DraggingFreeEnd
            && let Some(origin) = self.fsm.drag_free_origin
        {
            paint_connection_preview(&painter, origin, self.fsm.cursor_world, &self.viewport);
        }

        // Rubber-band rectangle while marquee-selecting.
        if self.fsm.state == CanvasState::Marquee {
            let a = self.viewport.world_to_screen(self.fsm.grab_world);
            let b = self.viewport.world_to_screen(self.fsm.cursor_world);
            let marquee = egui::Rect::from_two_pos(a, b);
            let accent = Color32::from_rgb(0x25, 0x63, 0xEB);
            painter.rect_filled(marquee, 0.0, Color32::from_rgba_unmultiplied(0x25, 0x63, 0xEB, 28));
            painter.rect_stroke(
                marquee,
                0.0,
                Stroke::new(1.0, accent),
                egui::StrokeKind::Inside,
            );
        }
    }

    /// Finish a connection drag: if the pointer was released near a
    /// port — or onto an existing wire's dangling [`EdgeEnd::Free`]
    /// end — add an edge from the source port to that drop target.
    ///
    /// A free-end drop matches the existing free's exact coordinate,
    /// so the new wire visually meets the original at the cut point.
    fn finish_connection(&mut self, from: (NodeId, PortId), cursor_world: (f32, f32)) {
        let port_radius = PORT_GRAB_PX / self.viewport.zoom;
        let to_end = self.registry.with_scene(|s| -> Option<EdgeEnd> {
            if let Some((nid, pid)) = hit_test_port(s, cursor_world, port_radius) {
                if (nid.clone(), pid.clone()) != from {
                    return Some(EdgeEnd::Port(nid, pid));
                }
            }
            if let Some((eid, side)) = hit_test_free_end(s, cursor_world, port_radius) {
                let edge = s.edges.iter().find(|e| e.id == eid)?;
                let target = match side {
                    EdgeEndSide::From => &edge.from,
                    EdgeEndSide::To => &edge.to,
                };
                if let EdgeEnd::Free(x, y) = target {
                    return Some(EdgeEnd::Free(*x, *y));
                }
            }
            None
        });
        let Some(to_end) = to_end else {
            return;
        };
        let (id, routing) = self
            .registry
            .with_scene(|s| (fresh_edge_id(s), s.settings.default_routing.clone()));
        self.registry.add_edge(Edge {
            id,
            from: EdgeEnd::Port(from.0, from.1),
            to: to_end,
            routing,
            overlay: EdgeOverlay::default(),
        });
    }

    fn fit_to_rect(&mut self, screen_rect: egui::Rect) {
        let bounds = self.registry.with_scene(scene_bounds);
        if let Some(b) = bounds {
            self.viewport = viewport_fit_to(b, screen_rect, self.fit_padding);
        }
    }

    /// Remove every selected edge and node through the registry. Removing a
    /// node also drops its attached edges (registry handles the cascade).
    fn delete_selection(&mut self) {
        // Whole-edge and node selections delete outright.
        for id in self.selection.edges.clone() {
            self.registry.remove_edge(&id);
        }
        for id in self.selection.nodes.clone() {
            self.registry.remove_node(&id);
        }
        // Segment selections truncate their wire — surviving runs stay,
        // dangling at the cuts. Group by edge so multiple segments on
        // one wire delete in a single pass.
        let mut handled: Vec<EdgeId> = Vec::new();
        for (eid, _) in self.selection.segments.clone() {
            if handled.contains(&eid) {
                continue;
            }
            let segs: Vec<usize> = self
                .selection
                .segments
                .iter()
                .filter(|(e, _)| e == &eid)
                .map(|(_, s)| *s)
                .collect();
            self.truncate_edge_segments(&eid, &segs);
            handled.push(eid);
        }
        self.selection.clear();
    }

    /// Remove `deleted` segments from edge `eid`, replacing it with the
    /// surviving runs. A run that no longer reaches the original port
    /// keeps the geometry but dangles as a [`EdgeEnd::Free`] end.
    fn truncate_edge_segments(&mut self, eid: &EdgeId, deleted: &[usize]) {
        let prepared = self.registry.with_scene(|s| {
            let edge = s.edges.iter().find(|e| &e.id == eid)?;
            let poly = crate::router::edge_polyline(s, edge)?;
            Some((
                edge.overlay.clone(),
                crate::interact::split_edge_on_deletes(edge, &poly, deleted),
            ))
        });
        let Some((overlay, survivors)) = prepared else {
            return;
        };
        self.registry.remove_edge(eid);
        for (i, surv) in survivors.into_iter().enumerate() {
            let id = if i == 0 {
                eid.clone()
            } else {
                self.registry.with_scene(fresh_edge_id)
            };
            self.registry.add_edge(Edge {
                id,
                from: surv.from,
                to: surv.to,
                routing: surv.routing,
                overlay: overlay.clone(),
            });
        }
    }

    /// Apply a right-click context-menu action.
    fn apply_context_action(&mut self, action: ContextAction) {
        match action {
            ContextAction::DeleteEdge(eid) => {
                self.registry.remove_edge(&eid);
                self.selection.edges.retain(|e| e != &eid);
            }
            ContextAction::DeletePivot(eid, idx) => {
                let routing = self.registry.with_scene(|s| {
                    s.edges.iter().find(|e| e.id == eid).map(|e| match &e.routing {
                        Routing::Manual { waypoints } => {
                            let mut wps = waypoints.clone();
                            if idx < wps.len() {
                                wps.remove(idx);
                            }
                            if wps.is_empty() {
                                Routing::Orthogonal
                            } else {
                                Routing::Manual { waypoints: wps }
                            }
                        }
                        other => other.clone(),
                    })
                });
                if let Some(routing) = routing {
                    self.registry.update_edge_routing(&eid, routing);
                }
            }
            ContextAction::DeleteSegment(eid, world) => {
                // Resolve the right-click point to a polyline segment, then
                // truncate the wire at it. Works for any routing — the
                // surviving runs become edges with a free endpoint at the
                // cut.
                let seg = self.registry.with_scene(|s| {
                    let e = s.edges.iter().find(|e| e.id == eid)?;
                    let poly = crate::router::edge_polyline(s, e)?;
                    crate::interact::nearest_segment_index(&poly, world)
                });
                if let Some(k) = seg {
                    self.truncate_edge_segments(&eid, &[k]);
                    // Segment indices shift after truncation — drop any
                    // selection entries for this edge so they don't dangle.
                    self.selection.segments.retain(|(e, _)| e != &eid);
                }
            }
            ContextAction::AddPort(nid, world) => {
                let info = self.registry.with_scene(|s| {
                    s.nodes.iter().find(|n| n.id == nid).map(|n| {
                        let anchor = nearest_perimeter_anchor(
                            n.transform.position,
                            n.transform.size,
                            world,
                        );
                        let mut k = n.ports.len();
                        let id = loop {
                            let cand = format!("port{k}");
                            if !n.ports.iter().any(|p| p.id.0 == cand) {
                                break cand;
                            }
                            k += 1;
                        };
                        (id, anchor)
                    })
                });
                if let Some((id, anchor)) = info {
                    self.registry.add_port(
                        &nid,
                        Port {
                            id: PortId(id.clone()),
                            name: id,
                            kind: PortKind::Bidir,
                            anchor,
                            data_type: None,
                        },
                    );
                }
            }
            ContextAction::SetEdgeOverlay(eid, overlay) => {
                self.registry.update_edge_overlay(&eid, overlay);
            }
        }
    }

    fn apply_routing_to_all(&self, routing: Routing) {
        let ids: Vec<_> = self.registry.with_scene(|s| s.edges.iter().map(|e| e.id.clone()).collect());
        for id in ids {
            self.registry.update_edge_routing(&id, routing.clone());
        }
    }

    /// Place a new primitive of the active tool, centered on `world`
    /// (grid-snapped when snapping is on), and select it so it can be
    /// moved or styled immediately.
    fn place_shape(&mut self, world: (f32, f32)) {
        if self.active_tool == ShapeTool::Select {
            return;
        }
        let (snap, spacing) =
            self.registry.with_scene(|s| (s.settings.snap_to_grid, s.settings.grid_spacing));
        let center = if snap { snap_to_grid(world, spacing) } else { world };
        let id = self.registry.with_scene(fresh_node_id);
        let node = make_shape_node(self.active_tool, id.clone(), center);
        self.registry.add_node(node);
        self.selection.select_only(id);
    }
}

const HOTKEY_TABLE: &[(&str, &str)] = &[
    ("G", "Toggle grid"),
    ("X", "Mirror about X axis"),
    ("Y", "Mirror about Y axis"),
    ("R", "Rotate 90° clockwise"),
    ("Del", "Delete selection"),
];

/// A `edge{n}` id not already used by any edge in the scene.
fn fresh_edge_id(scene: &Scene) -> EdgeId {
    let mut n = scene.edges.len();
    loop {
        let candidate = EdgeId(format!("edge{n}"));
        if !scene.edges.iter().any(|e| e.id == candidate) {
            return candidate;
        }
        n += 1;
    }
}

fn fresh_node_id(scene: &Scene) -> NodeId {
    let mut n = scene.nodes.len();
    loop {
        let candidate = NodeId(format!("node{n}"));
        if !scene.nodes.iter().any(|nd| nd.id == candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// Build a default node for a shape tool, centered on `center`. `Square`
/// is a `Rect` with equal sides; `Text` is an unframed label.
fn make_shape_node(tool: ShapeTool, id: NodeId, center: (f32, f32)) -> Node {
    let (kind, size) = match tool {
        ShapeTool::Rect => (NodeKind::Rect, (80.0, 50.0)),
        ShapeTool::Square => (NodeKind::Rect, (60.0, 60.0)),
        ShapeTool::Circle => (NodeKind::Circle, (60.0, 60.0)),
        ShapeTool::Ellipse => (NodeKind::Ellipse, (80.0, 50.0)),
        ShapeTool::Text => (NodeKind::Rect, (80.0, 30.0)),
        // Select never places a node — fall back to a rect defensively.
        ShapeTool::Select => (NodeKind::Rect, (80.0, 50.0)),
    };
    let position = (center.0 - size.0 * 0.5, center.1 - size.1 * 0.5);
    let overlay = if tool == ShapeTool::Text {
        // Unframed: transparent fill and border, just the label.
        Overlay {
            border: Border { color: "#00000000".into(), width: 0.0, style: LineStyle::Solid },
            fill: Fill { color: "#00000000".into(), alpha: 0.0 },
            text: Some(TextLabel {
                value: "Text".into(),
                anchor: TextAnchor::Center,
                font_family: String::new(),
                font_size: 14.0,
                bold: false,
                italic: false,
                color: "#111827".into(),
            }),
        }
    } else {
        Overlay {
            border: Border { color: "#1F2937".into(), width: 2.0, style: LineStyle::Solid },
            fill: Fill { color: "#DBEAFE".into(), alpha: 0.90 },
            text: None,
        }
    };
    Node {
        id,
        kind,
        transform: Transform { position, size, rotation: 0.0 },
        overlay,
        ports: vec![],
    }
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string()
}

fn sep(ui: &mut egui::Ui, vertical: bool) {
    if vertical {
        ui.add_space(6.0);
    } else {
        ui.separator();
    }
}

fn routing_label(r: &Routing) -> &'static str {
    match r {
        Routing::Orthogonal => "Orthogonal",
        Routing::Bezier => "Bezier",
        Routing::Straight => "Straight",
        Routing::Manual { .. } => "Manual",
    }
}

fn grid_style_label(s: GridStyle) -> &'static str {
    match s {
        GridStyle::Lines => "Lines",
        GridStyle::Dots => "Dots",
    }
}

/// Build a viewport positioned to fit a scene's bounds into `screen_rect`
/// with the given padding. Useful for initial app setup before the first
/// frame, when no real canvas rect exists yet.
pub fn fit_viewport_to_scene(scene: &Scene, screen_rect: egui::Rect, padding: f32) -> Viewport {
    match scene_bounds(scene) {
        Some(bounds) => viewport_fit_to(bounds, screen_rect, padding),
        None => Viewport::default(),
    }
}
