//! Property inspector — live editor for the currently selected
//! node or edge.
//!
//! Pattern lifted from the simcore Qt property inspector: one
//! dockable panel, form-style rows of label → field, every field
//! change writes back to the model immediately through the
//! [`Registry`] (so undo / redo continues to work). The right-click
//! context menus stay structural — delete, add connection, etc. —
//! and the inspector owns everything that's about *appearance*.
//!
//! Entry point: [`show_inspector`]. Returns `true` if anything was
//! changed this frame, in case the caller wants to flash a "dirty"
//! marker on the file tab.

use egui::{Color32, DragValue, RichText, Slider, TextEdit};

use crate::interact::Selection;
use crate::model::{
    ArrowHead, LineStyle, Node, NodeKind, Overlay, Routing, TextAnchor, TextLabel,
};
use crate::registry::Registry;

/// Render the inspector body into the given `ui`. The caller is
/// responsible for wrapping it in a panel / scroll area.
///
/// Returns `true` when the user made a change this frame.
pub fn show_inspector(ui: &mut egui::Ui, registry: &Registry, selection: &Selection) -> bool {
    ui.heading("Inspector");
    ui.separator();

    // Single-node selection wins over edges — block-diagram authors
    // care more about widgets than the wires between them, and this
    // mirrors the simcore disambiguation rule.
    if selection.nodes.len() == 1 {
        let id = selection.nodes[0].clone();
        let node = registry.with_scene(|s| s.nodes.iter().find(|n| n.id == id).cloned());
        if let Some(node) = node {
            return show_node_inspector(ui, registry, &node);
        }
        empty_state(ui, "Selected node no longer exists.");
        return false;
    }

    if selection.edges.len() == 1 {
        let id = selection.edges[0].clone();
        let edge = registry.with_scene(|s| s.edges.iter().find(|e| e.id == id).cloned());
        if let Some(edge) = edge {
            return show_edge_inspector(ui, registry, edge);
        }
        empty_state(ui, "Selected wire no longer exists.");
        return false;
    }

    if selection.nodes.is_empty() && selection.edges.is_empty() {
        empty_state(ui, "Select a widget or connection to inspect it.");
    } else {
        let n = selection.nodes.len();
        let e = selection.edges.len();
        empty_state(
            ui,
            &format!(
                "Multi-selection: {n} widget{}{}{e} connection{}.",
                if n == 1 { "" } else { "s" },
                if n > 0 && e > 0 { ", " } else { "" },
                if e == 1 { "" } else { "s" },
            ),
        );
    }
    false
}

fn empty_state(ui: &mut egui::Ui, msg: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(msg).color(Color32::from_gray(140)));
}

// =============================================================================
// Node inspector
// =============================================================================

fn show_node_inspector(ui: &mut egui::Ui, registry: &Registry, node: &Node) -> bool {
    ui.label(RichText::new(format!("Widget — {}", node.id.0)).strong());
    ui.label(
        RichText::new(format!("Kind: {}", kind_label(&node.kind)))
            .small()
            .color(Color32::from_gray(140)),
    );
    ui.add_space(6.0);

    let mut changed = false;

    // Transform — position + size, edited in world units.
    egui::CollapsingHeader::new("Transform")
        .default_open(true)
        .show(ui, |ui| {
            let mut x = node.transform.position.0;
            let mut y = node.transform.position.1;
            let mut w = node.transform.size.0;
            let mut h = node.transform.size.1;
            let mut dirty = false;
            egui::Grid::new("inspect_transform")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("X");
                    if ui.add(DragValue::new(&mut x).speed(1.0)).changed() {
                        dirty = true;
                    }
                    ui.end_row();
                    ui.label("Y");
                    if ui.add(DragValue::new(&mut y).speed(1.0)).changed() {
                        dirty = true;
                    }
                    ui.end_row();
                    ui.label("Width");
                    if ui
                        .add(DragValue::new(&mut w).range(1.0..=10000.0).speed(1.0))
                        .changed()
                    {
                        dirty = true;
                    }
                    ui.end_row();
                    ui.label("Height");
                    if ui
                        .add(DragValue::new(&mut h).range(1.0..=10000.0).speed(1.0))
                        .changed()
                    {
                        dirty = true;
                    }
                    ui.end_row();
                });
            if dirty {
                registry.set_node_transform(&node.id, (x, y), (w, h));
                changed = true;
            }
        });

    // Overlay — border, fill, text. One atomic update per field so
    // each tweak is its own undo step.
    let mut overlay = node.overlay.clone();
    let mut overlay_dirty = false;

    egui::CollapsingHeader::new("Border")
        .default_open(true)
        .show(ui, |ui| {
            if border_section(ui, &mut overlay) {
                overlay_dirty = true;
            }
        });

    egui::CollapsingHeader::new("Fill")
        .default_open(true)
        .show(ui, |ui| {
            if fill_section(ui, &mut overlay) {
                overlay_dirty = true;
            }
        });

    egui::CollapsingHeader::new("Text")
        .default_open(true)
        .show(ui, |ui| {
            if text_section(ui, &mut overlay) {
                overlay_dirty = true;
            }
        });

    if overlay_dirty {
        registry.update_node_overlay(&node.id, overlay);
        changed = true;
    }

    changed
}

