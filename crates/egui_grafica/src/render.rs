//! egui `Painter` rendering of a [`Scene`].
//!
//! This is the only module besides [`crate::citizen`] that depends on egui.
//! A future Masonry / wgpu / web backend would replace this module wholesale
//! without touching [`crate::model`] or [`crate::lang`].
//!
//! ## Entry point
//!
//! [`paint_scene`] takes an `egui::Painter`, a `Scene`, and a `Viewport`
//! (world-to-screen transform). It paints every node, then every edge.
//!
//! ## Coordinate spaces
//!
//! - **World space** — the coordinate system of the `Scene`. Pixels at zoom = 1.
//!   Node positions live here.
//! - **Screen space** — what egui's painter consumes. The `Viewport` converts.

use egui::{Align2, Color32, CornerRadius, FontFamily, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

use crate::model::{
    ArrowHead, Border, CanvasBackground, Edge, EdgeId, EdgeOverlay, Fill, GridStyle, LineStyle,
    Node, NodeId, NodeKind, Routing, Scene, TextAnchor, TextLabel,
};
use crate::router::{edge_polyline, port_position_on_node};

// =============================================================================
// Viewport
// =============================================================================

/// Pan + zoom transformation between world and screen space.
///
/// `origin` is the screen-space pixel where world `(0, 0)` lands.
/// `zoom` is pixels per world unit (1.0 = native).
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub origin: Pos2,
    pub zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self { origin: Pos2::ZERO, zoom: 1.0 }
    }
}

impl Viewport {
    pub fn new(origin: Pos2, zoom: f32) -> Self {
        Self { origin, zoom }
    }

    pub fn world_to_screen(&self, world: (f32, f32)) -> Pos2 {
        Pos2::new(self.origin.x + world.0 * self.zoom, self.origin.y + world.1 * self.zoom)
    }

    pub fn screen_to_world(&self, screen: Pos2) -> (f32, f32) {
        ((screen.x - self.origin.x) / self.zoom, (screen.y - self.origin.y) / self.zoom)
    }

