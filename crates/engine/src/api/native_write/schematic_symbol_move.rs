//! Source-qualified Symbol move preserving attached endpoints and electrical members.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, parse_verb_params},
    schematic_edit::cardinal_symbol_point,
    schematic_symbol_payload::{set_point, source_symbol_payload},
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
        let sources = snapshot
            .authored_sources()
            .map_err(|e| invalid(format!("Symbol occurrence basis unavailable: {e:?}")))?;
        let occurrences: Vec<_> = sources
            .iter()
            .filter(|s| {
                s.domain == SelectionSourceDomain::Schematic
                    && s.occurrence.class == "symbols"
                    && s.occurrence.source_id == symbol_uuid
            })
            .collect();
        if occurrences.len() != 1 || occurrences[0].occurrence.instance_path != instance_path {
            return Err(invalid(
                "Symbol move requires exactly one complete matching occurrence",
            ));
        }
        let schematic = snapshot
            .schematic_source()
            .ok_or_else(|| invalid("Native schematic source unavailable"))?;
        let sheets: Vec<_> = schematic
            .sheets
            .values()
            .filter(|s| s.symbols.contains_key(&symbol_uuid))
            .collect();
        if sheets.len() != 1 {
            return Err(invalid("Symbol source ownership is ambiguous"));
        }
        let sheet = sheets[0];
        let symbol = sheet.symbols[&symbol_uuid].clone();
        if symbol.uuid != symbol_uuid || sheet.symbols.iter().any(|(id, s)| *id != s.uuid) {
            return Err(invalid("Symbol source identity is inconsistent"));
        }
        let mut pin_ids = std::collections::BTreeSet::new();
        if sheet
            .symbols
            .values()
            .flat_map(|s| &s.pins)
            .any(|pin| !pin_ids.insert(pin.uuid))
        {
            return Err(invalid("Symbol pin source identity is ambiguous"));
        }
        let mut field_ids = std::collections::BTreeSet::new();
        if symbol
            .fields
            .iter()
            .any(|field| !field_ids.insert(field.uuid))
        {
            return Err(invalid("Symbol field source identity is ambiguous"));
        }
        if symbol.pin_overrides.iter().any(|p| p.position.is_some()) {
            return Err(invalid(
                "Symbol move with positioned pin overrides is not supported",
            ));
        }
        let contacts = SymbolContactBasis::capture(sheet, symbol_uuid)?;
        Ok(Self {
            sheet_id: sheet.uuid,
            symbol,
            contacts,
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
struct MoveParams {
    symbol: Uuid,
    instance_path: Vec<Uuid>,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: Option<i32>,
    mirrored: Option<bool>,
}

pub(super) fn move_symbol(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: MoveParams = parse_verb_params("datum.schematic.move_symbol", params)?;
    let snapshot = ElectricalSelectionSnapshot::capture(context.model)
        .map_err(|e| invalid(format!("Symbol source/contact basis unavailable: {e:?}")))?;
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
    let mut symbol = source_symbol_payload(context.model, basis.sheet_id, &basis.symbol)?;
    set_point(&mut symbol["position"], destination.symbol.position);
    symbol["rotation"] = serde_json::to_value(destination.symbol.rotation)?;
    symbol["mirrored"] = serde_json::to_value(destination.symbol.mirrored)?;
    for (index, pin) in destination.symbol.pins.iter().enumerate() {
        set_point(&mut symbol["pins"][index]["position"], pin.position);
    }
    for (index, field) in destination.symbol.fields.iter().enumerate() {
        if let Some(position) = field.position {
            set_point(&mut symbol["fields"][index]["position"], position);
        }
    }
    let mut operations = vec![Operation::SetSchematicSymbol {
        sheet_id: basis.sheet_id,
        symbol_id: destination.symbol.uuid,
        symbol,
    }];
    for wire in destination.wires {
        // Canonical wire operations have no Set variant. Delete/Create with
        // the same UUID is one guarded final-state edit, never an identity birth.
        operations.push(Operation::DeleteSchematicWire {
            sheet_id: basis.sheet_id,
            wire_id: wire.uuid,
            wire: serde_json::to_value(basis.contacts.original_wire(wire.uuid))?,
        });
        operations.push(Operation::CreateSchematicWire {
            sheet_id: basis.sheet_id,
            wire_id: wire.uuid,
            wire: serde_json::to_value(wire)?,
        });
    }
    BatchComposer::compose(context.model, provenance)
        .push_ops(operations)
        .primary_object(destination.symbol.uuid)
        .finish()
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}

#[cfg(test)]
#[path = "schematic_symbol_move_tests.rs"]
mod tests;
