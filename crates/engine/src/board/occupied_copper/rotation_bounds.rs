//! Exact containment certificates, not a replacement pad boundary or tolerance.
use super::*;

pub(super) struct PadBounds {
    inner: RoundedBox,
    outer: RoundedBox,
    exact: bool,
}

pub(super) fn pad_bounds(pad: &PlacedPad) -> Result<PadBounds> {
    if pad.shape == PadShape::Circle || pad.rotation.rem_euclid(90) == 0 {
        let shape = RoundedBox::pad(pad)?;
        return Ok(PadBounds {
            inner: shape,
            outer: shape,
            exact: true,
        });
    }
    // Validate authored dimensions, hole containment and corner ratio through
    // the same owner. This copy validates intrinsic shape only; its zero angle
    // is NEVER returned as the pad, used as projection, or used as a boundary.
    let mut intrinsic = pad.clone();
    intrinsic.rotation = 0;
    let validated = RoundedBox::pad(&intrinsic)?;
    let center = validated.center;
    let hole = validated.hole;
    // Every supported shape contains the centered min(width,height)/2 disk and
    // is contained in its width×height rectangle. Rotations preserve those
    // centered disks. Euclidean corner distance <= (width+height)/2 exactly.
    // Subtract the SAME centered drill hole from each set: inner ⊆ pad ⊆ outer.
    // Odd widths keep their rational halves; no output grid or epsilon enters.
    let inner_radius = R::new(i128::from(pad.width.min(pad.height)), 2)?;
    let outer_radius = R::new(i128::from(pad.width) + i128::from(pad.height), 2)?;
    let circle = |radius| RoundedBox {
        min: center,
        max: center,
        center,
        radius,
        hole,
    };
    Ok(PadBounds {
        inner: circle(inner_radius),
        outer: circle(outer_radius),
        exact: false,
    })
}

pub(super) fn certify_pad(
    pad: &PlacedPad,
    test: impl Fn(RoundedBox) -> Result<bool>,
) -> Result<bool> {
    let bounds = pad_bounds(pad)?;
    let inner = test(bounds.inner)?;
    if inner || bounds.exact {
        return Ok(inner);
    }
    if !test(bounds.outer)? {
        return Ok(false);
    }
    Err(GeometryError::UnresolvedPredicate)
}

pub(super) fn certify_pads(
    a: &PlacedPad,
    b: &PlacedPad,
    test: impl Fn(RoundedBox, RoundedBox) -> Result<bool>,
) -> Result<bool> {
    let a = pad_bounds(a)?;
    let b = pad_bounds(b)?;
    let inner = test(a.inner, b.inner)?;
    if inner || (a.exact && b.exact) {
        return Ok(inner);
    }
    if !test(a.outer, b.outer)? {
        return Ok(false);
    }
    Err(GeometryError::UnresolvedPredicate)
}

#[cfg(test)]
#[path = "rotation_bounds_tests.rs"]
mod tests;