    /// Scale a world-space length to screen pixels.
    pub fn scale(&self, length: f32) -> f32 {
        length * self.zoom
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Paint every node and edge of a scene through the given viewport.
///
/// Layer order: nodes → edges → ports → waypoints. Edges paint after
/// nodes so connectors appear on top. The grid is painted separately by
/// the caller — CPU [`paint_grid`] or the GPU canvas shader.
pub fn paint_scene(painter: &Painter, scene: &Scene, viewport: &Viewport) {
    for node in &scene.nodes {
        paint_node(painter, node, viewport);
    }
    for edge in &scene.edges {
        paint_edge(painter, edge, scene, viewport);
    }
    paint_ports(painter, scene, viewport);
    paint_waypoints(painter, scene, viewport);
}

/// Draw pivot-vertex handles for hand-routed wires so they can be seen and
/// grabbed. Constant screen radius, like ports.
pub fn paint_waypoints(painter: &Painter, scene: &Scene, viewport: &Viewport) {
    let accent = Color32::from_rgb(0x25, 0x63, 0xEB);
    for edge in &scene.edges {
        if let Routing::Manual { waypoints } = &edge.routing {
            for &(wx, wy) in waypoints {
                let p = viewport.world_to_screen((wx, wy));
                painter.circle(p, 4.0, Color32::WHITE, Stroke::new(1.5, accent));
            }
        }
    }
}

/// Draw every port as a small handle so connection points are visible and
/// grabbable. Handle radius is constant in screen pixels — ports stay the
/// same grabbable size at any zoom. Filled colour encodes [`PortKind`].
pub fn paint_ports(painter: &Painter, scene: &Scene, viewport: &Viewport) {
    const R: f32 = 4.0;
    for node in &scene.nodes {
        for port in &node.ports {
            let p = viewport.world_to_screen(port_position_on_node(node, port));
            painter.circle(p, R, port_fill(port.kind), Stroke::new(1.0, Color32::from_gray(40)));
        }
    }
}

/// Dashed rubber-band preview for a connection being drawn, from a source
/// port to the current cursor position.
pub fn paint_connection_preview(
    painter: &Painter,
    from_world: (f32, f32),
    cursor_world: (f32, f32),
    viewport: &Viewport,
) {
    let a = viewport.world_to_screen(from_world);
    let b = viewport.world_to_screen(cursor_world);
    let accent = Color32::from_rgb(0x25, 0x63, 0xEB);
    paint_dashed_line(painter, a, b, Stroke::new(2.0, accent), 6.0, 4.0);
    painter.circle_filled(b, 4.0, accent);
}

/// Draw a highlight halo over each selected edge.
pub fn paint_selected_edges(painter: &Painter, scene: &Scene, selected: &[EdgeId], viewport: &Viewport) {
    let halo = Color32::from_rgba_unmultiplied(0x25, 0x63, 0xEB, 110);
    for id in selected {
        let Some(edge) = scene.edges.iter().find(|e| &e.id == id) else {
            continue;
        };
        let Some(poly) = edge_polyline(scene, edge) else {
            continue;
        };
        let pts: Vec<Pos2> = poly.iter().map(|w| viewport.world_to_screen(*w)).collect();
        for seg in pts.windows(2) {
            painter.line_segment([seg[0], seg[1]], Stroke::new(6.0, halo));
        }
    }
}

fn port_fill(kind: crate::model::PortKind) -> Color32 {
    use crate::model::PortKind;
    match kind {
        PortKind::In => Color32::from_rgb(0x25, 0x63, 0xEB),
        PortKind::Out => Color32::from_rgb(0x05, 0x96, 0x69),
        PortKind::Bidir => Color32::from_rgb(0x7C, 0x3A, 0xED),
        PortKind::Untyped => Color32::from_rgb(0x6B, 0x72, 0x80),
    }
}

/// Fill colour for a [`CanvasBackground`] preset.
pub fn background_color(bg: CanvasBackground) -> Color32 {
    match bg {
        CanvasBackground::Light => Color32::from_rgb(0xF8, 0xFA, 0xFC),
        CanvasBackground::Slate => Color32::from_rgb(0xDD, 0xE1, 0xE7),
        CanvasBackground::Charcoal => Color32::from_rgb(0x2B, 0x30, 0x3A),
        CanvasBackground::Dark => Color32::from_rgb(0x14, 0x17, 0x1C),
    }
}

/// Paint the grid using `settings.grid_style`. Auto-hides when zoom is so low
/// that the grid would just look like noise.
pub fn paint_grid(painter: &Painter, viewport: &Viewport, settings: &crate::model::CanvasSettings, screen_clip: Rect) {
    let world_spacing = settings.grid_spacing;
    if world_spacing <= 0.0 {
        return;
    }
    let screen_spacing = world_spacing * viewport.zoom;
    if screen_spacing < 4.0 {
        return;
    }

    let (wx0, wy0) = viewport.screen_to_world(screen_clip.min);
    let (wx1, wy1) = viewport.screen_to_world(screen_clip.max);
    let start_ix = (wx0 / world_spacing).floor() as i32;
    let end_ix = (wx1 / world_spacing).ceil() as i32;
    let start_iy = (wy0 / world_spacing).floor() as i32;
    let end_iy = (wy1 / world_spacing).ceil() as i32;

    // Grid ink contrasts with the background — dark ink on light canvases,
    // light ink on dark ones — so it stays visible either way.
    let ink = if settings.background.is_dark() { 255u8 } else { 0u8 };

    match settings.grid_style {
        GridStyle::Lines => {
            let minor = Stroke::new(1.0, Color32::from_rgba_unmultiplied(ink, ink, ink, 26));
            let major = Stroke::new(1.0, Color32::from_rgba_unmultiplied(ink, ink, ink, 60));
            for ix in start_ix..=end_ix {
                let x_screen = viewport.world_to_screen((ix as f32 * world_spacing, 0.0)).x;
                let stroke = if ix % 5 == 0 { major } else { minor };
                painter.line_segment(
                    [Pos2::new(x_screen, screen_clip.top()), Pos2::new(x_screen, screen_clip.bottom())],
                    stroke,
                );
            }
            for iy in start_iy..=end_iy {
                let y_screen = viewport.world_to_screen((0.0, iy as f32 * world_spacing)).y;
                let stroke = if iy % 5 == 0 { major } else { minor };
                painter.line_segment(
                    [Pos2::new(screen_clip.left(), y_screen), Pos2::new(screen_clip.right(), y_screen)],
                    stroke,
                );
            }
        }
        GridStyle::Dots => {
            let minor = Color32::from_rgba_unmultiplied(ink, ink, ink, 90);
            let major = Color32::from_rgba_unmultiplied(ink, ink, ink, 150);
            // Diameter in screen px; clamp so dots never disappear nor blob out.
            let radius = ((settings.dot_size * viewport.zoom) * 0.5).clamp(0.6, 6.0);
            for ix in start_ix..=end_ix {
                let x_screen = viewport.world_to_screen((ix as f32 * world_spacing, 0.0)).x;
                for iy in start_iy..=end_iy {
                    let y_screen = viewport.world_to_screen((0.0, iy as f32 * world_spacing)).y;
                    let color = if ix % 5 == 0 && iy % 5 == 0 { major } else { minor };
                    painter.circle_filled(Pos2::new(x_screen, y_screen), radius, color);
                }
            }
        }
    }
}

/// Axis-aligned bounding box of all nodes, in world coordinates. Returns
/// `None` if the scene has no nodes (nothing to fit).
pub fn scene_bounds(scene: &Scene) -> Option<Rect> {
    let mut iter = scene.nodes.iter();
    let first = iter.next()?;
    let mut min_x = first.transform.position.0;
    let mut min_y = first.transform.position.1;
    let mut max_x = first.transform.position.0 + first.transform.size.0;
    let mut max_y = first.transform.position.1 + first.transform.size.1;
    for node in iter {
        let (x, y) = node.transform.position;
        let (w, h) = node.transform.size;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + w);
        max_y = max_y.max(y + h);
    }
    Some(Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y)))
}

