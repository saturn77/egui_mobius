//! Scene → vector / raster export.
//!
//! The exporter generates an SVG document directly from the [`Scene`]
//! model. SVG is the canonical interchange format here:
//!
//! * [`export_svg`] writes the SVG out unchanged — always available,
//!   no extra deps.
//! * [`export_png`] / [`export_jpg`] rasterise the SVG with `resvg`
//!   (feature `export-raster`).
//! * [`export_pdf`] runs the SVG through `svg2pdf` (feature
//!   `export-pdf`).
//!
//! All three downstream formats share the same renderer here, so a
//! change to (say) edge stroke style propagates everywhere.

use std::fmt::Write as _;
use std::path::Path;

use crate::geometry::PARALLELOGRAM_SKEW_RATIO;
use crate::model::{
    ArrowHead, Edge, EdgeOverlay, LineStyle, Node, NodeKind, Overlay, Routing, Scene, TextAnchor,
    TextLabel,
};
use crate::page::{page_geometry, world_per_inch};
use crate::router::edge_polyline;

/// Page region preference. If the scene has a paper configured we
/// default to [`Region::Page`]; otherwise the content bounds are used
/// with a fixed margin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Region {
    /// Use the page board's full sheet rect (requires a paper size).
    Page,
    /// Use the tight bounding box of all nodes + edge endpoints,
    /// expanded by `padding_world` world units.
    Content { padding_world: f32 },
}

/// Knobs that affect every exporter.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Which world-space rect to draw.
    pub region: Region,
    /// Background fill: `None` ⇒ transparent (recommended for PDF /
    /// PNG-on-dark-themes), `Some` ⇒ paint this fill behind the
    /// document. The sheet white background is *separate* from this
    /// — it always paints when a page board is configured.
    pub background: Option<String>,
    /// Draw the grid behind the content, mirroring on-screen.
    pub include_grid: bool,
    /// Draw port markers (small filled circles at each port).
    pub include_ports: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            region: Region::Content { padding_world: 24.0 },
            background: None,
            include_grid: false,
            include_ports: true,
        }
    }
}

/// Render `scene` to an SVG document string.
///
/// The viewBox is set in world units so downstream consumers
/// (resvg / svg2pdf) can pick any pixel size or DPI without
/// rounding errors creeping in here.
pub fn scene_to_svg(scene: &Scene, opts: &ExportOptions) -> String {
    let (vx, vy, vw, vh) = compute_region(scene, opts);
    let mut out = String::with_capacity(4096);

    let _ = writeln!(
        out,
        r##"<?xml version="1.0" encoding="UTF-8"?>"##,
    );
    let _ = writeln!(
        out,
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vx} {vy} {vw} {vh}" width="{vw}" height="{vh}">"##,
    );

    if let Some(bg) = opts.background.as_deref() {
        let _ = writeln!(
            out,
            r##"  <rect x="{vx}" y="{vy}" width="{vw}" height="{vh}" fill="{bg}" />"##,
        );
    }

    if opts.include_grid && scene.settings.show_grid {
        write_grid(&mut out, scene, vx, vy, vw, vh);
    }

    write_page_board(&mut out, scene);

    for node in &scene.nodes {
        write_node(&mut out, node);
    }

    for edge in &scene.edges {
        write_edge(&mut out, scene, edge);
    }

    if opts.include_ports {
        write_ports(&mut out, scene);
    }

    let _ = writeln!(out, "</svg>");
    out
}

/// Write the SVG to a `.svg` file. The most basic exporter; no
/// extra dependencies.
pub fn export_svg(scene: &Scene, opts: &ExportOptions, path: &Path) -> std::io::Result<()> {
    let svg = scene_to_svg(scene, opts);
    std::fs::write(path, svg.as_bytes())
}

// =============================================================================
// Region + bounds
// =============================================================================

