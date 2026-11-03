//! Source-qualified Symbol move preserving attached endpoints and electrical members.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, parse_verb_params},
    schematic_edit::cardinal_symbol_point,
    schematic_symbol_payload::{member_payload, set_point, source_sheet_payload},
};
use crate::{
    connectivity::SymbolContactBasis,
    error::EngineError,
    ir::geometry::Point,
    schematic::{PlacedSymbol, SchematicWire},
    substrate::{ElectricalSelectionSnapshot, Operation, SelectionSourceDomain},
};
use serde::Deserialize;
use uuid::Uuid;

#[path = "schematic_symbol_move_set.rs"]
mod set;
pub use set::{SymbolMoveSetBasis, SymbolMoveSetDestination, SymbolMoveSetRequest};

/// Immutable preview basis; source fields and every absolute pin move together.
/// The writer captures its own current basis before authoring the guarded Set.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolMoveBasis {
    pub sheet_id: Uuid,
    pub symbol: PlacedSymbol,
    contacts: SymbolContactBasis,
}

/// Complete preview/commit result; wires keep their authored identities.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolMoveDestination {
    pub symbol: PlacedSymbol,
    pub wires: Vec<SchematicWire>,
}
impl SymbolMoveBasis {
    pub fn capture(
        snapshot: &ElectricalSelectionSnapshot,
        symbol_uuid: Uuid,
        instance_path: &[Uuid],
    ) -> Result<Self, EngineError> {
        let mut basis = set::capture(snapshot, &[symbol_uuid], instance_path, false)?;
        Ok(Self {
            sheet_id: basis.sheet_id,
            symbol: basis.symbols.remove(0),
            contacts: basis.contacts,
        })
    }

    pub fn destination(&self, position: Point) -> Result<PlacedSymbol, EngineError> {
        Ok(self.destination_edit(position)?.symbol)
    }

    pub fn destination_edit(&self, position: Point) -> Result<SymbolMoveDestination, EngineError> {
        self.destination_with_pose(position, None, None)
    }