/// Build a viewport that fits `world_bounds` (with `padding_world` units of
/// margin) into `screen_rect`.
pub fn viewport_fit_to(world_bounds: Rect, screen_rect: Rect, padding_world: f32) -> Viewport {
    let padded = world_bounds.expand(padding_world);
    let zoom = (screen_rect.width() / padded.width())
        .min(screen_rect.height() / padded.height())
        .clamp(0.05, 20.0);
    // Place the world center at the screen center.
    let world_center = padded.center();
    let screen_center = screen_rect.center();
    let origin = Pos2::new(
        screen_center.x - world_center.x * zoom,
        screen_center.y - world_center.y * zoom,
    );
    Viewport { origin, zoom }
}

/// Draw a highlight outline around each selected node. Painted after
/// [`paint_scene`] so the highlight sits on top of node fills and edges.
pub fn paint_selection(painter: &Painter, scene: &Scene, selected: &[NodeId], viewport: &Viewport) {
    let accent = Color32::from_rgb(0x25, 0x63, 0xEB);
    let stroke = Stroke::new(2.0, accent);
    for id in selected {
        if let Some(node) = scene.nodes.iter().find(|n| &n.id == id) {
            let (x, y) = node.transform.position;
            let (w, h) = node.transform.size;
            let tl = viewport.world_to_screen((x, y));
            let br = viewport.world_to_screen((x + w, y + h));
            let rect = Rect::from_min_max(tl, br).expand(3.0);
            painter.rect_stroke(rect, CornerRadius::ZERO, stroke, StrokeKind::Outside);
        }
    }
}

