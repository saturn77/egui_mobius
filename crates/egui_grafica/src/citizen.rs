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
    apply_resize_delta, hit_test_edge, hit_test_edge_segment, hit_test_free_end, hit_test_node,
    hit_test_port, hit_test_resize_handle, hit_test_waypoint, insert_pivot,
    nearest_perimeter_anchor, prepare_segment_drag, snap_to_grid, CanvasEvent, CanvasFsm,
    CanvasState, HitTarget, Selection,
};
use crate::lang::{self, CommentBlock, ParsedDocument};
use crate::model::{
    Border, CanvasBackground, Edge, EdgeEnd, EdgeEndSide, EdgeId, EdgeOverlay, Fill, GridStyle,
    GridUnits, LineStyle, Node, NodeId, NodeKind, Overlay, Port, PortAnchor, PortId, PortKind,
    Routing, Scene, TextAnchor, TextLabel, Transform,
};
use crate::registry::Registry;
use crate::render::{
    paint_connection_preview, paint_resize_handles, paint_selected_edges,
    paint_selected_segments, paint_selection,
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
    ExportSvg,
    #[cfg(feature = "export-raster")]
    ExportPng,
    #[cfg(feature = "export-raster")]
    ExportJpg,
    #[cfg(feature = "export-pdf")]
    ExportPdf,
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
    Parallelogram,
    Text,
    /// Node-graph widget: a Rect pre-populated with two input ports on
    /// the West face and two output ports on the East face. Drops on
    /// the canvas the same way as the other primitives.
    NodeGraph,
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
    /// `Some(id)` when the named node is in inline text-edit mode —
    /// a TextEdit is overlaid on the node's centre and other canvas
    /// gestures are suppressed until edit ends.
    editing_node: Option<NodeId>,
    /// Live buffer for the inline text edit. Pushed to the node's
    /// overlay.text every frame so the canvas reflects keystrokes
    /// without a confirm step.
    edit_buffer: String,
    /// Page setup modal — open / closed flag persisted across frames
    /// so the window stays put and the text-edits keep their state.
    show_page_modal: bool,
}

/// An action chosen from the right-click context menu, applied after the
/// menu closure returns.
enum ContextAction {
    DeleteEdge(EdgeId),
    DeleteSegment(EdgeId, (f32, f32)),
    DeletePivot(EdgeId, usize),
    AddPort(NodeId, (f32, f32)),
    SetEdgeOverlay(EdgeId, EdgeOverlay),
    SetNodeOverlay(NodeId, Overlay),
}