    /// Apply an absolute requested pose to the immutable source basis. Omitted
    /// pose fields preserve historical translation-only behavior and bytes.
    pub fn destination_with_pose(
        &self,
        position: Point,
        rotation_deg: Option<i32>,
        mirrored: Option<bool>,
    ) -> Result<SymbolMoveDestination, EngineError> {
        let rotation = rotation_deg.unwrap_or(self.symbol.rotation);
        let mirrored = mirrored.unwrap_or(self.symbol.mirrored);
        let pose_changed = rotation.rem_euclid(360) != self.symbol.rotation.rem_euclid(360)
            || mirrored != self.symbol.mirrored;
        if pose_changed
            && (rotation.rem_euclid(90) != 0 || self.symbol.rotation.rem_euclid(90) != 0)
        {
            return Err(invalid("Symbol rotation requires an exact cardinal pose"));
        }
        // i128 deltas allow the full i64 origin domain; only the final stored
        // coordinates must fit. No rounding or newly invented tolerance.
        let dx = i128::from(position.x) - i128::from(self.symbol.position.x);
        let dy = i128::from(position.y) - i128::from(self.symbol.position.y);
        let mut symbol = self.symbol.clone();
        symbol.position = position;
        symbol.rotation = rotation;
        symbol.mirrored = mirrored;
        let transform = |point: Point| -> Result<Point, EngineError> {
            let (x, y) = if pose_changed {
                let relative = (
                    i128::from(point.x) - i128::from(self.symbol.position.x),
                    i128::from(point.y) - i128::from(self.symbol.position.y),
                );
                // Invert source rotation, then source reflection; apply target
                // reflection, then target rotation using the placement law.
                let unrotated =
                    cardinal_symbol_point(relative, -self.symbol.rotation.rem_euclid(360), false)?;
                let local = cardinal_symbol_point(unrotated, 0, self.symbol.mirrored)?;
                let posed = cardinal_symbol_point(local, rotation, mirrored)?;
                (
                    i128::from(position.x) + posed.0,
                    i128::from(position.y) + posed.1,
                )
            } else {
                (i128::from(point.x) + dx, i128::from(point.y) + dy)
            };
            Ok(Point {
                x: i64::try_from(x).map_err(|_| invalid("Symbol owned coordinate overflow"))?,
                y: i64::try_from(y).map_err(|_| invalid("Symbol owned coordinate overflow"))?,
            })
        };
        for pin in &mut symbol.pins {
            pin.position = transform(pin.position)?;
        }
        for field in &mut symbol.fields {
            field.position = field.position.map(transform).transpose()?;
        }
        let wires = self.contacts.moved_wires(&symbol)?;
        Ok(SymbolMoveDestination { symbol, wires })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
// Existing move_symbol schema adds optional absolute pose; absent/null values
// preserve the source pose and the historical translation-only request.
struct SingleMoveParams {
    symbol: Uuid,
    instance_path: Vec<Uuid>,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: Option<i32>,
    mirrored: Option<bool>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MoveParams {
    Single(SingleMoveParams),
    Group(SymbolMoveSetRequest),
}

pub(super) fn move_symbol(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: MoveParams = parse_verb_params("datum.schematic.move_symbol", params)?;
    let snapshot = ElectricalSelectionSnapshot::capture(context.model)
        .map_err(|e| invalid(format!("Symbol source/contact basis unavailable: {e:?}")))?;
    let params = match params {
        MoveParams::Single(params) => params,
        MoveParams::Group(params) => return set::build(context, provenance, &snapshot, params),
    };
    let basis = SymbolMoveBasis::capture(&snapshot, params.symbol, &params.instance_path)?;
    let destination = basis.destination_with_pose(
        Point {
            x: params.x_nm,
            y: params.y_nm,
        },
        params.rotation_deg,
        params.mirrored,
    )?;
    if destination.symbol == basis.symbol {
        return Err(invalid("Symbol move destination is unchanged"));
    }
    transform_write(
        context,
        provenance,
        basis.sheet_id,
        std::slice::from_ref(&basis.symbol),
        std::slice::from_ref(&destination.symbol),
        destination.wires,
        &basis.contacts,
    )
}

fn transform_write(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    sheet_id: Uuid,
    originals: &[PlacedSymbol],
    destinations: &[PlacedSymbol],
    wires: Vec<SchematicWire>,
    contacts: &SymbolContactBasis,
) -> Result<PreparedWrite, EngineError> {
    if originals.len() != destinations.len()
        || originals
            .iter()
            .zip(destinations)
            .any(|(a, b)| a.uuid != b.uuid)
    {
        return Err(invalid(
            "Transform destination does not match the complete source set",
        ));
    }
    let raw = source_sheet_payload(
        context.model,
        sheet_id,
        originals
            .first()
            .ok_or_else(|| invalid("Transform source set is empty"))?
            .uuid,
    )?;
    let mut operations = Vec::new();
    for (original, destination) in originals.iter().zip(destinations) {
        let mut symbol = member_payload(&raw, "symbols", original.uuid, original)?;
        set_point(&mut symbol["position"], destination.position);
        symbol["rotation"] = serde_json::to_value(destination.rotation)?;
        symbol["mirrored"] = serde_json::to_value(destination.mirrored)?;
        for (index, pin) in destination.pins.iter().enumerate() {
            set_point(&mut symbol["pins"][index]["position"], pin.position);
        }
        for (index, field) in destination.fields.iter().enumerate() {
            if let Some(position) = field.position {
                set_point(&mut symbol["fields"][index]["position"], position);
            }
        }
        operations.push(Operation::SetSchematicSymbol {
            sheet_id,
            symbol_id: destination.uuid,
            symbol,
        });
    }
    for wire in wires {
        let original = member_payload(&raw, "wires", wire.uuid, contacts.original_wire(wire.uuid))?;
        let mut moved = original.clone();
        set_point(&mut moved["from"], wire.from);
        set_point(&mut moved["to"], wire.to);
        operations.push(Operation::DeleteSchematicWire {
            sheet_id,
            wire_id: wire.uuid,
            wire: original,
        });
        operations.push(Operation::CreateSchematicWire {
            sheet_id,
            wire_id: wire.uuid,
            wire: moved,
        });
    }
    operations.extend(super::schematic_assignment::repin_symbols(
        context.model,
        originals,
    )?);
    let mut composer = BatchComposer::compose(context.model, provenance).push_ops(operations);
    if originals.len() == 1 {
        composer = composer.primary_object(originals[0].uuid);
    }
    composer.finish()
}

fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}

#[cfg(test)]
#[path = "schematic_symbol_move_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "schematic_bound_move_tests.rs"]
mod bound_tests;