// =============================================================================
// Color parsing
// =============================================================================

/// Parse `#RRGGBB` or `#RRGGBBAA` into a [`Color32`]. Returns transparent on
/// malformed input — the renderer prefers graceful degradation to panics.
pub(crate) fn parse_color(hex: &str) -> Color32 {
    let s = hex.trim_start_matches('#');
    let parse_byte = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    match s.len() {
        6 => match (parse_byte(0), parse_byte(2), parse_byte(4)) {
            (Some(r), Some(g), Some(b)) => Color32::from_rgb(r, g, b),
            _ => Color32::TRANSPARENT,
        },
        8 => match (parse_byte(0), parse_byte(2), parse_byte(4), parse_byte(6)) {
            (Some(r), Some(g), Some(b), Some(a)) => Color32::from_rgba_unmultiplied(r, g, b, a),
            _ => Color32::TRANSPARENT,
        },
        _ => Color32::TRANSPARENT,
    }
}

pub(crate) fn fill_to_color(fill: &Fill) -> Color32 {
    let base = parse_color(&fill.color);
    let alpha = (fill.alpha.clamp(0.0, 1.0) * 255.0) as u8;
    Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha)
}

fn stroke_for_border(border: &Border, viewport: &Viewport) -> Stroke {
    Stroke::new(border.width * viewport.zoom, parse_color(&border.color))
}

fn stroke_for_edge(overlay: &EdgeOverlay, viewport: &Viewport) -> Stroke {
    Stroke::new(overlay.width * viewport.zoom, parse_color(&overlay.color))
}

// =============================================================================
// Node painting
// =============================================================================

fn paint_node(painter: &Painter, node: &Node, viewport: &Viewport) {
    let (x, y) = node.transform.position;
    let (w, h) = node.transform.size;
    let top_left = viewport.world_to_screen((x, y));
    let bot_right = viewport.world_to_screen((x + w, y + h));
    let screen_rect = Rect::from_min_max(top_left, bot_right);

    let fill = fill_to_color(&node.overlay.fill);
    let stroke = stroke_for_border(&node.overlay.border, viewport);

    match &node.kind {
        NodeKind::Rect => {
            painter.rect(screen_rect, CornerRadius::ZERO, fill, stroke, StrokeKind::Inside);
        }
        NodeKind::Circle => {
            let center = screen_rect.center();
            let radius = screen_rect.width().min(screen_rect.height()) * 0.5;
            painter.circle(center, radius, fill, stroke);
        }
        NodeKind::Ellipse => {
            let center = screen_rect.center();
            let rx = screen_rect.width() * 0.5;
            let ry = screen_rect.height() * 0.5;
            paint_ellipse(painter, center, rx, ry, fill, stroke);
        }
        NodeKind::Path(_) | NodeKind::Group(_) => {
            // Not yet implemented — placeholder rect so the node is visible
            painter.rect(screen_rect, CornerRadius::ZERO, fill, stroke, StrokeKind::Inside);
        }
    }

    if let Some(text) = &node.overlay.text {
        paint_node_text(painter, text, screen_rect, viewport);
    }
}

/// Paint only the text labels of every node. The GPU path draws node
/// bodies on the GPU but keeps text on the egui painter.
pub fn paint_node_labels(painter: &Painter, scene: &Scene, viewport: &Viewport) {
    for node in &scene.nodes {
        let Some(text) = &node.overlay.text else {
            continue;
        };
        let (x, y) = node.transform.position;
        let (w, h) = node.transform.size;
        let top_left = viewport.world_to_screen((x, y));
        let bot_right = viewport.world_to_screen((x + w, y + h));
        let screen_rect = Rect::from_min_max(top_left, bot_right);
        paint_node_text(painter, text, screen_rect, viewport);
    }
}

