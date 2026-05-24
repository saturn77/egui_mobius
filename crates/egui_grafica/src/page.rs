//! Drafting-style page board: sized sheet, border frame with zone
//! markers (A–H × 1–6), and an optional title block in the bottom
//! right. This is the same engineering-drawing layout the simcore
//! Qt frontend uses; the patterns are ported here in egui idiom.
//!
//! World units interpretation: paper dimensions are intrinsic in
//! *inches*, then converted to the active world-unit base via the
//! scene's `grid_units` setting:
//!
//! * [`GridUnits::Inches`]      → 1 inch = 1 world unit
//! * [`GridUnits::Mils`]        → 1 inch = 1000 world units
//! * [`GridUnits::Millimeters`] → 1 inch = 25.4 world units
//! * [`GridUnits::Pixels`]      → 1 inch = 96 world units (CSS DPI)
//!
//! Painting happens *under* the scene content, so wires and nodes
//! sit on top of the sheet exactly the way they do in PCB CAD.

use egui::{Color32, FontFamily, FontId, Painter, Pos2, Rect, Stroke};
use serde::{Deserialize, Serialize};

use crate::model::{CanvasSettings, GridUnits};
use crate::render::Viewport;

/// Portrait dimensions of every named paper size we recognise, in
/// inches. Landscape is just the swapped pair, computed in
/// [`paper_dims_inches`].
pub const PAPER_SIZES_INCHES: &[(&str, (f32, f32))] = &[
    ("Letter", (8.5, 11.0)),
    ("Legal", (8.5, 14.0)),
    ("Tabloid", (11.0, 17.0)),
    ("A5", (5.83, 8.27)),
    ("A4", (8.27, 11.69)),
    ("A3", (11.69, 16.54)),
    ("ANSI C", (17.0, 22.0)),
    ("ANSI D", (22.0, 34.0)),
];

/// Engineering-drawing title block. Field set mirrors the simcore
/// Qt frontend so future tooling that round-trips both layouts
/// doesn't have to translate field names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TitleBlock {
    pub visible: bool,
    pub title: String,
    pub company: String,
    pub drawing_no: String,
    pub revision: String,
    /// ISO-style date string. Empty means "fill in `today` at render time".
    pub date: String,
    pub drawn_by: String,
    /// Free text — typically `"1 of 1"`.
    pub sheet: String,
}

impl Default for TitleBlock {
    fn default() -> Self {
        Self {
            visible: true,
            title: "Untitled".into(),
            company: "Company Name".into(),
            drawing_no: "DWG-001".into(),
            revision: "A".into(),
            date: String::new(),
            drawn_by: String::new(),
            sheet: "1 of 1".into(),
        }
    }
}

/// World-units value of one inch given the scene's grid_units.
pub fn world_per_inch(units: GridUnits) -> f32 {
    match units {
        GridUnits::Inches => 1.0,
        GridUnits::Mils => 1000.0,
        GridUnits::Millimeters => 25.4,
        GridUnits::Pixels => 96.0,
    }
}

/// Look up a named paper size and apply orientation; returns
/// `(width_in, height_in)` with landscape applied as a swap.
pub fn paper_dims_inches(name: &str, orientation: Option<&str>) -> Option<(f32, f32)> {
    let (w, h) = PAPER_SIZES_INCHES
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, d)| *d)?;
    let landscape = matches!(orientation, Some(o) if o.eq_ignore_ascii_case("landscape"));
    Some(if landscape { (h, w) } else { (w, h) })
}

/// Paper dimensions in *world units*, derived from the scene's
/// `paper_size`, `paper_orientation`, and `grid_units`. Returns
/// `None` if no paper is configured — paper is fully optional.
pub fn paper_dims_world(settings: &CanvasSettings) -> Option<(f32, f32)> {
    let name = settings.paper_size.as_deref()?;
    let (win, hin) = paper_dims_inches(name, settings.paper_orientation.as_deref())?;
    let k = world_per_inch(settings.grid_units);
    Some((win * k, hin * k))
}

/// Geometry of a laid-out page in world units.
#[derive(Debug, Clone, Copy)]
pub struct PageGeometry {
    /// Top-left of the sheet in world coords.
    pub origin: (f32, f32),
    /// Sheet size in world units.
    pub paper: (f32, f32),
    /// Border margin in world units (inset on every side).
    pub margin: f32,
}

impl PageGeometry {
    /// World-space rect of the framed drawing area (inside the margin).
    pub fn frame(&self) -> (f32, f32, f32, f32) {
        let (x, y) = self.origin;
        let (w, h) = self.paper;
        (x + self.margin, y + self.margin, w - 2.0 * self.margin, h - 2.0 * self.margin)
    }