fn border_section(ui: &mut egui::Ui, overlay: &mut Overlay) -> bool {
    let mut changed = false;
    let mut rgb = hex_to_rgb(&overlay.border.color);
    if rgb_editor(ui, "Color", &mut rgb) {
        overlay.border.color = rgb_to_hex(rgb);
        changed = true;
    }
    if ui
        .add(Slider::new(&mut overlay.border.width, 0.0..=8.0).text("Width"))
        .changed()
    {
        changed = true;
    }
    let before = overlay.border.style;
    egui::ComboBox::from_id_salt("inspect_border_style")
        .selected_text(line_style_label(overlay.border.style))
        .width(120.0)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut overlay.border.style, LineStyle::Solid, "Solid");
            ui.selectable_value(&mut overlay.border.style, LineStyle::Dashed, "Dashed");
            ui.selectable_value(&mut overlay.border.style, LineStyle::Dotted, "Dotted");
        });
    if before != overlay.border.style {
        changed = true;
    }
    changed
}

fn fill_section(ui: &mut egui::Ui, overlay: &mut Overlay) -> bool {
    let mut changed = false;
    let mut rgb = hex_to_rgb(&overlay.fill.color);
    if rgb_editor(ui, "Color", &mut rgb) {
        overlay.fill.color = rgb_to_hex(rgb);
        changed = true;
    }
    if ui
        .add(Slider::new(&mut overlay.fill.alpha, 0.0..=1.0).text("Alpha"))
        .changed()
    {
        changed = true;
    }
    changed
}

fn text_section(ui: &mut egui::Ui, overlay: &mut Overlay) -> bool {
    let mut changed = false;
    let mut tl = overlay.text.clone().unwrap_or_else(default_text_label);
    let has_text = overlay.text.is_some();
    let mut keep_text = has_text;
    if ui.checkbox(&mut keep_text, "Show label").changed() {
        changed = true;
    }
    if !keep_text {
        if has_text {
            overlay.text = None;
        }
        return changed;
    }

    if ui
        .add(TextEdit::multiline(&mut tl.value).desired_rows(2).hint_text("Label text"))
        .changed()
    {
        changed = true;
    }
    egui::Grid::new("inspect_text_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("Anchor");
            let before = tl.anchor;
            egui::ComboBox::from_id_salt("inspect_text_anchor")
                .selected_text(anchor_label(tl.anchor))
                .width(120.0)
                .show_ui(ui, |ui| {
                    for a in TEXT_ANCHORS {
                        ui.selectable_value(&mut tl.anchor, *a, anchor_label(*a));
                    }
                });
            if before != tl.anchor {
                changed = true;
            }
            ui.end_row();

            ui.label("Size");
            if ui
                .add(Slider::new(&mut tl.font_size, 6.0..=48.0))
                .changed()
            {
                changed = true;
            }
            ui.end_row();

            ui.label("Bold");
            if ui.checkbox(&mut tl.bold, "").changed() {
                changed = true;
            }
            ui.end_row();

            ui.label("Italic");
            if ui.checkbox(&mut tl.italic, "").changed() {
                changed = true;
            }
            ui.end_row();
        });

    let mut rgb = hex_to_rgb(&tl.color);
    if rgb_editor(ui, "Color", &mut rgb) {
        tl.color = rgb_to_hex(rgb);
        changed = true;
    }

    if keep_text {
        overlay.text = Some(tl);
    }
    changed
}