fn hex_to_rgb(hex: &str) -> [u8; 3] {
    let s = hex.trim_start_matches('#');
    let byte = |i: usize| u8::from_str_radix(s.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    [byte(0), byte(2), byte(4)]
}

fn rgb_to_hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

/// Compact inline RGB editor — three drag-value spinners plus a colour
/// swatch preview. Self-contained: no popups, so it composes inside
/// nested menus without fighting auto-close behaviour.
fn inline_color_editor(ui: &mut egui::Ui, rgb: &mut [u8; 3]) {
    ui.horizontal(|ui| {
        ui.label("Color");
        ui.add(egui::DragValue::new(&mut rgb[0]).range(0..=255).speed(1.0).prefix("R "));
        ui.add(egui::DragValue::new(&mut rgb[1]).range(0..=255).speed(1.0).prefix("G "));
        ui.add(egui::DragValue::new(&mut rgb[2]).range(0..=255).speed(1.0).prefix("B "));
        let (swatch, _) =
            ui.allocate_exact_size(egui::vec2(22.0, 18.0), egui::Sense::hover());
        ui.painter().rect_filled(
            swatch,
            2.0,
            egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
        );
    });
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
            editing_node: None,
            edit_buffer: String::new(),
            show_page_modal: false,
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

        // Page setup modal — rendered last so it floats on top of the
        // panels. Stays open across frames; closes only via the X or
        // the Close button inside the window.
        self.show_page_setup_window(ui.ctx());
    }

    /// Free-floating Page setup dialog. Reads / writes settings
    /// through the registry, with the modal's open / closed state
    /// kept on `self` so re-renders don't lose user focus or values.
    fn show_page_setup_window(&mut self, ctx: &egui::Context) {
        if !self.show_page_modal {
            return;
        }
        let mut open = true;
        let mut settings = self.registry.with_scene(|s| s.settings.clone());
        let mut changed = false;
        let mut close_clicked = false;

        egui::Window::new("Page setup")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                changed = page_setup_body(ui, &mut settings);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        close_clicked = true;
                    }
                });
            });

        self.show_page_modal = open && !close_clicked;
        if changed {
            self.registry.update_settings(settings);
        }
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
            btn(ui, ShapeTool::Parallelogram, ico::PARALLELOGRAM, "Parallelogram");
            btn(ui, ShapeTool::Text, ico::TEXT_T, "Text");
            btn(
                ui,
                ShapeTool::NodeGraph,
                ico::TREE_STRUCTURE,
                "Node-graph widget — 2 in / 2 out",
            );

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
                // Tighten widths *before* the wrapped row begins so each
                // child widget reports a small min-size — the wrap
                // algorithm only breaks when the next item plus the
                // current row exceeds the available width, and the
                // default 100 px slider + 100 px combo defaults blow
                // past any narrow dock-tab in one shot otherwise.
                ui.spacing_mut().slider_width = 90.0;
                ui.spacing_mut().combo_width = 92.0;
                ui.spacing_mut().interact_size.x = 28.0;
                ui.spacing_mut().item_spacing.x = 4.0;
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
                ui.separator();
                if ui.button(format!("{} Export SVG…", ico::FILE)).clicked() {
                    file_action = Some(FileAction::ExportSvg);
                    ui.close();
                }
                #[cfg(feature = "export-raster")]
                {
                    if ui.button(format!("{} Export PNG…", ico::FILE)).clicked() {
                        file_action = Some(FileAction::ExportPng);
                        ui.close();
                    }
                    if ui.button(format!("{} Export JPG…", ico::FILE)).clicked() {
                        file_action = Some(FileAction::ExportJpg);
                        ui.close();
                    }
                }
                #[cfg(feature = "export-pdf")]
                {
                    if ui.button(format!("{} Export PDF…", ico::FILE)).clicked() {
                        file_action = Some(FileAction::ExportPdf);
                        ui.close();
                    }
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
                .width(80.0)
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
                .width(96.0)
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
                .width(92.0)
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
                .width(100.0)
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

            // Page setup lives in a modal — clicking opens / closes
            // a Window that persists across frames, so text-edits keep
            // focus and the dialog doesn't auto-close on every gesture
            // the way an inline menu_button does.
            if ui.button(format!("{} Page…", ico::FILE)).clicked() {
                self.show_page_modal = !self.show_page_modal;
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
            FileAction::ExportSvg => self.export_canvas("svg"),
            #[cfg(feature = "export-raster")]
            FileAction::ExportPng => self.export_canvas("png"),
            #[cfg(feature = "export-raster")]
            FileAction::ExportJpg => self.export_canvas("jpg"),
            #[cfg(feature = "export-pdf")]
            FileAction::ExportPdf => self.export_canvas("pdf"),
        }
    }

    /// Pop an rfd save dialog and write the scene out in the
    /// requested format. Dispatches to the right exporter — the
    /// raster + pdf paths are feature-gated.
    fn export_canvas(&mut self, ext: &str) {
        let default_name = self
            .current_path
            .as_ref()
            .and_then(|p| p.file_stem().and_then(|s| s.to_str()))
            .unwrap_or("canvas")
            .to_string();
        let label = ext.to_ascii_uppercase();
        let Some(path) = rfd::FileDialog::new()
            .add_filter(&label, &[ext])
            .set_file_name(format!("{default_name}.{ext}"))
            .save_file()
        else {
            return;
        };
        let region = if self
            .registry
            .with_scene(|s| s.settings.paper_size.is_some())
        {
            crate::export::Region::Page
        } else {
            crate::export::Region::Content { padding_world: 24.0 }
        };
        let opts = crate::export::ExportOptions {
            region,
            background: None,
            include_grid: false,
            include_ports: true,
        };
        let result = self.registry.with_scene(|scene| -> std::io::Result<()> {
            match ext {
                "svg" => crate::export::export_svg(scene, &opts, &path),
                #[cfg(feature = "export-raster")]
                "png" => crate::export::export_png(scene, &opts, &path, 150.0),
                #[cfg(feature = "export-raster")]
                "jpg" => crate::export::export_jpg(scene, &opts, &path, 150.0),
                #[cfg(feature = "export-pdf")]
                "pdf" => crate::export::export_pdf(scene, &opts, &path),
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("unsupported export format: {ext}"),
                )),
            }
        });
        self.status = match result {
            Ok(()) => format!("Exported {} → {}", label, file_name(&path)),
            Err(e) => format!("Export failed: {e}"),
        };
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

        // Inline text-edit mode swallows all canvas gestures and
        // hotkeys until the editor loses focus. The TextEdit overlay
        // is rendered at the end of this function and owns the
        // keyboard while editing.
        let editing = self.editing_node.is_some();

        // ── Press: classify what was hit and drive the FSM ──
        //
        // Gated to the PRIMARY button — `Sense::click_and_drag()` reports
        // drags from any button, so without this the right button also
        // panned and right/left events interleaved into the FSM.
        //
        // Hit-test at the true press origin, not `interact_pointer_pos` —
        // egui only reports a drag once the pointer has moved a few pixels,
        // and testing that drifted point misses thin wires and small ports.
        if !editing
            && self.active_tool == ShapeTool::Select
            && response.drag_started_by(egui::PointerButton::Primary)
            && let Some(screen) = ui.input(|i| i.pointer.press_origin())
        {
            let world = self.viewport.screen_to_world(screen);
            let port_radius = PORT_GRAB_PX / self.viewport.zoom;
            let edge_thresh = EDGE_GRAB_PX / self.viewport.zoom;

            // Priority: pivot vertex > free end > resize handle > port
            // > node body > wire > empty. Resize handles only exist on
            // *selected* nodes — that's why they win over a coincident
            // port: the selection said "I'm editing this node."
            let waypoint_hit =
                self.registry.with_scene(|s| hit_test_waypoint(s, world, port_radius));
            let free_end_hit =
                self.registry.with_scene(|s| hit_test_free_end(s, world, port_radius));
            let resize_hit = self.registry.with_scene(|s| {
                hit_test_resize_handle(s, &self.selection.nodes, world, port_radius)
            });
            let port_hit = self.registry.with_scene(|s| hit_test_port(s, world, port_radius));
            let node_hit = self.registry.with_scene(|s| hit_test_node(s, world));
            let edge_hit = self.registry.with_scene(|s| hit_test_edge(s, world, edge_thresh));
            let target = if waypoint_hit.is_some() {
                HitTarget::Waypoint
            } else if free_end_hit.is_some() {
                HitTarget::FreeEnd
            } else if resize_hit.is_some() {
                HitTarget::ResizeHandle
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

            // Frame-by-frame drags would otherwise flood the undo
            // stack — collapse the whole gesture into one undoable
            // step by opening a batch on press and closing on release.
            // Marquee is the lone exception: it doesn't mutate the
            // scene at all, so a batch would just leave a no-op
            // snapshot behind.
            if self.fsm.state != CanvasState::Idle
                && self.fsm.state != CanvasState::Marquee
            {
                self.registry.begin_undo_batch();
            }

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
                CanvasState::ResizingNode => {
                    let (eid, handle) =
                        resize_hit.expect("ResizeHandle target implies a resize-handle hit");
                    let (pos, size) = self
                        .registry
                        .with_scene(|s| {
                            s.nodes
                                .iter()
                                .find(|n| n.id == eid)
                                .map(|n| (n.transform.position, n.transform.size))
                        })
                        .unwrap_or_default();
                    self.fsm.resize_node = Some(eid);
                    self.fsm.resize_handle = Some(handle);
                    self.fsm.resize_origin_pos = pos;
                    self.fsm.resize_origin_size = size;
                }
                _ => {}
            }
        }

        // ── Drag: act on the FSM's current state (primary button only) ──
        if !editing && response.dragged_by(egui::PointerButton::Primary) {
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
                CanvasState::ResizingNode => {
                    if let Some(eid) = self.fsm.resize_node.clone()
                        && let Some(handle) = self.fsm.resize_handle
                        && let Some(screen) = response.interact_pointer_pos()
                    {
                        let cursor = self.viewport.screen_to_world(screen);
                        let delta = (
                            cursor.0 - self.fsm.grab_world.0,
                            cursor.1 - self.fsm.grab_world.1,
                        );
                        let (snap, spacing) = self.registry.with_scene(|s| {
                            (s.settings.snap_to_grid, s.settings.grid_spacing)
                        });
                        const MIN_SIZE: f32 = 10.0;
                        let (pos, size) = apply_resize_delta(
                            handle,
                            self.fsm.resize_origin_pos,
                            self.fsm.resize_origin_size,
                            delta,
                            MIN_SIZE,
                            if snap { spacing } else { 0.0 },
                        );
                        self.registry.set_node_transform(&eid, pos, size);
                    }
                }
                CanvasState::Idle => {}
            }
        }

        // ── Pan the viewport — independent of the FSM. ──
        //
        // Right-mouse-drag pans the canvas — standard CAD gesture.
        // Middle-button drag is kept as a fallback for trackpads /
        // setups without a usable secondary button. A right-click
        // *without* drag still opens the context menu, because
        // `context_menu` triggers on release-without-drag, so an
        // RMB drag never resolves to a menu open.
        let pan = !editing
            && (response.dragged_by(egui::PointerButton::Secondary)
                || response.dragged_by(egui::PointerButton::Middle));
        if pan {
            let delta = response.drag_delta();
            self.viewport.origin.x += delta.x;
            self.viewport.origin.y += delta.y;
        }

        // ── Release: finalise the gesture, return the FSM to Idle ──
        if !editing && response.drag_stopped_by(egui::PointerButton::Primary) {
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
            // Close the undo batch opened on press if the gesture
            // had one (everything except Marquee).
            let had_batch = self.fsm.state != CanvasState::Idle
                && self.fsm.state != CanvasState::Marquee;
            self.fsm.dispatch(CanvasEvent::Release, HitTarget::Empty);
            if had_batch {
                self.registry.end_undo_batch();
            }
        }

        // Click (press + release, no movement): select node, else wire, else clear.
        if !editing
            && response.clicked()
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

        if !editing && response.hovered() {
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
            // Plain-letter hotkeys ignore Ctrl so Ctrl+Z / Ctrl+Y don't
            // also fire 'Z' / 'Y' actions (e.g. the Y-axis mirror).
            let mods = ui.input(|i| i.modifiers);
            let plain = !mods.ctrl && !mods.alt;
            let (g, x, y, r, a, z, yk) = ui.input(|i| {
                (
                    i.key_pressed(Key::G),
                    i.key_pressed(Key::X),
                    i.key_pressed(Key::Y),
                    i.key_pressed(Key::R),
                    i.key_pressed(Key::A),
                    i.key_pressed(Key::Z),
                    i.key_pressed(Key::Y),
                )
            });
            if plain {
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
                // A — snap every selected node to the nearest grid
                // intersection. Adjacent Manual-wire waypoints follow
                // via the move_nodes path so wires don't tear off.
                if a && !self.selection.nodes.is_empty() {
                    let ids = self.selection.nodes.clone();
                    self.registry.align_selection_to_grid(&ids);
                }
            }
            // Undo / redo. Ctrl+Z undoes; Ctrl+Shift+Z and Ctrl+Y both
            // redo. Drags and inline text edits batch through
            // Registry::begin_undo_batch so each gesture is one
            // undoable step rather than per-frame or per-keystroke.
            if mods.ctrl && z {
                if mods.shift {
                    self.registry.redo();
                } else {
                    self.registry.undo();
                }
            }
            if mods.ctrl && !mods.shift && yk {
                self.registry.redo();
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
        if !editing
            && response.double_clicked()
            && let Some(screen) = response.interact_pointer_pos()
        {
            let world = self.viewport.screen_to_world(screen);
            let port_radius = PORT_GRAB_PX / self.viewport.zoom;
            let edge_thresh = EDGE_GRAB_PX / self.viewport.zoom;

            if let Some(nid) = self.registry.with_scene(|s| hit_test_node(s, world)) {
                // Double-click a node body → enter inline text-edit
                // mode. Centred TextEdit overlays the node; type to
                // change the label, Enter / Escape / click-outside
                // exits.
                let current_text = self
                    .registry
                    .with_scene(|s| {
                        s.nodes
                            .iter()
                            .find(|n| n.id == nid)
                            .and_then(|n| n.overlay.text.as_ref().map(|t| t.value.clone()))
                    })
                    .unwrap_or_default();
                // Treat the "Text" placeholder as empty on entry so a
                // newly-placed shape lets the user type a fresh label.
                self.edit_buffer = if current_text == "Text" {
                    String::new()
                } else {
                    current_text
                };
                self.editing_node = Some(nid);
                // Collapse the whole edit session into one undo step
                // — every keystroke would otherwise push its own
                // snapshot through update_node_overlay.
                self.registry.begin_undo_batch();
            } else if let Some((eid, idx)) =
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
        if !editing
            && response.secondary_clicked()
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
        let node_overlay = hit_node.as_ref().and_then(|nid| {
            self.registry
                .with_scene(|s| s.nodes.iter().find(|n| &n.id == nid).map(|n| n.overlay.clone()))
        });
        let mut action: Option<ContextAction> = None;
        // Priority: pivot > node > edge > empty. Right-clicking inside
        // a node body reaches the node menu even when a wire passes
        // through the click — clicks land *on the shape*. Wire editing
        // is still available by right-clicking the wire outside any
        // node body. The whole menu is suppressed during inline text
        // editing.
        if !editing {
            response.context_menu(|ui| {
                if let Some((eid, idx)) = &hit_wp {
                    if ui.button("Delete pivot").clicked() {
                        action = Some(ContextAction::DeletePivot(eid.clone(), *idx));
                        ui.close();
                    }
                } else if let Some(nid) = &hit_node {
                    // Label editing isn't on the context menu — it
                    // lives on double-click of the node body. The menu
                    // only carries the style + structural edits.
                    if let Some(overlay) = &node_overlay {
                        ui.menu_button("Border", |ui| {
                            let mut next = overlay.clone();
                            let mut rgb = hex_to_rgb(&next.border.color);
                            inline_color_editor(ui, &mut rgb);
                            next.border.color = rgb_to_hex(rgb);
                            ui.add(
                                egui::Slider::new(&mut next.border.width, 0.0..=8.0).text("Width"),
                            );
                            // Dashed / dotted not rendered yet — the
                            // selector returns when the perimeter dash
                            // pattern lands in the node shader.
                            if next != *overlay {
                                action = Some(ContextAction::SetNodeOverlay(nid.clone(), next));
                            }
                        });
                        ui.separator();
                    }
                    if ui.button("Add connection").clicked() {
                        action = Some(ContextAction::AddPort(nid.clone(), ctx.unwrap_or_default()));
                        ui.close();
                    }
                } else if let Some(eid) = &hit_edge {
                    if ui.button("Delete segment").clicked() {
                        action = Some(ContextAction::DeleteSegment(
                            eid.clone(),
                            ctx.unwrap_or_default(),
                        ));
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
                            inline_color_editor(ui, &mut rgb);
                            next.color = rgb_to_hex(rgb);
                            ui.add(
                                egui::Slider::new(&mut next.width, 0.5..=6.0).text("Width"),
                            );
                            ui.separator();
                            for style in [LineStyle::Solid, LineStyle::Dashed, LineStyle::Dotted] {
                                if ui
                                    .selectable_label(
                                        next.line_style == style,
                                        line_style_label(style),
                                    )
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
                } else {
                    ui.label(egui::RichText::new("Nothing here").weak());
                }
            });
            if let Some(action) = action {
                self.apply_context_action(action);
            }
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
            // Page board on the painter — frame + title block sits on
            // top of the GPU-rendered nodes. The scene lock is already
            // held, so read settings from `scene` (don't re-lock).
            crate::page::paint_page(&painter, &self.viewport, &scene.settings);
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
            // Page board on top of the grid so the sheet covers it
            // wherever the paper sits — same visual model simcore uses.
            crate::page::paint_page(&painter, &self.viewport, &settings);
            self.registry.with_scene(|scene| paint_scene(&painter, scene, &self.viewport));
        }

        // Selection highlights — painter-side on both paths.
        self.registry.with_scene(|scene| {
            paint_selected_edges(&painter, scene, &self.selection.edges, &self.viewport);
            paint_selected_segments(&painter, scene, &self.selection.segments, &self.viewport);
            paint_selection(&painter, scene, &self.selection.nodes, &self.viewport);
            paint_resize_handles(&painter, scene, &self.selection.nodes, &self.viewport);
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

        // ── Inline text-edit overlay ──
        //
        // Sits on top of the canvas paint, takes keyboard focus, and
        // pushes the buffer into the node's overlay.text each frame so
        // the canvas reflects keystrokes live. Enter, Escape, or
        // clicking outside the editor commits and exits.
        if self.editing_node.is_some()
            && let Some(resp) = self.render_inline_edit(ui)
        {
            let escape = ui.input(|i| i.key_pressed(Key::Escape));
            if resp.lost_focus() || escape {
                self.editing_node = None;
                self.edit_buffer.clear();
                self.registry.end_undo_batch();
            }
        }
    }

    /// Render the inline TextEdit overlay for the currently-editing
    /// node. Returns the egui response if a TextEdit was drawn, so the
    /// caller can detect Enter / Escape / click-outside via
    /// `.lost_focus()`. Returns `None` if `editing_node` is `None` or
    /// the node has been deleted out from under us.
    fn render_inline_edit(&mut self, ui: &mut egui::Ui) -> Option<egui::Response> {
        let nid = self.editing_node.clone()?;
        let (pos, size) = self.registry.with_scene(|s| {
            s.nodes
                .iter()
                .find(|n| n.id == nid)
                .map(|n| (n.transform.position, n.transform.size))
        })?;

        // Centre the editor on the node's centroid, width clamped so
        // the field stays grabbable on tiny nodes and not absurd on
        // huge ones.
        let center_world = (pos.0 + size.0 * 0.5, pos.1 + size.1 * 0.5);
        let center_screen = self.viewport.world_to_screen(center_world);
        let edit_w = (size.0 * self.viewport.zoom * 0.85).clamp(80.0, 240.0);
        let edit_h = 22.0;
        let rect = egui::Rect::from_center_size(center_screen, egui::vec2(edit_w, edit_h));

        let id = ui.make_persistent_id(("grafica_inline_edit", nid.0.clone()));
        let text_edit = egui::TextEdit::singleline(&mut self.edit_buffer)
            .id(id)
            .horizontal_align(egui::Align::Center)
            .hint_text("Label…")
            .desired_width(edit_w - 8.0);
        let resp = ui.put(rect, text_edit);

        // Keep focus on the editor until it explicitly loses it.
        if !resp.has_focus() && !resp.lost_focus() {
            resp.request_focus();
        }

        // Push the live buffer to the node's text every frame the
        // value changes. Empty buffer clears the label.
        let buf = self.edit_buffer.clone();
        let overlay_opt = self
            .registry
            .with_scene(|s| s.nodes.iter().find(|n| n.id == nid).map(|n| n.overlay.clone()));
        if let Some(mut overlay) = overlay_opt {
            let current = overlay.text.as_ref().map(|t| t.value.clone()).unwrap_or_default();
            if current != buf {
                if buf.is_empty() {
                    overlay.text = None;
                } else if let Some(t) = overlay.text.as_mut() {
                    t.value = buf;
                } else {
                    overlay.text = Some(TextLabel {
                        value: buf,
                        anchor: TextAnchor::Center,
                        font_family: String::new(),
                        font_size: 12.0,
                        bold: false,
                        italic: false,
                        color: "#111827".into(),
                    });
                }
                self.registry.update_node_overlay(&nid, overlay);
            }
        }

        Some(resp)
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
            ContextAction::SetNodeOverlay(nid, overlay) => {
                self.registry.update_node_overlay(&nid, overlay);
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
    ("A", "Align selection to grid"),
    ("Del", "Delete selection"),
    ("Esc", "Disarm shape tool"),
    ("Ctrl+Z", "Undo"),
    ("Ctrl+Shift+Z", "Redo"),
    ("Ctrl+Y", "Redo (alt)"),
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
        ShapeTool::Parallelogram => (NodeKind::Parallelogram, (80.0, 50.0)),
        ShapeTool::Text => (NodeKind::Rect, (80.0, 30.0)),
        // Node-graph widget: a slightly taller rect so the four ports
        // distribute cleanly without crowding the label.
        ShapeTool::NodeGraph => (NodeKind::Rect, (120.0, 80.0)),
        // Select never places a node — fall back to a rect defensively.
        ShapeTool::Select => (NodeKind::Rect, (80.0, 50.0)),
    };
    let position = (center.0 - size.0 * 0.5, center.1 - size.1 * 0.5);
    // Every placed shape gets a centred "Text" placeholder label so it
    // reads as editable on sight — double-click the node body to
    // change it inline. The Text tool is the un-framed variant; the
    // rest carry the standard fill + border.
    let text_label = TextLabel {
        value: default_label_for(tool).into(),
        anchor: TextAnchor::Center,
        font_family: String::new(),
        font_size: if tool == ShapeTool::Text { 14.0 } else { 12.0 },
        bold: false,
        italic: false,
        color: "#111827".into(),
    };
    let overlay = if tool == ShapeTool::Text {
        Overlay {
            border: Border { color: "#00000000".into(), width: 0.0, style: LineStyle::Solid },
            fill: Fill { color: "#00000000".into(), alpha: 0.0 },
            text: Some(text_label),
        }
    } else {
        Overlay {
            border: Border { color: "#1F2937".into(), width: 2.0, style: LineStyle::Solid },
            fill: Fill { color: "#DBEAFE".into(), alpha: 0.90 },
            text: Some(text_label),
        }
    };
    Node {
        id,
        kind,
        transform: Transform { position, size, rotation: 0.0 },
        overlay,
        ports: default_ports_for(tool),
    }
}

/// Placeholder text for a freshly-placed shape. NodeGraph widgets get
/// a more descriptive default so the role is obvious at a glance.
fn default_label_for(tool: ShapeTool) -> &'static str {
    match tool {
        ShapeTool::NodeGraph => "Node",
        _ => "Text",
    }
}

/// Pre-populated ports for a freshly-placed shape. Only NodeGraph
/// has any — block-diagram primitives stay portless until the user
/// wires from a body face.
fn default_ports_for(tool: ShapeTool) -> Vec<Port> {
    match tool {
        ShapeTool::NodeGraph => vec![
            Port {
                id: PortId("in0".into()),
                name: "in0".into(),
                kind: PortKind::In,
                anchor: PortAnchor::West(0.30),
                data_type: None,
            },
            Port {
                id: PortId("in1".into()),
                name: "in1".into(),
                kind: PortKind::In,
                anchor: PortAnchor::West(0.70),
                data_type: None,
            },
            Port {
                id: PortId("out0".into()),
                name: "out0".into(),
                kind: PortKind::Out,
                anchor: PortAnchor::East(0.30),
                data_type: None,
            },
            Port {
                id: PortId("out1".into()),
                name: "out1".into(),
                kind: PortKind::Out,
                anchor: PortAnchor::East(0.70),
                data_type: None,
            },
        ],
        _ => vec![],
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

/// Page-setup modal body: paper size, orientation, and title-block
/// fields. Returns `true` whenever the user changed anything — caller
/// pushes the mutated settings back into the registry.
fn page_setup_body(ui: &mut egui::Ui, settings: &mut crate::model::CanvasSettings) -> bool {
    use crate::page::PAPER_SIZES_INCHES;
    let mut changed = false;

    ui.label(egui::RichText::new("Paper").strong());
    ui.horizontal(|ui| {
        ui.label("Size");
        let selected_paper = settings.paper_size.clone().unwrap_or_else(|| "None".into());
        egui::ComboBox::from_id_salt("grafica_paper_size")
            .selected_text(&selected_paper)
            .width(160.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(settings.paper_size.is_none(), "None")
                    .clicked()
                {
                    settings.paper_size = None;
                    changed = true;
                }
                for (name, _) in PAPER_SIZES_INCHES {
                    if ui
                        .selectable_label(
                            settings.paper_size.as_deref() == Some(*name),
                            *name,
                        )
                        .clicked()
                    {
                        settings.paper_size = Some((*name).into());
                        changed = true;
                    }
                }
            });
    });

    let mut is_landscape = matches!(
        settings.paper_orientation.as_deref(),
        Some(o) if o.eq_ignore_ascii_case("landscape")
    );
    ui.horizontal(|ui| {
        ui.label("Orientation");
        if ui.radio(!is_landscape, "Portrait").clicked() {
            is_landscape = false;
            settings.paper_orientation = Some("portrait".into());
            changed = true;
        }
        if ui.radio(is_landscape, "Landscape").clicked() {
            is_landscape = true;
            settings.paper_orientation = Some("landscape".into());
            changed = true;
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.label(egui::RichText::new("Title block").strong());

    let mut tb = settings.title_block.clone().unwrap_or_default();
    let mut tb_present = settings.title_block.is_some();
    if ui.checkbox(&mut tb_present, "Show title block").changed() {
        settings.title_block = if tb_present { Some(tb.clone()) } else { None };
        changed = true;
    }
    if tb_present {
        egui::Grid::new("grafica_title_block_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                let mut field = |ui: &mut egui::Ui, label: &str, value: &mut String| {
                    ui.label(egui::RichText::new(label).monospace().small());
                    if ui
                        .add(egui::TextEdit::singleline(value).desired_width(240.0))
                        .changed()
                    {
                        changed = true;
                    }
                    ui.end_row();
                };
                field(ui, "TITLE", &mut tb.title);
                field(ui, "COMPANY", &mut tb.company);
                field(ui, "DWG NO", &mut tb.drawing_no);
                field(ui, "REV", &mut tb.revision);
                field(ui, "DATE", &mut tb.date);
                field(ui, "DRAWN BY", &mut tb.drawn_by);
                field(ui, "SHEET", &mut tb.sheet);
            });
        if changed {
            settings.title_block = Some(tb);
        }
    }

    changed
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_graph_tool_places_a_widget_with_two_in_and_two_out_ports() {
        let node =
            make_shape_node(ShapeTool::NodeGraph, NodeId("n".into()), (0.0, 0.0));
        assert_eq!(node.ports.len(), 4);
        let ins: Vec<_> = node.ports.iter().filter(|p| p.kind == PortKind::In).collect();
        let outs: Vec<_> = node.ports.iter().filter(|p| p.kind == PortKind::Out).collect();
        assert_eq!(ins.len(), 2);
        assert_eq!(outs.len(), 2);
        assert!(matches!(ins[0].anchor, PortAnchor::West(_)));
        assert!(matches!(outs[0].anchor, PortAnchor::East(_)));
        // Visibly larger than the base rect so four ports distribute cleanly.
        assert!(node.transform.size.0 >= 100.0);
        assert!(node.transform.size.1 >= 60.0);
        // More descriptive default label than the generic "Text".
        assert_eq!(node.overlay.text.as_ref().unwrap().value, "Node");
    }

    #[test]
    fn other_shape_tools_remain_portless() {
        for tool in [
            ShapeTool::Rect,
            ShapeTool::Square,
            ShapeTool::Circle,
            ShapeTool::Ellipse,
            ShapeTool::Parallelogram,
            ShapeTool::Text,
        ] {
            let node = make_shape_node(tool, NodeId("x".into()), (0.0, 0.0));
            assert!(node.ports.is_empty(), "{tool:?} should not seed ports");
        }
    }
}