    /// World-space rect of the paper sheet itself.
    pub fn sheet(&self) -> (f32, f32, f32, f32) {
        (self.origin.0, self.origin.1, self.paper.0, self.paper.1)
    }
}

/// Compute the page geometry for the current settings. Sheet is
/// anchored at world origin so the document is reproducible — moving
/// the viewport doesn't move the paper.
///
/// Returns `None` if no paper size is configured.
pub fn page_geometry(settings: &CanvasSettings) -> Option<PageGeometry> {
    let paper = paper_dims_world(settings)?;
    // 0.5 inch drawing margin — same default simcore uses.
    let margin = 0.5 * world_per_inch(settings.grid_units);
    Some(PageGeometry {
        origin: (0.0, 0.0),
        paper,
        margin,
    })
}

/// Paint the page board: drawing border with zone markers and the
/// title block when visible. The sheet outline is drawn but **not**
/// filled, so content underneath remains visible — the engineering
/// frame sits *on top* of the wires and nodes, not behind them.
/// No-op if no paper size is configured.
pub fn paint_page(painter: &Painter, viewport: &Viewport, settings: &CanvasSettings) {
    let Some(geom) = page_geometry(settings) else {
        return;
    };
    let dark = settings.background.is_dark();

    paint_sheet_outline(painter, viewport, &geom, dark);
    paint_border_and_zones(painter, viewport, &geom, dark);

    if let Some(tb) = settings.title_block.as_ref()
        && tb.visible
    {
        paint_title_block(painter, viewport, &geom, settings, tb);
    }
}

/// Sheet outline only — a thin grey rectangle at the paper edge so
/// the user knows where the page is, with no fill so nodes/wires
/// drawn earlier stay visible.
fn paint_sheet_outline(painter: &Painter, viewport: &Viewport, geom: &PageGeometry, dark: bool) {
    let (sx, sy, sw, sh) = geom.sheet();
    let tl = viewport.world_to_screen((sx, sy));
    let br = viewport.world_to_screen((sx + sw, sy + sh));
    let rect = Rect::from_two_pos(tl, br);

    let edge = if dark {
        Color32::from_rgb(0x60, 0x60, 0x60)
    } else {
        Color32::from_rgb(0xB0, 0xB0, 0xB0)
    };
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, edge), egui::StrokeKind::Inside);
}

fn paint_border_and_zones(
    painter: &Painter,
    viewport: &Viewport,
    geom: &PageGeometry,
    _dark: bool,
) {
    let (fx, fy, fw, fh) = geom.frame();
    let tl = viewport.world_to_screen((fx, fy));
    let br = viewport.world_to_screen((fx + fw, fy + fh));
    let frame_rect = Rect::from_two_pos(tl, br);

    // Frame outline — always black on the paper so it prints as
    // expected even from a dark editing background.
    painter.rect_stroke(
        frame_rect,
        0.0,
        Stroke::new(2.0, Color32::BLACK),
        egui::StrokeKind::Inside,
    );

    // Zone markers: A..H horizontal (8 columns), 1..6 vertical.
    // The numbering goes top-to-bottom = 6..1 so cell A1 is the
    // bottom-left corner — engineering convention.
    let zone_color = Color32::from_rgb(0x55, 0x55, 0x55);
    let zone_font = FontId::new(11.0, FontFamily::Proportional);

    let zones_h = 8usize;
    let zones_v = 6usize;
    let cell_w = frame_rect.width() / zones_h as f32;
    let cell_h = frame_rect.height() / zones_v as f32;

    for i in 0..zones_h {
        let letter = char::from(b'A' + i as u8);
        let cx = frame_rect.min.x + i as f32 * cell_w + cell_w * 0.5;
        painter.text(
            Pos2::new(cx, frame_rect.min.y - 8.0),
            egui::Align2::CENTER_BOTTOM,
            letter,
            zone_font.clone(),
            zone_color,
        );
        painter.text(
            Pos2::new(cx, frame_rect.max.y + 8.0),
            egui::Align2::CENTER_TOP,
            letter,
            zone_font.clone(),
            zone_color,
        );
        // Tick marks at zone boundaries.
        if i > 0 {
            let x = frame_rect.min.x + i as f32 * cell_w;
            painter.line_segment(
                [Pos2::new(x, frame_rect.min.y - 4.0), Pos2::new(x, frame_rect.min.y)],
                Stroke::new(1.0, zone_color),
            );
            painter.line_segment(
                [Pos2::new(x, frame_rect.max.y), Pos2::new(x, frame_rect.max.y + 4.0)],
                Stroke::new(1.0, zone_color),
            );
        }
    }

    for i in 0..zones_v {
        let num = zones_v - i; // bottom-up numbering
        let cy = frame_rect.min.y + i as f32 * cell_h + cell_h * 0.5;
        painter.text(
            Pos2::new(frame_rect.min.x - 8.0, cy),
            egui::Align2::RIGHT_CENTER,
            num,
            zone_font.clone(),
            zone_color,
        );
        painter.text(
            Pos2::new(frame_rect.max.x + 8.0, cy),
            egui::Align2::LEFT_CENTER,
            num,
            zone_font.clone(),
            zone_color,
        );
        if i > 0 {
            let y = frame_rect.min.y + i as f32 * cell_h;
            painter.line_segment(
                [Pos2::new(frame_rect.min.x - 4.0, y), Pos2::new(frame_rect.min.x, y)],
                Stroke::new(1.0, zone_color),
            );
            painter.line_segment(
                [Pos2::new(frame_rect.max.x, y), Pos2::new(frame_rect.max.x + 4.0, y)],
                Stroke::new(1.0, zone_color),
            );
        }
    }
}