fn paint_ellipse(painter: &Painter, center: Pos2, rx: f32, ry: f32, fill: Color32, stroke: Stroke) {
    // Polygon approximation. 64 segments looks smooth at any reasonable zoom.
    const SEGMENTS: usize = 64;
    let pts: Vec<Pos2> = (0..SEGMENTS)
        .map(|i| {
            let theta = (i as f32) / (SEGMENTS as f32) * std::f32::consts::TAU;
            Pos2::new(center.x + theta.cos() * rx, center.y + theta.sin() * ry)
        })
        .collect();
    painter.add(egui::Shape::convex_polygon(pts, fill, stroke));
}

fn paint_node_text(painter: &Painter, text: &TextLabel, screen_rect: Rect, viewport: &Viewport) {
    let pad = 4.0 * viewport.zoom;
    let (pos, align) = match text.anchor {
        TextAnchor::Center => (screen_rect.center(), Align2::CENTER_CENTER),
        TextAnchor::TopCenter => (Pos2::new(screen_rect.center().x, screen_rect.top() + pad), Align2::CENTER_TOP),
        TextAnchor::BottomCenter => (Pos2::new(screen_rect.center().x, screen_rect.bottom() - pad), Align2::CENTER_BOTTOM),
        TextAnchor::Left => (Pos2::new(screen_rect.left() + pad, screen_rect.center().y), Align2::LEFT_CENTER),
        TextAnchor::Right => (Pos2::new(screen_rect.right() - pad, screen_rect.center().y), Align2::RIGHT_CENTER),
        TextAnchor::TopLeft => (Pos2::new(screen_rect.left() + pad, screen_rect.top() + pad), Align2::LEFT_TOP),
        TextAnchor::TopRight => (Pos2::new(screen_rect.right() - pad, screen_rect.top() + pad), Align2::RIGHT_TOP),
        TextAnchor::BottomLeft => (Pos2::new(screen_rect.left() + pad, screen_rect.bottom() - pad), Align2::LEFT_BOTTOM),
        TextAnchor::BottomRight => (Pos2::new(screen_rect.right() - pad, screen_rect.bottom() - pad), Align2::RIGHT_BOTTOM),
    };

    // Bold/italic aren't expressible through FontFamily alone — egui resolves
    // those through font definitions. For now we accept the family-name as-is
    // and rely on the default proportional/monospace fallback chain.
    let family = if text.font_family.is_empty() {
        FontFamily::Proportional
    } else {
        // Fall back to proportional if the named family isn't registered.
        // (egui will use its default font; this avoids panics on unknown names.)
        FontFamily::Proportional
    };

    let font_id = FontId::new(text.font_size * viewport.zoom, family);
    let color = parse_color(&text.color);
    painter.text(pos, align, &text.value, font_id, color);
}

// =============================================================================
// Edge painting
// =============================================================================

/// Paint every edge of the scene. The GPU path that draws node bodies
/// on the GPU but not yet edges calls this directly.
pub fn paint_edges(painter: &Painter, scene: &Scene, viewport: &Viewport) {
    for edge in &scene.edges {
        paint_edge(painter, edge, scene, viewport);
    }
}

fn paint_edge(painter: &Painter, edge: &Edge, scene: &Scene, viewport: &Viewport) {
    // The router owns the path; the renderer only maps it to screen and draws.
    let Some(world) = edge_polyline(scene, edge) else {
        return;
    };
    if world.len() < 2 {
        return;
    }
    let screen: Vec<Pos2> = world.iter().map(|w| viewport.world_to_screen(*w)).collect();
    let stroke = stroke_for_edge(&edge.overlay, viewport);
    for seg in screen.windows(2) {
        paint_line(painter, seg[0], seg[1], stroke, edge.overlay.line_style);
    }
    let n = screen.len();
    paint_arrowhead(painter, edge, viewport, (screen[n - 2], screen[n - 1]));
}

fn paint_line(painter: &Painter, a: Pos2, b: Pos2, stroke: Stroke, style: LineStyle) {
    match style {
        LineStyle::Solid => {
            painter.line_segment([a, b], stroke);
        }
        LineStyle::Dashed => paint_dashed_line(painter, a, b, stroke, 8.0, 4.0),
        LineStyle::Dotted => paint_dashed_line(painter, a, b, stroke, 2.0, 3.0),
    }
}

