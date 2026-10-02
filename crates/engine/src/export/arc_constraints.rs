//! Conservative, source-bound manufacturing diagnostics; no policy waiver.
use super::arc_witness::{ArcOutputWitness, ArcWitnessError, arc_output_witness};
use crate::board::{
    PlacedPad, Stackup, Track, Via,
    nominal_geometry::{DistanceBoundary, GeometryError, Rational as R},
    occupied_copper::{
        conductive_layer, pad_layers, track_pad_within, track_via_within, via_layers,
    },
    track_contact::tracks_within,
};
use std::cmp::Ordering;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMarginResult {
    Preserved,
    AtRisk,
    Unavailable,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcOutputConstraintReport {
    pub source_id: Uuid,
    pub minimum_width: OutputMarginResult,
    /// Only the supplied source-qualified targets, never an entire Board claim.
    pub pairs: Vec<ArcOutputPairCheck>,
    pub requires_model_bound_fill_and_complete_inventory: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcOutputPairCheck {
    pub other_source_id: Uuid,
    pub nominal_contact: bool,
    pub nominal_clearance_violation: bool,
    pub clearance: OutputMarginResult,
    pub topology: OutputMarginResult,
}
/// Existing geometry owner supplies nominal checks. The integer expansion is a
/// conservative CEILING of computed deviation, not a manufacturing tolerance.
pub struct ArcOutputConstraintContext<'a> {
    pub stackup: &'a Stackup,
    pub minimum_width_nm: i64,
    pub clearance_nm: i64,
    pub tracks: &'a [Track],
    pub pads: &'a [PlacedPad],
    pub vias: &'a [Via],
}
pub fn check_arc_output_constraints(
    witness: &ArcOutputWitness,
    context: ArcOutputConstraintContext<'_>,
) -> Result<ArcOutputConstraintReport, ArcWitnessError> {
    let ArcOutputConstraintContext {
        stackup,
        minimum_width_nm,
        clearance_nm,
        tracks,
        pads,
        vias,
    } = context;
    conductive_layer(stackup, witness.source().layer)?;
    if minimum_width_nm < 0 || clearance_nm < 0 {
        return Err(ArcWitnessError::Geometry(GeometryError::InvalidWidth));
    }
    let source = witness.source();
    let deviation = witness.occupied_deviation_upper_nm();
    let minimum_width = match deviation {
        Some(e) => {
            let guaranteed =
                R::integer(i128::from(source.width)).checked_sub(e.checked_mul(R::integer(2))?)?;
            if guaranteed.compare(R::integer(i128::from(minimum_width_nm)))? == Ordering::Less {
                OutputMarginResult::AtRisk
            } else {
                OutputMarginResult::Preserved
            }
        }
        None => OutputMarginResult::Unavailable,
    };
    let mut pairs = Vec::new();
    for other in tracks {
        if other.uuid == source.uuid {
            if other != source {
                return Err(ArcWitnessError::SourceConflict);
            }
            continue;
        }
        if other.layer != source.layer {
            continue;
        }
        let e = if other.midpoint.is_some() {
            arc_output_witness(other)?.occupied_deviation_upper_nm()
        } else {
            Some(R::integer(0))
        };
        pairs.push(pair_check(
            other.uuid,
            deviation,
            e,
            clearance_nm,
            &|n, b| tracks_within(source, other, n, b),
        )?);
    }
    for pad in pads {
        if !pad_layers(stackup, pad)?.contains(&source.layer) {
            continue;
        }
        let certified_aperture = matches!(pad.shape, crate::board::PadShape::Circle)
            || (pad.shape == crate::board::PadShape::Rect && pad.rotation.rem_euclid(180) == 0);
        pairs.push(pair_check(
            pad.uuid,
            deviation,
            certified_aperture.then_some(R::integer(0)),
            clearance_nm,
            &|n, b| track_pad_within(source, pad, n, b),
        )?);
    }
    for via in vias {
        if !via_layers(stackup, via)?.contains(&source.layer) {
            continue;
        }
        pairs.push(pair_check(
            via.uuid,
            deviation,
            Some(R::integer(0)),
            clearance_nm,
            &|n, b| track_via_within(source, via, n, b),
        )?);
    }
    pairs.sort_by_key(|p| p.other_source_id);
    Ok(ArcOutputConstraintReport {
        source_id: source.uuid,
        minimum_width,
        pairs,
        requires_model_bound_fill_and_complete_inventory: true,
    })
}

pub(super) fn pair_check(
    id: Uuid,
    deviation: Option<R>,
    candidate_error: Option<R>,
    clearance_nm: i64,
    predicate: &dyn Fn(i64, DistanceBoundary) -> Result<bool, GeometryError>,
) -> Result<ArcOutputPairCheck, ArcWitnessError> {
    let contact = predicate(0, DistanceBoundary::Inclusive)?;
    let violation = predicate(clearance_nm, DistanceBoundary::Strict)?;
    let (clearance, topology) = if let (Some(a), Some(b)) = (deviation, candidate_error) {
        let expansion = a.checked_add(b)?;
        let ceiling = expansion.numerator / expansion.denominator
            + i128::from(expansion.numerator % expansion.denominator != 0);
        let delta = i64::try_from(ceiling).map_err(|_| ArcWitnessError::OutputRange)?;
        let threshold = clearance_nm
            .checked_add(delta)
            .ok_or(ArcWitnessError::OutputRange)?;
        let clearance = if violation || predicate(threshold, DistanceBoundary::Strict)? {
            OutputMarginResult::AtRisk
        } else {
            OutputMarginResult::Preserved
        };
        // Zero-error geometry preserves BOTH connected and disconnected
        // topology. A nonzero bound protects separation only; existing
        // tangent/interior contact needs an explicit connection witness.
        let topology = if expansion.numerator == 0
            || (!contact && !predicate(delta, DistanceBoundary::Inclusive)?)
        {
            OutputMarginResult::Preserved
        } else {
            OutputMarginResult::AtRisk
        };
        (clearance, topology)
    } else {
        (
            OutputMarginResult::Unavailable,
            OutputMarginResult::Unavailable,
        )
    };
    Ok(ArcOutputPairCheck {
        other_source_id: id,
        nominal_contact: contact,
        nominal_clearance_violation: violation,
        clearance,
        topology,
    })
}
