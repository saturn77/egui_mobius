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

use hypercurve::{LineSeg2, Point2, Rational, Real};

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
