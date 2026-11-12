//! Authenticated native body alignment using the shared complete-set writer.
pub use super::alignment_geometry::{AlignmentAxis, AlignmentPosition};
use super::{
    alignment_geometry::{BodyBounds, destinations},
    context::{PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, parse_verb_params},
    schematic_edit::place_local_point,
    schematic_symbol_move::{SymbolMoveSetBasis, SymbolMoveSetDestination},
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    pool::{Primitive, Symbol},
    schematic::PlacedSymbol,
    substrate::{DesignModel, ElectricalSelectionSnapshot},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentOperation {
    #[default]
    Align,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentReference {
    #[default]
    SelectionBounds,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SymbolAlignmentRequest {
    pub symbols: Vec<Uuid>,
    pub instance_path: Vec<Uuid>,
    #[serde(default)]
    pub op: AlignmentOperation,
    pub axis: AlignmentAxis,
    pub position: AlignmentPosition,
    #[serde(default)]
    pub reference: AlignmentReference,
}

/// Preview and commit use the same model-qualified body and contact capture.
pub fn plan_alignment(
    model: &DesignModel,
    request: &SymbolAlignmentRequest,
) -> Result<(SymbolMoveSetBasis, SymbolMoveSetDestination), EngineError> {
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Alignment source unavailable: {e:?}")))?;
    let basis = SymbolMoveSetBasis::capture(&snapshot, &request.symbols, &request.instance_path)?;
    let bodies = basis
        .symbols
        .iter()
        .map(|placed| body_bounds(model, placed).map(|bounds| (placed.position, bounds)))
        .collect::<Result<Vec<_>, _>>()?;
    let positions = destinations(&bodies, request.axis, request.position)?;
    let destination = basis.destination_positions(&positions)?;
    Ok((basis, destination))
}

pub(super) fn align_symbols(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let request = parse_verb_params("datum.schematic.align_symbols", params)?;
    let (basis, destination) = plan_alignment(context.model, &request)?;
    basis.prepare_destination(context, provenance, destination)
}

fn body_bounds(model: &DesignModel, placed: &PlacedSymbol) -> Result<BodyBounds, EngineError> {
    let id = placed
        .lib_id
        .as_ref()
        .and_then(|id| id.parse::<Uuid>().ok())
        .ok_or_else(|| invalid("Alignment requires an authenticated native Symbol body"))?;
    let raw = model
        .materialized_placed_symbol_definition(placed.uuid, id)?
        .ok_or_else(|| invalid("Alignment Symbol body has no authenticated placed source"))?;
    let definition: Symbol = serde_json::from_value(raw)?;
    let mut bounds: Option<BodyBounds> = None;
    let mut include = |points: &[Point], width: i64| -> Result<(), EngineError> {
        if width < 0 || points.is_empty() {
            return Err(invalid(
                "Alignment body has invalid stroke or empty geometry",
            ));
        }
        for point in points {
            let point =
                place_local_point(*point, placed.position, placed.rotation, placed.mirrored)?;
            let low = [
                2 * i128::from(point.x) - i128::from(width),
                2 * i128::from(point.y) - i128::from(width),
            ];
            let high = [
                2 * i128::from(point.x) + i128::from(width),
                2 * i128::from(point.y) + i128::from(width),
            ];
            match &mut bounds {
                Some(bounds) => {
                    for i in 0..2 {
                        bounds.min[i] = bounds.min[i].min(low[i]);
                        bounds.max[i] = bounds.max[i].max(high[i]);
                    }
                }
                None => {
                    bounds = Some(BodyBounds {
                        min: low,
                        max: high,
                    })
                }
            }
        }
        Ok(())
    };
    for drawing in &definition.drawings {
        match drawing {
            Primitive::Rect { min, max, width } => {
                if min.x > max.x || min.y > max.y {
                    return Err(invalid("Alignment body rectangle is invalid"));
                }
                include(
                    &[
                        *min,
                        *max,
                        Point::new(min.x, max.y),
                        Point::new(max.x, min.y),
                    ],
                    *width,
                )?;
            }
            Primitive::Circle {
                center,
                radius,
                width,
            } => {
                if *radius < 0 {
                    return Err(invalid("Alignment body circle is invalid"));
                }
                let low = Point::new(
                    center
                        .x
                        .checked_sub(*radius)
                        .ok_or_else(|| invalid("Body coordinate overflow"))?,
                    center
                        .y
                        .checked_sub(*radius)
                        .ok_or_else(|| invalid("Body coordinate overflow"))?,
                );
                let high = Point::new(
                    center
                        .x
                        .checked_add(*radius)
                        .ok_or_else(|| invalid("Body coordinate overflow"))?,
                    center
                        .y
                        .checked_add(*radius)
                        .ok_or_else(|| invalid("Body coordinate overflow"))?,
                );
                include(
                    &[
                        low,
                        high,
                        Point::new(low.x, high.y),
                        Point::new(high.x, low.y),
                    ],
                    *width,
                )?;
            }
            Primitive::Line { from, to, width } => include(&[*from, *to], *width)?,
            Primitive::Polygon { polygon, width: 0 } => include(&polygon.vertices, 0)?,
            _ => {
                return Err(invalid(
                    "Alignment body extent unavailable for this primitive; selection unchanged",
                ));
            }
        }
    }
    bounds.ok_or_else(|| invalid("Alignment requires a nonempty authenticated Symbol body"))
}

fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}

#[cfg(test)]
#[path = "schematic_alignment_tests.rs"]
mod tests;