const TEXT_ANCHORS: &[TextAnchor] = &[
    TextAnchor::Center,
    TextAnchor::TopCenter,
    TextAnchor::BottomCenter,
    TextAnchor::Left,
    TextAnchor::Right,
    TextAnchor::TopLeft,
    TextAnchor::TopRight,
    TextAnchor::BottomLeft,
    TextAnchor::BottomRight,
];

fn anchor_label(a: TextAnchor) -> &'static str {
    match a {
        TextAnchor::Center => "Center",
        TextAnchor::TopCenter => "Top",
        TextAnchor::BottomCenter => "Bottom",
        TextAnchor::Left => "Left",
        TextAnchor::Right => "Right",
        TextAnchor::TopLeft => "Top-Left",
        TextAnchor::TopRight => "Top-Right",
        TextAnchor::BottomLeft => "Bottom-Left",
        TextAnchor::BottomRight => "Bottom-Right",
    }
}

fn default_text_label() -> TextLabel {
    TextLabel {
        value: "Text".into(),
        anchor: TextAnchor::Center,
        font_family: String::new(),
        font_size: 12.0,
        bold: false,
        italic: false,
        color: "#111827".into(),
    }
}

fn kind_label(k: &NodeKind) -> &'static str {
    match k {
        NodeKind::Rect => "Rectangle",
        NodeKind::Circle => "Circle",
        NodeKind::Ellipse => "Ellipse",
        NodeKind::Parallelogram => "Parallelogram",
        NodeKind::Path(_) => "Path",
        NodeKind::Group(_) => "Group",
    }
}

// =============================================================================
// Edge inspector
// =============================================================================

fn show_edge_inspector(
    ui: &mut egui::Ui,
    registry: &Registry,
    edge: crate::model::Edge,
) -> bool {
    ui.label(RichText::new(format!("Connection — {}", edge.id.0)).strong());
    let from_lbl = end_label(&edge.from);
    let to_lbl = end_label(&edge.to);
    ui.label(
        RichText::new(format!("{from_lbl}  →  {to_lbl}"))
            .small()
            .color(Color32::from_gray(140)),
    );
    ui.add_space(6.0);

    let mut overlay = edge.overlay.clone();
    let mut routing = edge.routing.clone();
    let mut overlay_dirty = false;
    let mut routing_dirty = false;

    egui::CollapsingHeader::new("Style")
        .default_open(true)
        .show(ui, |ui| {
            let mut rgb = hex_to_rgb(&overlay.color);
            if rgb_editor(ui, "Color", &mut rgb) {
                overlay.color = rgb_to_hex(rgb);
                overlay_dirty = true;
            }
            if ui
                .add(Slider::new(&mut overlay.width, 0.5..=8.0).text("Width"))
                .changed()
            {
                overlay_dirty = true;
            }
            let before = overlay.line_style;
            egui::ComboBox::from_id_salt("inspect_edge_style")
                .selected_text(line_style_label(overlay.line_style))
                .width(120.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut overlay.line_style, LineStyle::Solid, "Solid");
                    ui.selectable_value(&mut overlay.line_style, LineStyle::Dashed, "Dashed");
                    ui.selectable_value(&mut overlay.line_style, LineStyle::Dotted, "Dotted");
                });
            if before != overlay.line_style {
                overlay_dirty = true;
            }
        });

    egui::CollapsingHeader::new("Arrows")
        .default_open(false)
        .show(ui, |ui| {
            if arrow_picker(ui, "Head", "inspect_arrow_head", &mut overlay.arrow_head) {
                overlay_dirty = true;
            }
            if arrow_picker(ui, "Tail", "inspect_arrow_tail", &mut overlay.arrow_tail) {
                overlay_dirty = true;
            }
        });

    egui::CollapsingHeader::new("Routing")
        .default_open(false)
        .show(ui, |ui| {
            let label = routing_label(&routing);
            let before = label.to_string();
            egui::ComboBox::from_id_salt("inspect_edge_routing")
                .selected_text(label)
                .width(140.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(matches!(routing, Routing::Orthogonal), "Orthogonal").clicked() {
                        routing = Routing::Orthogonal;
                    }
                    if ui.selectable_label(matches!(routing, Routing::Bezier), "Bezier").clicked() {
                        routing = Routing::Bezier;
                    }
                    if ui.selectable_label(matches!(routing, Routing::Straight), "Straight").clicked() {
                        routing = Routing::Straight;
                    }
                });
            if before != routing_label(&routing) {
                routing_dirty = true;
            }
        });

    if overlay_dirty {
        registry.update_edge_overlay(&edge.id, overlay);
    }
    if routing_dirty {
        registry.update_edge_routing(&edge.id, routing);
    }
    overlay_dirty || routing_dirty
}

