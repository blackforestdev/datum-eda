//! Stroke weight-class model (spec §4 "Weight-Class Policy").
//!
//! Resolves the "strokes thicken on zoom-in" defect and its companion latent
//! bug: a min-width clamp that floors in **nanometres** instead of device
//! pixels — a no-op, since 1 nm is invisible (spec §4, §4.3). Every drawable
//! primitive is assigned exactly one [`WeightClass`]; the width, floor, and
//! projection math live here so all surfaces resolve stroke width the same way.

/// The three stroke weight classes (spec §4.1).
///
/// Only [`WeightClass::ScreenConstant`] (class A) is zoom-invariant. Everything
/// that represents real document/fab geometry is class B or C and *must* scale
/// with the camera, so that what the viewport draws matches what fabrication
/// consumes (the `render == CAM` law).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeightClass {
    /// **Class A** — fixed device-pixel weight, resolved every frame against the
    /// live camera and never emitted into a retained world (nm) buffer. It never
    /// scales with zoom. Presentation chrome only: grid, selection, hover,
    /// cursor, marquee, snap glyphs. The grid-thickening bug is a class-A
    /// primitive that was mis-implemented as world geometry.
    ScreenConstant(f32),

    /// **Class B** — a true per-object world width in nanometres. It scales with
    /// zoom (physically correct), but its *projected* width is floored at
    /// `min_px` **device pixels** — the clamp is applied AFTER multiplying by the
    /// live scale (spec §4.3) — so a thin object never vanishes when zoomed out.
    WorldWidthWithMinClamp {
        /// Nominal world width, in nanometres.
        nominal_nm: i64,
        /// Minimum projected width, in device pixels, applied after projection.
        min_px: f32,
    },

    /// **Class C** — a house/importer nominal nm literal (e.g. a document-default
    /// wire width). It renders identically to class B; the B-vs-C split is a
    /// **provenance** distinction (user-owned width vs document-default width),
    /// not a render-behaviour one.
    AuthoredConstantNm {
        /// Nominal authored world width, in nanometres.
        nominal_nm: i64,
        /// Minimum projected width, in device pixels, applied after projection.
        min_px: f32,
    },
}

impl WeightClass {
    /// Resolve this class to a concrete stroke width in device pixels for the
    /// live camera, where `scale_px_per_nm` is the camera's device-pixels-per-
    /// nanometre.
    ///
    /// - Class A returns its fixed device-pixel width unchanged (zoom-invariant).
    /// - Classes B/C project the nominal nm width through the live scale and then
    ///   floor at `min_px`.
    ///
    /// The floor is deliberately applied **in device pixels, after** multiplying
    /// by the live scale: `(nominal_nm * scale_px_per_nm).max(min_px)`. This is
    /// the correctness point of spec §4.3 — flooring in nanometres (as the old
    /// `world_stroke_nm` did with `.max(1.0)` on a nm value) is a no-op, because
    /// 1 nm is invisible, so the intended min-width clamp never fired.
    pub fn resolve_px(&self, scale_px_per_nm: f32) -> f32 {
        match *self {
            WeightClass::ScreenConstant(px) => px,
            WeightClass::WorldWidthWithMinClamp { nominal_nm, min_px }
            | WeightClass::AuthoredConstantNm { nominal_nm, min_px } => {
                (nominal_nm as f32 * scale_px_per_nm).max(min_px)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Class A is the only zoom-invariant class: `ScreenConstant` returns its
    /// exact device-pixel width regardless of camera scale. This is the grid /
    /// chrome readability fix.
    #[test]
    fn screen_constant_is_zoom_invariant() {
        let w = WeightClass::ScreenConstant(1.0);
        // Two very different live scales; both must return the same 1.0 px.
        assert_eq!(w.resolve_px(1e-9), 1.0);
        assert_eq!(w.resolve_px(1.0), 1.0);
    }

    /// A 6-mil schematic wire (152_400 nm) scales with zoom and floors at exactly
    /// `min_px` device pixels when the projected width would fall below it. This
    /// proves the device-pixel floor fires — the bug fix.
    #[test]
    fn world_width_scales_and_floors_in_device_px() {
        let wire = WeightClass::WorldWidthWithMinClamp {
            nominal_nm: 152_400,
            min_px: 1.0,
        };

        // Bigger scale -> physically wider stroke.
        // 152_400 nm * 1e-4 px/nm = 15.24 px (well above the floor).
        assert!((wire.resolve_px(1e-4) - 15.24).abs() < 1e-3);

        // A smaller scale yields a smaller, but still un-floored, width.
        // 152_400 nm * 1e-5 px/nm = 1.524 px (> 1.0, no floor).
        assert!((wire.resolve_px(1e-5) - 1.524).abs() < 1e-3);

        // Zoom monotonicity: a bigger scale is a wider stroke.
        assert!(wire.resolve_px(1e-4) > wire.resolve_px(1e-5));

        // Tiny scale: projected width 152_400 * 1e-9 = 0.0001524 px, well below
        // the 1.0 px floor, so the floor fires and returns exactly 1.0.
        assert_eq!(wire.resolve_px(1e-9), 1.0);
    }

    /// Class C renders identically to the equivalent class B — the split is
    /// provenance-only, not a render-behaviour one.
    #[test]
    fn authored_constant_matches_world_width() {
        let b = WeightClass::WorldWidthWithMinClamp {
            nominal_nm: 152_400,
            min_px: 1.0,
        };
        let c = WeightClass::AuthoredConstantNm {
            nominal_nm: 152_400,
            min_px: 1.0,
        };
        for scale in [1e-9_f32, 1e-5, 1e-4, 1.0] {
            assert_eq!(b.resolve_px(scale), c.resolve_px(scale));
        }
    }
}
