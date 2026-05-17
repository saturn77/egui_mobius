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

use egui::{Color32, Key, Sense};
use egui_phosphor::regular as ico;

use crate::interact::{hit_test_node, snap_to_grid, DragState, Selection};
use crate::model::{GridStyle, GridUnits, Routing, Scene};
use crate::registry::Registry;
use crate::render::{paint_scene, paint_selection, scene_bounds, viewport_fit_to, Viewport};

/// Which side of the citizen the ribbon docks to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RibbonSide {
    Top,
    Bottom,
    Left,
    Right,
}

/// The canvas citizen widget.
pub struct CanvasCitizen {
    pub registry: Registry,
    pub viewport: Viewport,
    pub fit_padding: f32,
    pub routing_picker_applies_to_all: bool,
    pub ribbon_side: RibbonSide,
    /// Currently-selected nodes.
    pub selection: Selection,
    /// In-progress pointer gesture (pan vs node-drag).
    drag: DragState,
    /// Set by the Fit button; consumed in the canvas pass where the real
    /// canvas rect is available.
    pending_fit: bool,
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
            selection: Selection::default(),
            drag: DragState::Idle,
            pending_fit: false,
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
        egui::CentralPanel::default().show_inside(ui, |ui| self.show_canvas(ui));
    }

    fn show_ribbon(&mut self, ui: &mut egui::Ui, vertical: bool) {
        let lay = |ui: &mut egui::Ui, body: &mut dyn FnMut(&mut egui::Ui)| {
            if vertical {
                ui.vertical(body);
            } else {
                ui.horizontal(body);
            }
        };

        let mut settings = self.registry.with_scene(|s| s.settings.clone());
        let mut settings_changed = false;
        let mut routing_changed_to: Option<Routing> = None;
        let mut dock_to: Option<RibbonSide> = None;
        let mut reset_clicked = false;

        lay(ui, &mut |ui| {
            ui.label(egui::RichText::new(format!("{} Grafica", ico::PALETTE)).strong());
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
    }

    fn show_canvas(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = response.rect;

        if self.pending_fit {
            self.fit_to_rect(rect);
            self.pending_fit = false;
        }

        let shift = ui.input(|i| i.modifiers.shift);

        // Press: hit-test to decide pan vs node-drag, and prime the selection
        // so the user sees what they're about to move.
        if response.drag_started()
            && let Some(screen) = response.interact_pointer_pos()
        {
            let world = self.viewport.screen_to_world(screen);
            match self.registry.with_scene(|s| hit_test_node(s, world)) {
                Some(id) => {
                    if !self.selection.contains(&id) {
                        if shift {
                            self.selection.toggle(id.clone());
                        } else {
                            self.selection.select_only(id.clone());
                        }
                    }
                    let origins = self.registry.with_scene(|s| {
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
                    self.drag = DragState::Nodes { grab_world: world, origins };
                }
                None => self.drag = DragState::Pan,
            }
        }

        // Drag continue.
        if response.dragged() {
            match &self.drag {
                DragState::Pan => {
                    let delta = response.drag_delta();
                    self.viewport.origin.x += delta.x;
                    self.viewport.origin.y += delta.y;
                }
                DragState::Nodes { grab_world, origins } => {
                    if let Some(screen) = response.interact_pointer_pos() {
                        let now = self.viewport.screen_to_world(screen);
                        let (dx, dy) = (now.0 - grab_world.0, now.1 - grab_world.1);
                        let (snap, spacing) = self
                            .registry
                            .with_scene(|s| (s.settings.snap_to_grid, s.settings.grid_spacing));
                        let moves: Vec<_> = origins
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
                DragState::Idle => {}
            }
        }

        if response.drag_stopped() {
            self.drag = DragState::Idle;
        }

        // Click (press + release, no movement): update selection.
        if response.clicked()
            && let Some(screen) = response.interact_pointer_pos()
        {
            let world = self.viewport.screen_to_world(screen);
            match self.registry.with_scene(|s| hit_test_node(s, world)) {
                Some(id) => {
                    if shift {
                        self.selection.toggle(id);
                    } else {
                        self.selection.select_only(id);
                    }
                }
                None => {
                    if !shift {
                        self.selection.clear();
                    }
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
        }

        if response.double_clicked() {
            self.fit_to_rect(rect);
        }

        painter.rect_filled(rect, 0.0, Color32::from_rgb(0xF8, 0xFA, 0xFC));

        self.registry.with_scene(|scene| {
            paint_scene(&painter, scene, &self.viewport, rect);
            paint_selection(&painter, scene, &self.selection.nodes, &self.viewport);
        });
    }

    fn fit_to_rect(&mut self, screen_rect: egui::Rect) {
        let bounds = self.registry.with_scene(scene_bounds);
        if let Some(b) = bounds {
            self.viewport = viewport_fit_to(b, screen_rect, self.fit_padding);
        }
    }

    fn apply_routing_to_all(&self, routing: Routing) {
        let ids: Vec<_> = self.registry.with_scene(|s| s.edges.iter().map(|e| e.id.clone()).collect());
        for id in ids {
            self.registry.update_edge_routing(&id, routing.clone());
        }
    }
}

const HOTKEY_TABLE: &[(&str, &str)] = &[
    ("G", "Toggle grid"),
    ("X", "Mirror about X axis"),
    ("Y", "Mirror about Y axis"),
    ("R", "Rotate 90° clockwise"),
];

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
        Routing::Manual(_) => "Manual",
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
