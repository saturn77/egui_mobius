//! Bridge between egui_grafica's `f32` coordinate space and the hypercurve
//! exact-geometry kernel.
//!
//! egui_grafica's model, the `.canvas` DSL, mouse input, and the egui painter
//! all work in `f32`. hypercurve works in `hyperreal::Real` — exact
//! arithmetic. This module is the single conversion boundary.
//!
//! An `f32` *is* a dyadic rational, so `f32 → Real` is **lossless**: nothing
//! is approximated crossing into the kernel. `Real → f32` (for the painter
//! and for hit-testing) is the only lossy step, and it is confined to here.
//!
//! hypercurve is the geometry foundation: shapes and wire routes are built
//! and reasoned about as `Real`-backed hypercurve geometry; this module
//! projects to `f32` only at the rendering / interaction edge.

use hypercurve::{BulgeVertex2, Contour2, LineSeg2, Point2, Rational, Real, Segment2};

use crate::model::{Node, NodeKind};

/// Exact `f32` → `Real`. An `f32` is a dyadic rational, so this is lossless.
/// Non-finite inputs (`NaN`, `±∞`) map to zero.
pub fn real(v: f32) -> Real {
    match Rational::try_from(v) {
        Ok(r) => Real::new(r),
        Err(_) => Real::zero(),
    }
}

/// `Real` → `f32`, for rendering and hit-testing. Lossy — `f32` has finite
/// precision. Values that cannot be represented fall back to zero.
pub fn to_f32(r: &Real) -> f32 {
    r.to_f32_lossy().unwrap_or(0.0)
}

/// An `f32` coordinate pair as a hypercurve [`Point2`].
pub fn point(p: (f32, f32)) -> Point2 {
    Point2::new(real(p.0), real(p.1))
}

/// A hypercurve [`Point2`] as an `f32` coordinate pair.
pub fn point_xy(p: &Point2) -> (f32, f32) {
    (to_f32(p.x()), to_f32(p.y()))
}

/// A hypercurve [`LineSeg2`], or `None` if the endpoints coincide —
/// hypercurve rejects zero-length segments.
pub fn line_seg(a: (f32, f32), b: (f32, f32)) -> Option<LineSeg2> {
    LineSeg2::try_new(point(a), point(b)).ok()
}

/// Number of line segments used to approximate an ellipse contour —
/// hypercurve contours hold lines and circular arcs, not ellipse arcs.
const ELLIPSE_SEGMENTS: usize = 64;

/// A node's outline as a hypercurve [`Contour2`] — the exact shape geometry
/// the kernel reasons about (containment, intersection, area).
///
/// `Rect` is four line segments; `Circle` is two semicircular arcs (exact,
/// via bulge vertices); `Ellipse` is a polygon approximation since a
/// hypercurve contour has no ellipse-arc segment. `Path`/`Group` fall back
/// to the bounding rectangle, matching how the renderer draws them.
pub fn node_contour(node: &Node) -> Option<Contour2> {
    let (x, y) = node.transform.position;
    let (w, h) = node.transform.size;
    match node.kind {
        NodeKind::Circle => circle_contour(x, y, w, h),
        NodeKind::Ellipse => ellipse_contour(x, y, w, h),
        NodeKind::Rect | NodeKind::Path(_) | NodeKind::Group(_) => rect_contour(x, y, w, h),
    }
}

fn rect_contour(x: f32, y: f32, w: f32, h: f32) -> Option<Contour2> {
    let corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
    let mut segments = Vec::with_capacity(4);
    for i in 0..4 {
        segments.push(Segment2::Line(line_seg(corners[i], corners[(i + 1) % 4])?));
    }
    Contour2::try_new(segments).ok()
}

fn circle_contour(x: f32, y: f32, w: f32, h: f32) -> Option<Contour2> {
    let r = w.min(h) * 0.5;
    let (cx, cy) = (x + w * 0.5, y + h * 0.5);
    // Two bulge vertices, each with bulge 1.0 (a semicircular arc): the pair
    // closes into an exact circle.
    let verts = [
        BulgeVertex2::new(point((cx - r, cy)), real(1.0)),
        BulgeVertex2::new(point((cx + r, cy)), real(1.0)),
    ];
    Contour2::from_bulge_vertices(&verts).ok()
}

fn ellipse_contour(x: f32, y: f32, w: f32, h: f32) -> Option<Contour2> {
    let (cx, cy) = (x + w * 0.5, y + h * 0.5);
    let (rx, ry) = (w * 0.5, h * 0.5);
    let pts: Vec<(f32, f32)> = (0..ELLIPSE_SEGMENTS)
        .map(|i| {
            let t = i as f32 / ELLIPSE_SEGMENTS as f32 * std::f32::consts::TAU;
            (cx + t.cos() * rx, cy + t.sin() * ry)
        })
        .collect();
    let mut segments = Vec::with_capacity(ELLIPSE_SEGMENTS);
    for i in 0..ELLIPSE_SEGMENTS {
        segments.push(Segment2::Line(line_seg(pts[i], pts[(i + 1) % ELLIPSE_SEGMENTS])?));
    }
    Contour2::try_new(segments).ok()
}

/// True if `world` lies inside (or on the boundary of) the node's exact
/// contour. Falls back to `true` only if a contour cannot be built — the
/// caller is expected to have already done a cheap bounding-box pre-filter.
pub fn contour_contains(node: &Node, world: (f32, f32)) -> bool {
    use hypercurve::{Classification, ContourPointLocation, CurvePolicy};
    match node_contour(node) {
        Some(contour) => matches!(
            contour.classify_point(&point(world), &CurvePolicy::default()),
            Classification::Decided(ContourPointLocation::Inside | ContourPointLocation::Boundary)
        ),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_f32_round_trips_through_real() {
        // Every value here is exactly representable in f32, so f32 -> Real
        // (lossless) -> f32 must return the identical bits.
        for v in [0.0_f32, 1.0, -3.5, 120.0, 0.25, -0.0625, 1024.5, -2048.0] {
            assert_eq!(to_f32(&real(v)), v);
        }
    }

    #[test]
    fn non_finite_maps_to_zero() {
        assert_eq!(to_f32(&real(f32::NAN)), 0.0);
        assert_eq!(to_f32(&real(f32::INFINITY)), 0.0);
        assert_eq!(to_f32(&real(f32::NEG_INFINITY)), 0.0);
    }

    #[test]
    fn point_round_trips() {
        assert_eq!(point_xy(&point((12.5, -7.25))), (12.5, -7.25));
    }

    #[test]
    fn line_seg_rejects_zero_length() {
        assert!(line_seg((1.0, 1.0), (1.0, 1.0)).is_none());
        assert!(line_seg((0.0, 0.0), (10.0, 0.0)).is_some());
    }
}