fn paint_dashed_line(painter: &Painter, a: Pos2, b: Pos2, stroke: Stroke, dash: f32, gap: f32) {
    let delta = b - a;
    let total = delta.length();
    if total < 0.01 {
        return;
    }
    let dir = delta / total;
    let mut t = 0.0;
    while t < total {
        let start = a + dir * t;
        let end_t = (t + dash).min(total);
        let end = a + dir * end_t;
        painter.line_segment([start, end], stroke);
        t = end_t + gap;
    }
}

fn paint_arrowhead(painter: &Painter, edge: &Edge, viewport: &Viewport, last_segment: (Pos2, Pos2)) {
    if edge.overlay.arrow_head == ArrowHead::None {
        return;
    }
    let (seg_from, seg_to) = last_segment;
    let dir = seg_to - seg_from;
    if dir.length() < 0.01 {
        return;
    }
    let dir_n = dir.normalized();
    let perp = Vec2::new(-dir_n.y, dir_n.x);

    let head_len = 10.0 * viewport.zoom;
    let head_half_width = 5.0 * viewport.zoom;

    let base = seg_to - dir_n * head_len;
    let left = base + perp * head_half_width;
    let right = base - perp * head_half_width;

    let color = parse_color(&edge.overlay.color);

    match edge.overlay.arrow_head {
        ArrowHead::Arrow | ArrowHead::Triangle => {
            painter.add(egui::Shape::convex_polygon(vec![seg_to, left, right], color, Stroke::NONE));
        }
        ArrowHead::Circle => {
            painter.circle_filled(seg_to, head_half_width, color);
        }
        ArrowHead::Diamond => {
            let tail = seg_to - dir_n * (head_len * 2.0);
            let mid_left = base + perp * head_half_width;
            let mid_right = base - perp * head_half_width;
            painter.add(egui::Shape::convex_polygon(
                vec![seg_to, mid_left, tail, mid_right],
                color,
                Stroke::NONE,
            ));
        }
        ArrowHead::None => {}
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_roundtrip_identity() {
        let vp = Viewport::default();
        let p = vp.world_to_screen((10.0, 20.0));
        assert_eq!(p, Pos2::new(10.0, 20.0));
        let w = vp.screen_to_world(p);
        assert_eq!(w, (10.0, 20.0));
    }

    #[test]
    fn viewport_pan_and_zoom() {
        let vp = Viewport::new(Pos2::new(100.0, 50.0), 2.0);
        let p = vp.world_to_screen((10.0, 20.0));
        assert_eq!(p, Pos2::new(120.0, 90.0));
        let w = vp.screen_to_world(p);
        assert!((w.0 - 10.0).abs() < 1e-5);
        assert!((w.1 - 20.0).abs() < 1e-5);
    }

    #[test]
    fn parse_color_six_digit() {
        assert_eq!(parse_color("#FF8040"), Color32::from_rgb(255, 128, 64));
        assert_eq!(parse_color("FF8040"), Color32::from_rgb(255, 128, 64));
    }

    #[test]
    fn parse_color_eight_digit_with_alpha() {
        // Color32 stores premultiplied RGB internally; round-trip through
        // to_srgba_unmultiplied to recover the original input bytes.
        let c = parse_color("#FF804080");
        let [r, g, b, a] = c.to_srgba_unmultiplied();
        assert_eq!(r, 255);
        assert_eq!(g, 128);
        assert_eq!(b, 64);
        assert_eq!(a, 128);
    }

    #[test]
    fn parse_color_malformed_falls_back_to_transparent() {
        assert_eq!(parse_color("nope"), Color32::TRANSPARENT);
        assert_eq!(parse_color("#ZZZZZZ"), Color32::TRANSPARENT);
    }
}