fn paint_title_block(
    painter: &Painter,
    viewport: &Viewport,
    geom: &PageGeometry,
    settings: &CanvasSettings,
    tb: &TitleBlock,
) {
    let (fx, fy, fw, fh) = geom.frame();
    // Block sized proportionally to the sheet, capped at 3.5 × 1.4
    // *inches* so it stays readable but doesn't dominate small papers.
    let wpi = world_per_inch(settings.grid_units);
    let tb_w = (fw * 0.35).min(3.5 * wpi);
    let tb_h = (fh * 0.14).min(1.4 * wpi);
    let tb_x = fx + fw - tb_w;
    let tb_y = fy + fh - tb_h;

    let tl = viewport.world_to_screen((tb_x, tb_y));
    let br = viewport.world_to_screen((tb_x + tb_w, tb_y + tb_h));
    let rect = Rect::from_two_pos(tl, br);

    // Semi-transparent backdrop so the title-block text stays readable
    // when it sits over wires / nodes, without fully hiding them.
    painter.rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(0xFB, 0xFB, 0xFB, 230));
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.5, Color32::BLACK),
        egui::StrokeKind::Inside,
    );

    // 4 rows × the label/value split. Row heights split evenly.
    let row_h = rect.height() / 4.0;
    let label_w = rect.width() * 0.24;

    let thin = Stroke::new(0.8, Color32::BLACK);
    for i in 1..4 {
        let y = rect.min.y + i as f32 * row_h;
        painter.line_segment([Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)], thin);
    }
    // Label column.
    let label_x = rect.min.x + label_w;
    painter.line_segment([Pos2::new(label_x, rect.min.y), Pos2::new(label_x, rect.max.y)], thin);
    // Rev / Sheet split column on rows 2 + 3.
    let rev_w = rect.width() * 0.18;
    let rev_x = rect.max.x - rev_w;
    painter.line_segment(
        [
            Pos2::new(rev_x, rect.min.y + 2.0 * row_h),
            Pos2::new(rev_x, rect.max.y),
        ],
        thin,
    );

    // Fonts scale with row height so the block reads at any zoom.
    let label_size = (row_h * 0.32).clamp(7.0, 11.0);
    let value_size = (row_h * 0.42).clamp(9.0, 14.0);
    let title_size = (row_h * 0.55).clamp(11.0, 20.0);

    let label_font = FontId::new(label_size, FontFamily::Proportional);
    let value = FontId::new(value_size, FontFamily::Proportional);
    let title = FontId::new(title_size, FontFamily::Proportional);
    let label_color = Color32::from_rgb(0x70, 0x70, 0x70);
    let value_color = Color32::BLACK;

    let pad = (row_h * 0.06).clamp(2.0, 6.0);

    // Row 0 — TITLE
    painter.text(
        Pos2::new(rect.min.x + pad, rect.min.y + pad),
        egui::Align2::LEFT_TOP,
        "TITLE",
        label_font.clone(),
        label_color,
    );
    painter.text(
        Pos2::new(label_x + pad, rect.min.y + pad),
        egui::Align2::LEFT_TOP,
        &tb.title,
        title,
        value_color,
    );

    // Row 1 — COMPANY
    painter.text(
        Pos2::new(rect.min.x + pad, rect.min.y + row_h + pad),
        egui::Align2::LEFT_TOP,
        "COMPANY",
        label_font.clone(),
        label_color,
    );
    painter.text(
        Pos2::new(label_x + pad, rect.min.y + row_h + pad),
        egui::Align2::LEFT_TOP,
        &tb.company,
        value.clone(),
        value_color,
    );

    // Row 2 — DWG NO | REV
    painter.text(
        Pos2::new(rect.min.x + pad, rect.min.y + 2.0 * row_h + pad),
        egui::Align2::LEFT_TOP,
        "DWG NO",
        label_font.clone(),
        label_color,
    );
    painter.text(
        Pos2::new(label_x + pad, rect.min.y + 2.0 * row_h + pad),
        egui::Align2::LEFT_TOP,
        &tb.drawing_no,
        value.clone(),
        value_color,
    );
    painter.text(
        Pos2::new(rev_x + pad, rect.min.y + 2.0 * row_h + pad),
        egui::Align2::LEFT_TOP,
        "REV",
        label_font.clone(),
        label_color,
    );
    painter.text(
        Pos2::new(rev_x + rev_w * 0.5, rect.min.y + 2.0 * row_h + row_h * 0.5),
        egui::Align2::CENTER_CENTER,
        &tb.revision,
        value.clone(),
        value_color,
    );

    // Row 3 — DATE | SHEET
    let date_str = if tb.date.is_empty() {
        // Cheap "today" without pulling chrono into the core crate.
        // Renders an empty cell at minimum; downstream apps can fill
        // a value in if they want a clock dependency.
        String::new()
    } else {
        tb.date.clone()
    };
    painter.text(
        Pos2::new(rect.min.x + pad, rect.min.y + 3.0 * row_h + pad),
        egui::Align2::LEFT_TOP,
        "DATE",
        label_font.clone(),
        label_color,
    );
    painter.text(
        Pos2::new(label_x + pad, rect.min.y + 3.0 * row_h + pad),
        egui::Align2::LEFT_TOP,
        &date_str,
        value.clone(),
        value_color,
    );
    painter.text(
        Pos2::new(rev_x + pad, rect.min.y + 3.0 * row_h + pad),
        egui::Align2::LEFT_TOP,
        "SHEET",
        label_font.clone(),
        label_color,
    );
    painter.text(
        Pos2::new(rev_x + rev_w * 0.5, rect.min.y + 3.0 * row_h + row_h * 0.7),
        egui::Align2::CENTER_CENTER,
        &tb.sheet,
        value,
        value_color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CanvasSettings, GridUnits};

    #[test]
    fn paper_dims_inches_recognises_letter_portrait_and_landscape() {
        assert_eq!(paper_dims_inches("Letter", Some("portrait")), Some((8.5, 11.0)));
        assert_eq!(paper_dims_inches("Letter", Some("landscape")), Some((11.0, 8.5)));
        assert_eq!(paper_dims_inches("letter", None), Some((8.5, 11.0)));
        assert!(paper_dims_inches("Unknown", None).is_none());
    }

    #[test]
    fn paper_dims_inches_recognises_ansi_c_and_d() {
        assert_eq!(paper_dims_inches("ANSI C", None), Some((17.0, 22.0)));
        assert_eq!(paper_dims_inches("ANSI D", None), Some((22.0, 34.0)));
    }

    #[test]
    fn world_per_inch_matches_unit_conventions() {
        assert_eq!(world_per_inch(GridUnits::Inches), 1.0);
        assert_eq!(world_per_inch(GridUnits::Mils), 1000.0);
        assert!((world_per_inch(GridUnits::Millimeters) - 25.4).abs() < 1e-3);
        assert_eq!(world_per_inch(GridUnits::Pixels), 96.0);
    }

    #[test]
    fn paper_dims_world_scales_with_grid_units() {
        let mut s = CanvasSettings {
            paper_size: Some("Letter".into()),
            paper_orientation: Some("landscape".into()),
            grid_units: GridUnits::Mils,
            ..Default::default()
        };
        assert_eq!(paper_dims_world(&s), Some((11000.0, 8500.0)));
        s.grid_units = GridUnits::Inches;
        assert_eq!(paper_dims_world(&s), Some((11.0, 8.5)));
        s.paper_size = None;
        assert!(paper_dims_world(&s).is_none());
    }

    #[test]
    fn page_geometry_inset_matches_half_inch_margin() {
        let s = CanvasSettings {
            paper_size: Some("Letter".into()),
            grid_units: GridUnits::Inches,
            ..Default::default()
        };
        let geom = page_geometry(&s).unwrap();
        assert_eq!(geom.paper, (8.5, 11.0));
        assert_eq!(geom.margin, 0.5);
        let (fx, fy, fw, fh) = geom.frame();
        assert!((fx - 0.5).abs() < 1e-6);
        assert!((fy - 0.5).abs() < 1e-6);
        assert!((fw - 7.5).abs() < 1e-6);
        assert!((fh - 10.0).abs() < 1e-6);
    }
}
