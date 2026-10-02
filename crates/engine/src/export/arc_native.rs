//! Capture once through existing engine authority; no raw fill/outline fallback.
use super::arc_constraints::pair_check;
use super::{
    ArcOutputConstraintContext, ArcOutputConstraintReport, ArcOutputWitness, ArcWitnessError,
    arc_output_witness, check_arc_output_constraints,
};
use crate::board::{
    nominal_geometry::{GeometryError, Rational as R},
    occupied_copper::track_polygon_within,
};
use crate::substrate::{
    DesignModel, ElectricalQueryFailure, ElectricalSelectionSnapshot, ModelRevision,
    SelectionSourceBasis,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeArcOutputError {
    Authority(ElectricalQueryFailure),
    MissingSource(Uuid),
    Witness(ArcWitnessError),
    UnverifiedFill(Uuid),
}
impl From<ArcWitnessError> for NativeArcOutputError {
    fn from(e: ArcWitnessError) -> Self {
        Self::Witness(e)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeArcOutputReport {
    pub project_id: Uuid,
    pub model_revision: ModelRevision,
    pub source_basis: Vec<SelectionSourceBasis>,
    pub witness: ArcOutputWitness,
    /// Margins concern supplied rule values against captured nominal neighbours.
    /// This is not a full manufacturing-set/reader/process acceptance statement.
    pub constraints: ArcOutputConstraintReport,
}
/// Width/clearance must come from the caller's applicable rules, never an
/// inferred fabrication allowance. This report cannot authorize output.
pub fn native_arc_output_report(
    model: &DesignModel,
    source_id: Uuid,
    minimum_width_nm: i64,
    clearance_nm: i64,
) -> Result<NativeArcOutputReport, NativeArcOutputError> {
    let snapshot =
        ElectricalSelectionSnapshot::capture(model).map_err(NativeArcOutputError::Authority)?;
    let board = snapshot
        .board_source()
        .map_err(NativeArcOutputError::Authority)?;
    let source = board
        .tracks
        .get(&source_id)
        .ok_or(NativeArcOutputError::MissingSource(source_id))?;
    let witness = arc_output_witness(source)?;
    let tracks = board.tracks.values().cloned().collect::<Vec<_>>();
    let pads = board.pads.values().cloned().collect::<Vec<_>>();
    let vias = board.vias.values().cloned().collect::<Vec<_>>();
    let mut constraints = check_arc_output_constraints(
        &witness,
        ArcOutputConstraintContext {
            stackup: &board.stackup,
            minimum_width_nm,
            clearance_nm,
            tracks: &tracks,
            pads: &pads,
            vias: &vias,
        },
    )?;
    for zone in board.zones.values().filter(|z| z.layer == source.layer) {
        let fill = snapshot
            .current_zone_fill(zone.uuid)
            .ok_or(NativeArcOutputError::UnverifiedFill(zone.uuid))?;
        let pair = pair_check(
            zone.uuid,
            witness.occupied_deviation_upper_nm(),
            Some(R::integer(0)),
            clearance_nm,
            &|n, b| {
                let mut contact = false;
                for polygon in &fill.islands {
                    contact |= track_polygon_within(source, polygon, n, b)?;
                }
                Ok::<_, GeometryError>(contact)
            },
        )?;
        constraints.pairs.push(pair);
    }
    constraints.pairs.sort_by_key(|p| p.other_source_id);
    constraints.requires_model_bound_fill_and_complete_inventory = false;
    Ok(NativeArcOutputReport {
        project_id: snapshot.project_id(),
        model_revision: snapshot.revision().clone(),
        source_basis: snapshot.source_basis().to_vec(),
        witness,
        constraints,
    })
}