fn arrow_picker(ui: &mut egui::Ui, label: &str, id_salt: &str, value: &mut ArrowHead) -> bool {
    let before = *value;
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(id_salt)
            .selected_text(arrow_label(*value))
            .width(110.0)
            .show_ui(ui, |ui| {
                for a in [
                    ArrowHead::None,
                    ArrowHead::Arrow,
                    ArrowHead::Triangle,
                    ArrowHead::Diamond,
                    ArrowHead::Circle,
                ] {
                    ui.selectable_value(value, a, arrow_label(a));
                }
            });
    });
    before != *value
}

fn arrow_label(a: ArrowHead) -> &'static str {
    match a {
        ArrowHead::None => "None",
        ArrowHead::Arrow => "Arrow",
        ArrowHead::Triangle => "Triangle",
        ArrowHead::Diamond => "Diamond",
        ArrowHead::Circle => "Circle",
    }
}

fn end_label(end: &crate::model::EdgeEnd) -> String {
    match end {
        crate::model::EdgeEnd::Port(n, p) => format!("{}.{}", n.0, p.0),
        crate::model::EdgeEnd::Free(x, y) => format!("({x:.0}, {y:.0})"),
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

// =============================================================================
// Shared helpers
// =============================================================================

fn line_style_label(s: LineStyle) -> &'static str {
    match s {
        LineStyle::Solid => "Solid",
        LineStyle::Dashed => "Dashed",
        LineStyle::Dotted => "Dotted",
    }
}

fn hex_to_rgb(hex: &str) -> [u8; 3] {
    let s = hex.trim_start_matches('#');
    let byte = |i: usize| u8::from_str_radix(s.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    [byte(0), byte(2), byte(4)]
}

fn rgb_to_hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

/// One-row RGB editor: three drag-spinners *and* a clickable
/// swatch that opens egui's HSV / hex picker popup. The popup is
/// safe to use here because the Inspector sits in a regular panel,
/// not inside an auto-closing menu.
fn rgb_editor(ui: &mut egui::Ui, label: &str, rgb: &mut [u8; 3]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        if ui
            .add(DragValue::new(&mut rgb[0]).range(0..=255).speed(1.0).prefix("R "))
            .changed()
        {
            changed = true;
        }
        if ui
            .add(DragValue::new(&mut rgb[1]).range(0..=255).speed(1.0).prefix("G "))
            .changed()
        {
            changed = true;
        }
        if ui
            .add(DragValue::new(&mut rgb[2]).range(0..=255).speed(1.0).prefix("B "))
            .changed()
        {
            changed = true;
        }
        // Clickable color picker — opens a popup with HSV wheel,
        // value sliders, and a hex field. Stays in sync with the
        // R/G/B spinners since they share the same `rgb` buffer.
        if ui.color_edit_button_srgb(rgb).changed() {
            changed = true;
        }
    });
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip() {
        let rgb = [0x1F, 0x29, 0x37];
        let hex = rgb_to_hex(rgb);
        assert_eq!(hex, "#1F2937");
        assert_eq!(hex_to_rgb(&hex), rgb);
    }

    #[test]
    fn malformed_hex_falls_back_to_black() {
        assert_eq!(hex_to_rgb(""), [0, 0, 0]);
        assert_eq!(hex_to_rgb("not a color"), [0, 0, 0]);
    }
}
