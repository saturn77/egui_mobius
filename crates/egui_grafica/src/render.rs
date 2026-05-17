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
    ArrowHead, Border, Edge, EdgeOverlay, Fill, GridStyle, LineStyle, Node, NodeId, NodeKind,
    Port, PortAnchor, PortId, Routing, Scene, TextAnchor, TextLabel,
};

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
/// Layer order: grid (if enabled in settings) → nodes → edges. Edges paint
/// after nodes so connectors appear on top — matches the visual convention of
/// every block-diagram tool.
pub fn paint_scene(painter: &Painter, scene: &Scene, viewport: &Viewport, screen_clip: Rect) {
    if scene.settings.show_grid {
        paint_grid(painter, viewport, &scene.settings, screen_clip);
    }
    for node in &scene.nodes {
        paint_node(painter, node, viewport);
    }
    for edge in &scene.edges {
        paint_edge(painter, edge, scene, viewport);
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

    match settings.grid_style {
        GridStyle::Lines => {
            let minor = Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 18));
            let major = Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 45));
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
            let minor = Color32::from_rgba_unmultiplied(0, 0, 0, 70);
            let major = Color32::from_rgba_unmultiplied(0, 0, 0, 130);
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

/// Look up a port's world-space position given its node+port ids.
///
/// Returns `None` if either id is unknown — useful for skipping orphaned edges
/// during in-progress edits.
pub fn port_world_position(scene: &Scene, node_id: &NodeId, port_id: &PortId) -> Option<(f32, f32)> {
    let node = scene.nodes.iter().find(|n| &n.id == node_id)?;
    let port = node.ports.iter().find(|p| &p.id == port_id)?;
    Some(port_position_on_node(node, port))
}

// =============================================================================
// Color parsing
// =============================================================================

/// Parse `#RRGGBB` or `#RRGGBBAA` into a [`Color32`]. Returns transparent on
/// malformed input — the renderer prefers graceful degradation to panics.
fn parse_color(hex: &str) -> Color32 {
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

fn fill_to_color(fill: &Fill) -> Color32 {
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

fn paint_edge(painter: &Painter, edge: &Edge, scene: &Scene, viewport: &Viewport) {
    let from_world = match port_world_position(scene, &edge.from.0, &edge.from.1) {
        Some(p) => p,
        None => return,
    };
    let to_world = match port_world_position(scene, &edge.to.0, &edge.to.1) {
        Some(p) => p,
        None => return,
    };

    let stroke = stroke_for_edge(&edge.overlay, viewport);
    let style = edge.overlay.line_style;

    let last_segment_end = match &edge.routing {
        Routing::Straight => {
            let p0 = viewport.world_to_screen(from_world);
            let p1 = viewport.world_to_screen(to_world);
            paint_line(painter, p0, p1, stroke, style);
            (p0, p1)
        }
        Routing::Orthogonal => paint_orthogonal(painter, from_world, to_world, viewport, stroke, style),
        Routing::Bezier => paint_bezier(painter, from_world, to_world, viewport, stroke, style),
        Routing::Manual(_) => {
            // Manual segment routing not yet implemented; fall back to orthogonal
            // so the edge at least respects right angles.
            paint_orthogonal(painter, from_world, to_world, viewport, stroke, style)
        }
    };

    paint_arrowhead(painter, edge, viewport, last_segment_end);
}

/// Three-segment H-V-H orthogonal route through the midpoint.
fn paint_orthogonal(
    painter: &Painter,
    from_world: (f32, f32),
    to_world: (f32, f32),
    viewport: &Viewport,
    stroke: Stroke,
    style: LineStyle,
) -> (Pos2, Pos2) {
    let p0 = viewport.world_to_screen(from_world);
    let p1 = viewport.world_to_screen(to_world);
    let mid_x = (p0.x + p1.x) * 0.5;
    let pa = Pos2::new(mid_x, p0.y);
    let pb = Pos2::new(mid_x, p1.y);
    paint_line(painter, p0, pa, stroke, style);
    paint_line(painter, pa, pb, stroke, style);
    paint_line(painter, pb, p1, stroke, style);
    (pb, p1)
}

/// Horizontal-out, horizontal-in cubic bezier — the natural shape for two
/// rectangular nodes connected east-to-west. Control-point offset scales with
/// horizontal distance so short edges curve gently and long edges flow.
fn paint_bezier(
    painter: &Painter,
    from_world: (f32, f32),
    to_world: (f32, f32),
    viewport: &Viewport,
    stroke: Stroke,
    style: LineStyle,
) -> (Pos2, Pos2) {
    let p0 = viewport.world_to_screen(from_world);
    let p3 = viewport.world_to_screen(to_world);
    let dx = (p3.x - p0.x).abs().max(40.0 * viewport.zoom);
    let handle = dx * 0.5;
    let p1 = Pos2::new(p0.x + handle, p0.y);
    let p2 = Pos2::new(p3.x - handle, p3.y);

    // De Casteljau evaluation into a polyline so we get dashed-style support for free.
    const SEGMENTS: usize = 32;
    let mut prev = p0;
    for i in 1..=SEGMENTS {
        let t = (i as f32) / (SEGMENTS as f32);
        let pt = cubic_bezier(p0, p1, p2, p3, t);
        paint_line(painter, prev, pt, stroke, style);
        prev = pt;
    }

    // Return the tangent direction at t=1 so the arrowhead points correctly.
    // Derivative of a cubic bezier at t=1: 3 * (P3 - P2).
    let tangent_start = Pos2::new(p3.x - 3.0 * (p3.x - p2.x), p3.y - 3.0 * (p3.y - p2.y));
    (tangent_start, p3)
}

fn cubic_bezier(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let u = 1.0 - t;
    let b0 = u * u * u;
    let b1 = 3.0 * u * u * t;
    let b2 = 3.0 * u * t * t;
    let b3 = t * t * t;
    Pos2::new(
        b0 * p0.x + b1 * p1.x + b2 * p2.x + b3 * p3.x,
        b0 * p0.y + b1 * p1.y + b2 * p2.y + b3 * p3.y,
    )
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
// Port position resolution
// =============================================================================

fn port_position_on_node(node: &Node, port: &Port) -> (f32, f32) {
    let (x, y) = node.transform.position;
    let (w, h) = node.transform.size;
    match port.anchor {
        PortAnchor::North(t) => (x + w * t, y),
        PortAnchor::South(t) => (x + w * t, y + h),
        PortAnchor::East(t) => (x + w, y + h * t),
        PortAnchor::West(t) => (x, y + h * t),
        PortAnchor::Free(fx, fy) => (x + w * fx, y + h * fy),
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