fn compute_region(scene: &Scene, opts: &ExportOptions) -> (f32, f32, f32, f32) {
    match opts.region {
        Region::Page => {
            if let Some(g) = page_geometry(&scene.settings) {
                return (g.origin.0, g.origin.1, g.paper.0, g.paper.1);
            }
            // Fallback: scene bounds if page wasn't configured.
            content_bounds(scene, 24.0)
        }
        Region::Content { padding_world } => content_bounds(scene, padding_world),
    }
}

fn content_bounds(scene: &Scene, padding: f32) -> (f32, f32, f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for n in &scene.nodes {
        let (x, y) = n.transform.position;
        let (w, h) = n.transform.size;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + w);
        max_y = max_y.max(y + h);
    }
    if min_x.is_infinite() {
        // Empty scene — yield a small default sheet.
        return (0.0, 0.0, 200.0, 150.0);
    }
    let w = (max_x - min_x) + 2.0 * padding;
    let h = (max_y - min_y) + 2.0 * padding;
    (min_x - padding, min_y - padding, w, h)
}

// =============================================================================
// SVG primitives
// =============================================================================

fn write_grid(out: &mut String, scene: &Scene, vx: f32, vy: f32, vw: f32, vh: f32) {
    let step = scene.settings.grid_spacing.max(1.0);
    let grid_color = if scene.settings.background.is_dark() {
        "#3F3F46"
    } else {
        "#E5E7EB"
    };
    let _ = writeln!(out, r##"  <g stroke="{grid_color}" stroke-width="0.5">"##);
    let mut x = (vx / step).floor() * step;
    while x <= vx + vw {
        let _ = writeln!(
            out,
            r##"    <line x1="{x}" y1="{vy}" x2="{x}" y2="{}" />"##,
            vy + vh
        );
        x += step;
    }
    let mut y = (vy / step).floor() * step;
    while y <= vy + vh {
        let _ = writeln!(
            out,
            r##"    <line x1="{vx}" y1="{y}" x2="{}" y2="{y}" />"##,
            vx + vw
        );
        y += step;
    }
    let _ = writeln!(out, "  </g>");
}

fn write_page_board(out: &mut String, scene: &Scene) {
    let Some(geom) = page_geometry(&scene.settings) else {
        return;
    };
    let (sx, sy) = geom.origin;
    let (sw, sh) = geom.paper;

    // Sheet body — paper white, with a thin grey edge.
    let _ = writeln!(
        out,
        r##"  <rect x="{sx}" y="{sy}" width="{sw}" height="{sh}" fill="#FFFFFF" stroke="#B0B0B0" stroke-width="0.4" />"##,
    );

    let (fx, fy, fw, fh) = geom.frame();
    let _ = writeln!(
        out,
        r##"  <rect x="{fx}" y="{fy}" width="{fw}" height="{fh}" fill="none" stroke="#000000" stroke-width="0.8" />"##,
    );

    // Zone markers (A–H × 6–1). Font size scales to the sheet so
    // small papers stay readable.
    let wpi = world_per_inch(scene.settings.grid_units);
    let zone_font = (0.12 * wpi).max(6.0);
    let zone_color = "#555555";

    for i in 0..8 {
        let letter = char::from(b'A' + i as u8);
        let cx = fx + (i as f32 + 0.5) * fw / 8.0;
        let _ = writeln!(
            out,
            r##"  <text x="{cx}" y="{}" font-size="{zone_font}" fill="{zone_color}" text-anchor="middle" dominant-baseline="alphabetic">{letter}</text>"##,
            fy - 0.04 * wpi
        );
        let _ = writeln!(
            out,
            r##"  <text x="{cx}" y="{}" font-size="{zone_font}" fill="{zone_color}" text-anchor="middle" dominant-baseline="hanging">{letter}</text>"##,
            fy + fh + 0.04 * wpi
        );
    }
    for i in 0..6 {
        let num = 6 - i;
        let cy = fy + (i as f32 + 0.5) * fh / 6.0;
        let _ = writeln!(
            out,
            r##"  <text x="{}" y="{cy}" font-size="{zone_font}" fill="{zone_color}" text-anchor="end" dominant-baseline="middle">{num}</text>"##,
            fx - 0.04 * wpi
        );
        let _ = writeln!(
            out,
            r##"  <text x="{}" y="{cy}" font-size="{zone_font}" fill="{zone_color}" text-anchor="start" dominant-baseline="middle">{num}</text>"##,
            fx + fw + 0.04 * wpi
        );
    }

    if let Some(tb) = scene.settings.title_block.as_ref()
        && tb.visible
    {
        write_title_block(out, geom.frame(), &scene.settings, tb);
    }
}

fn write_title_block(
    out: &mut String,
    (fx, fy, fw, fh): (f32, f32, f32, f32),
    settings: &crate::model::CanvasSettings,
    tb: &crate::page::TitleBlock,
) {
    let wpi = world_per_inch(settings.grid_units);
    let tb_w = (fw * 0.35).min(3.5 * wpi);
    let tb_h = (fh * 0.14).min(1.4 * wpi);
    let tb_x = fx + fw - tb_w;
    let tb_y = fy + fh - tb_h;

    let _ = writeln!(
        out,
        r##"  <g stroke="#000000" stroke-width="0.5" fill="none">"##,
    );
    let _ = writeln!(
        out,
        r##"    <rect x="{tb_x}" y="{tb_y}" width="{tb_w}" height="{tb_h}" fill="#FBFBFB" stroke-width="0.9" />"##,
    );

    let row_h = tb_h / 4.0;
    let label_w = tb_w * 0.24;
    let rev_w = tb_w * 0.18;

    for i in 1..4 {
        let y = tb_y + i as f32 * row_h;
        let _ = writeln!(
            out,
            r##"    <line x1="{tb_x}" y1="{y}" x2="{}" y2="{y}" />"##,
            tb_x + tb_w
        );
    }
    let label_x = tb_x + label_w;
    let _ = writeln!(
        out,
        r##"    <line x1="{label_x}" y1="{tb_y}" x2="{label_x}" y2="{}" />"##,
        tb_y + tb_h
    );
    let rev_x = tb_x + tb_w - rev_w;
    let _ = writeln!(
        out,
        r##"    <line x1="{rev_x}" y1="{}" x2="{rev_x}" y2="{}" />"##,
        tb_y + 2.0 * row_h,
        tb_y + tb_h
    );
    let _ = writeln!(out, "  </g>");

    // Fonts scale with row height; clamp so very small papers stay legible.
    let label_size = (row_h * 0.32).clamp(6.0, 11.0);
    let value_size = (row_h * 0.42).clamp(8.0, 14.0);
    let title_size = (row_h * 0.55).clamp(10.0, 20.0);
    let pad = (row_h * 0.18).clamp(1.5, 6.0);

    let row_text = |out: &mut String, x: f32, y: f32, sz: f32, color: &str, anchor: &str, txt: &str| {
        let _ = writeln!(
            out,
            r##"  <text x="{x}" y="{y}" font-size="{sz}" fill="{color}" text-anchor="{anchor}" dominant-baseline="middle">{}</text>"##,
            escape_xml(txt),
        );
    };

    // Row 0 — TITLE
    row_text(out, tb_x + pad, tb_y + row_h * 0.5, label_size, "#707070", "start", "TITLE");
    row_text(out, label_x + pad, tb_y + row_h * 0.5, title_size, "#000000", "start", &tb.title);
    // Row 1 — COMPANY
    row_text(out, tb_x + pad, tb_y + row_h * 1.5, label_size, "#707070", "start", "COMPANY");
    row_text(out, label_x + pad, tb_y + row_h * 1.5, value_size, "#000000", "start", &tb.company);
    // Row 2 — DWG NO | REV
    row_text(out, tb_x + pad, tb_y + row_h * 2.5, label_size, "#707070", "start", "DWG NO");
    row_text(out, label_x + pad, tb_y + row_h * 2.5, value_size, "#000000", "start", &tb.drawing_no);
    row_text(out, rev_x + pad, tb_y + row_h * 2.5, label_size, "#707070", "start", "REV");
    row_text(out, rev_x + rev_w * 0.5, tb_y + row_h * 2.5, value_size, "#000000", "middle", &tb.revision);
    // Row 3 — DATE | SHEET
    row_text(out, tb_x + pad, tb_y + row_h * 3.5, label_size, "#707070", "start", "DATE");
    row_text(out, label_x + pad, tb_y + row_h * 3.5, value_size, "#000000", "start", &tb.date);
    row_text(out, rev_x + pad, tb_y + row_h * 3.5, label_size, "#707070", "start", "SHEET");
    row_text(out, rev_x + rev_w * 0.5, tb_y + row_h * 3.5, value_size, "#000000", "middle", &tb.sheet);
}

fn write_node(out: &mut String, node: &Node) {
    let (x, y) = node.transform.position;
    let (w, h) = node.transform.size;
    let (fill, fill_opacity) = fill_attrs(&node.overlay);
    let (stroke, stroke_w, dash) = stroke_attrs(&node.overlay);
    let rot = node.transform.rotation;
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;

    // Group everything so the rotation transform applies uniformly.
    let _ = writeln!(
        out,
        r##"  <g transform="rotate({rot} {cx} {cy})">"##,
    );

    match &node.kind {
        NodeKind::Rect => {
            let _ = writeln!(
                out,
                r##"    <rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}" fill-opacity="{fill_opacity}" stroke="{stroke}" stroke-width="{stroke_w}"{dash} />"##,
            );
        }
        NodeKind::Circle | NodeKind::Ellipse => {
            let rx = w * 0.5;
            let ry = h * 0.5;
            let _ = writeln!(
                out,
                r##"    <ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{fill}" fill-opacity="{fill_opacity}" stroke="{stroke}" stroke-width="{stroke_w}"{dash} />"##,
            );
        }
        NodeKind::Parallelogram => {
            // Right-leaning skew, inscribed in (x, y, w, h).
            let skew = h * PARALLELOGRAM_SKEW_RATIO;
            let p = format!(
                "{},{} {},{} {},{} {},{}",
                x + skew, y,
                x + w, y,
                x + w - skew, y + h,
                x, y + h,
            );
            let _ = writeln!(
                out,
                r##"    <polygon points="{p}" fill="{fill}" fill-opacity="{fill_opacity}" stroke="{stroke}" stroke-width="{stroke_w}"{dash} />"##,
            );
        }
        NodeKind::Path(_) | NodeKind::Group(_) => {
            // Path / Group nodes not yet supported in export.
            let _ = writeln!(
                out,
                r##"    <rect x="{x}" y="{y}" width="{w}" height="{h}" fill="none" stroke="#888888" stroke-dasharray="4 3" />"##,
            );
        }
    }

    if let Some(text) = node.overlay.text.as_ref() {
        write_node_text(out, node, text);
    }
    let _ = writeln!(out, "  </g>");
}

fn write_node_text(out: &mut String, node: &Node, text: &TextLabel) {
    if text.value.is_empty() {
        return;
    }
    let (x, y) = node.transform.position;
    let (w, h) = node.transform.size;
    let (tx, ty, anchor, baseline) = match text.anchor {
        TextAnchor::Center => (x + w * 0.5, y + h * 0.5, "middle", "middle"),
        TextAnchor::TopCenter => (x + w * 0.5, y + text.font_size * 0.2, "middle", "hanging"),
        TextAnchor::BottomCenter => (x + w * 0.5, y + h - text.font_size * 0.2, "middle", "alphabetic"),
        TextAnchor::Left => (x + text.font_size * 0.2, y + h * 0.5, "start", "middle"),
        TextAnchor::Right => (x + w - text.font_size * 0.2, y + h * 0.5, "end", "middle"),
        TextAnchor::TopLeft => (x + text.font_size * 0.2, y + text.font_size * 0.2, "start", "hanging"),
        TextAnchor::TopRight => (x + w - text.font_size * 0.2, y + text.font_size * 0.2, "end", "hanging"),
        TextAnchor::BottomLeft => (x + text.font_size * 0.2, y + h - text.font_size * 0.2, "start", "alphabetic"),
        TextAnchor::BottomRight => (x + w - text.font_size * 0.2, y + h - text.font_size * 0.2, "end", "alphabetic"),
    };
    let weight = if text.bold { " font-weight=\"bold\"" } else { "" };
    let style = if text.italic { " font-style=\"italic\"" } else { "" };
    let family = if text.font_family.is_empty() {
        "sans-serif"
    } else {
        text.font_family.as_str()
    };
    let _ = writeln!(
        out,
        r##"    <text x="{tx}" y="{ty}" font-size="{}" font-family="{family}"{weight}{style} fill="{}" text-anchor="{anchor}" dominant-baseline="{baseline}">{}</text>"##,
        text.font_size,
        text.color,
        escape_xml(&text.value),
    );
}

fn write_edge(out: &mut String, scene: &Scene, edge: &Edge) {
    let Some(pts) = edge_polyline(scene, edge) else {
        return;
    };
    if pts.len() < 2 {
        return;
    }
    let (stroke, stroke_w, dash) = edge_stroke_attrs(&edge.overlay);

    let d = polyline_to_path_d(&pts, &edge.routing);
    let _ = writeln!(
        out,
        r##"  <path d="{d}" fill="none" stroke="{stroke}" stroke-width="{stroke_w}"{dash} />"##,
    );

    // Arrowheads — small triangles aligned with the relevant end segment.
    if !matches!(edge.overlay.arrow_head, ArrowHead::None) && pts.len() >= 2 {
        let n = pts.len();
        write_arrowhead(out, pts[n - 2], pts[n - 1], &edge.overlay);
    }
    if !matches!(edge.overlay.arrow_tail, ArrowHead::None) && pts.len() >= 2 {
        write_arrowhead(out, pts[1], pts[0], &edge.overlay);
    }
}

fn polyline_to_path_d(pts: &[(f32, f32)], routing: &Routing) -> String {
    let mut s = String::with_capacity(pts.len() * 16);
    let (x0, y0) = pts[0];
    let _ = write!(s, "M {x0} {y0}");
    if matches!(routing, Routing::Bezier) && pts.len() == 4 {
        let (x1, y1) = pts[1];
        let (x2, y2) = pts[2];
        let (x3, y3) = pts[3];
        let _ = write!(s, " C {x1} {y1} {x2} {y2} {x3} {y3}");
    } else {
        for &(x, y) in &pts[1..] {
            let _ = write!(s, " L {x} {y}");
        }
    }
    s
}

fn write_arrowhead(
    out: &mut String,
    from: (f32, f32),
    tip: (f32, f32),
    overlay: &EdgeOverlay,
) {
    let dx = tip.0 - from.0;
    let dy = tip.1 - from.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-3 {
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let arrow_len = 8.0;
    let arrow_w = 4.0;
    // Base point — `arrow_len` behind the tip along the segment.
    let bx = tip.0 - ux * arrow_len;
    let by = tip.1 - uy * arrow_len;
    // Perpendicular for the two wings.
    let px = -uy;
    let py = ux;
    let l = (bx + px * arrow_w, by + py * arrow_w);
    let r = (bx - px * arrow_w, by - py * arrow_w);
    let _ = writeln!(
        out,
        r##"  <polygon points="{},{} {},{} {},{}" fill="{}" />"##,
        tip.0, tip.1, l.0, l.1, r.0, r.1, overlay.color,
    );
}

fn write_ports(out: &mut String, scene: &Scene) {
    for node in &scene.nodes {
        for port in &node.ports {
            let Some((px, py)) = crate::router::port_world_position(scene, &node.id, &port.id)
            else {
                continue;
            };
            let _ = writeln!(
                out,
                r##"  <circle cx="{px}" cy="{py}" r="2.5" fill="#1F2937" />"##,
            );
        }
    }
}

// =============================================================================
// Style helpers
// =============================================================================

fn fill_attrs(overlay: &Overlay) -> (String, f32) {
    (overlay.fill.color.clone(), overlay.fill.alpha.clamp(0.0, 1.0))
}

fn stroke_attrs(overlay: &Overlay) -> (String, f32, String) {
    let dash = line_style_dasharray(overlay.border.style, overlay.border.width);
    (overlay.border.color.clone(), overlay.border.width, dash)
}

fn edge_stroke_attrs(overlay: &EdgeOverlay) -> (String, f32, String) {
    let dash = line_style_dasharray(overlay.line_style, overlay.width);
    (overlay.color.clone(), overlay.width, dash)
}

fn line_style_dasharray(style: LineStyle, width: f32) -> String {
    match style {
        LineStyle::Solid => String::new(),
        LineStyle::Dashed => format!(r##" stroke-dasharray="{} {}""##, width * 4.0, width * 3.0),
        LineStyle::Dotted => format!(r##" stroke-dasharray="{} {}""##, width, width * 2.0),
    }
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

// =============================================================================
// Raster + PDF exporters (feature-gated)
// =============================================================================

/// Rasterise the scene to a PNG at the requested DPI.
#[cfg(feature = "export-raster")]
pub fn export_png(
    scene: &Scene,
    opts: &ExportOptions,
    path: &Path,
    dpi: f32,
) -> std::io::Result<()> {
    let pixmap = rasterise(scene, opts, dpi)?;
    pixmap
        .save_png(path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}

/// Rasterise the scene and encode as JPEG at the requested DPI.
#[cfg(feature = "export-raster")]
pub fn export_jpg(
    scene: &Scene,
    opts: &ExportOptions,
    path: &Path,
    dpi: f32,
) -> std::io::Result<()> {
    let pixmap = rasterise(scene, opts, dpi)?;
    let w = pixmap.width();
    let h = pixmap.height();
    let buf = image::RgbaImage::from_raw(w, h, pixmap.take())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "pixmap → image failed"))?;
    let rgb = image::DynamicImage::ImageRgba8(buf).to_rgb8();
    rgb.save_with_format(path, image::ImageFormat::Jpeg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}

#[cfg(feature = "export-raster")]
fn rasterise(
    scene: &Scene,
    opts: &ExportOptions,
    dpi: f32,
) -> std::io::Result<resvg::tiny_skia::Pixmap> {
    let svg = scene_to_svg(scene, opts);
    let tree = resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    let (_, _, vw, vh) = compute_region(scene, opts);
    // World units → pixels: assume world is in inches when grid_units
    // is Inches, mils ⇒ /1000 in, mm ⇒ /25.4 in, pixels ⇒ /96 in.
    let inch_per_world = 1.0 / world_per_inch(scene.settings.grid_units);
    let pw = (vw * inch_per_world * dpi).max(1.0) as u32;
    let ph = (vh * inch_per_world * dpi).max(1.0) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(pw, ph).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "failed to allocate pixmap")
    })?;
    let sx = pw as f32 / vw;
    let sy = ph as f32 / vh;
    let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Ok(pixmap)
}

/// Export the scene to a single-page PDF via `svg2pdf`.
#[cfg(feature = "export-pdf")]
pub fn export_pdf(scene: &Scene, opts: &ExportOptions, path: &Path) -> std::io::Result<()> {
    let svg = scene_to_svg(scene, opts);
    let mut svg_opts = svg2pdf::usvg::Options::default();
    svg_opts.fontdb_mut().load_system_fonts();
    let tree = svg2pdf::usvg::Tree::from_str(&svg, &svg_opts)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    let pdf =
        svg2pdf::to_pdf(&tree, svg2pdf::ConversionOptions::default(), svg2pdf::PageOptions::default())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    std::fs::write(path, pdf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Border, CanvasBackground, CanvasSettings, Edge, EdgeEnd, EdgeId, EdgeOverlay, Fill,
        GridStyle, GridUnits, LineStyle, Node, NodeId, NodeKind, Overlay, Routing, Scene,
        TextAnchor, TextLabel, Transform,
    };

    fn rect_node(id: &str, pos: (f32, f32), size: (f32, f32)) -> Node {
        Node {
            id: NodeId(id.into()),
            kind: NodeKind::Rect,
            transform: Transform { position: pos, size, rotation: 0.0 },
            overlay: Overlay {
                border: Border { color: "#1F2937".into(), width: 1.5, style: LineStyle::Solid },
                fill: Fill { color: "#DBEAFE".into(), alpha: 0.9 },
                text: Some(TextLabel {
                    value: "hi".into(),
                    anchor: TextAnchor::Center,
                    font_family: String::new(),
                    font_size: 12.0,
                    bold: false,
                    italic: false,
                    color: "#111827".into(),
                }),
            },
            ports: vec![],
            style_ref: None,
        }
    }

    #[test]
    fn scene_to_svg_emits_a_well_formed_document_with_node_and_text() {
        let scene = Scene {
            nodes: vec![rect_node("a", (10.0, 20.0), (40.0, 30.0))],
            ..Default::default()
        };
        let svg = scene_to_svg(&scene, &ExportOptions::default());
        assert!(svg.starts_with("<?xml"), "must start with XML decl");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains(r##"<rect x="10" y="20" width="40" height="30""##));
        assert!(svg.contains(">hi<"));
    }

    #[test]
    fn page_region_uses_paper_bounds_when_configured() {
        let mut scene = Scene::default();
        scene.settings.paper_size = Some("Letter".into());
        scene.settings.paper_orientation = Some("portrait".into());
        scene.settings.grid_units = GridUnits::Inches;
        let opts = ExportOptions { region: Region::Page, ..Default::default() };
        let (x, y, w, h) = compute_region(&scene, &opts);
        assert_eq!((x, y), (0.0, 0.0));
        assert!((w - 8.5).abs() < 1e-3);
        assert!((h - 11.0).abs() < 1e-3);
    }

    #[test]
    fn content_region_pads_around_node_bounds() {
        let scene = Scene {
            nodes: vec![rect_node("a", (100.0, 200.0), (50.0, 40.0))],
            ..Default::default()
        };
        let (x, y, w, h) = compute_region(
            &scene,
            &ExportOptions { region: Region::Content { padding_world: 10.0 }, ..Default::default() },
        );
        assert!((x - 90.0).abs() < 1e-3);
        assert!((y - 190.0).abs() < 1e-3);
        assert!((w - 70.0).abs() < 1e-3);
        assert!((h - 60.0).abs() < 1e-3);
    }

    #[test]
    fn edge_polyline_round_trips_into_svg_path_d() {
        let mut scene = Scene::default();
        scene.nodes.push(rect_node("a", (0.0, 0.0), (40.0, 40.0)));
        scene.nodes.push(rect_node("b", (200.0, 100.0), (40.0, 40.0)));
        scene.edges.push(Edge {
            id: EdgeId("e".into()),
            from: EdgeEnd::Free(40.0, 20.0),
            to: EdgeEnd::Free(200.0, 120.0),
            routing: Routing::Straight,
            overlay: EdgeOverlay::default(),
        });
        let svg = scene_to_svg(&scene, &ExportOptions::default());
        assert!(svg.contains("<path d=\"M 40 20 L 200 120"));
    }

    #[test]
    fn export_svg_writes_a_file() {
        let scene = Scene {
            nodes: vec![rect_node("a", (0.0, 0.0), (40.0, 30.0))],
            ..Default::default()
        };
        let tmp = std::env::temp_dir().join("egui_grafica_export_test.svg");
        export_svg(&scene, &ExportOptions::default(), &tmp).unwrap();
        let written = std::fs::read_to_string(&tmp).unwrap();
        assert!(written.contains("<svg"));
        std::fs::remove_file(tmp).ok();
    }

    #[test]
    fn xml_escape_handles_special_chars() {
        assert_eq!(escape_xml("a & b < c > d \"e\" 'f'"), "a &amp; b &lt; c &gt; d &quot;e&quot; &apos;f&apos;");
    }
}
