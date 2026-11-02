//! Complete Symbol-set pose over one immutable contact and source basis.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Group shape of the existing move_symbol verb. Rotation/reflection are deltas
/// about the pickup reference; x/y specify its translated destination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SymbolMoveSetRequest {
    pub symbols: Vec<Uuid>,
    pub instance_path: Vec<Uuid>,
    pub reference: Point,
    pub x_nm: i64,
    pub y_nm: i64,
    #[serde(default)]
    pub rotation_deg: i32,
    #[serde(default)]
    pub mirrored: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolMoveSetBasis {
    pub sheet_id: Uuid,
    pub symbols: Vec<PlacedSymbol>,
    pub(super) contacts: SymbolContactBasis,
    locked_wires: BTreeSet<Uuid>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolMoveSetDestination {
    pub symbols: Vec<PlacedSymbol>,
    pub wires: Vec<SchematicWire>,
}
impl SymbolMoveSetBasis {
    pub fn capture(
        snapshot: &ElectricalSelectionSnapshot,
        ids: &[Uuid],
        path: &[Uuid],
    ) -> Result<Self, EngineError> {
        capture(snapshot, ids, path, true)
    }
    pub fn destination_with_transform(
        &self,
        reference: Point,
        destination: Point,
        rotation_delta: i32,
        mirrored: bool,
    ) -> Result<SymbolMoveSetDestination, EngineError> {
        if rotation_delta.rem_euclid(90) != 0 {
            return Err(invalid("Group rotation requires an exact cardinal delta"));
        }
        let rotation = rotation_delta.rem_euclid(360);
        let pose_changed = rotation != 0 || mirrored;
        let transform = |point: Point| -> Result<Point, EngineError> {
            let relative = (
                i128::from(point.x) - i128::from(reference.x),
                i128::from(point.y) - i128::from(reference.y),
            );
            let (x, y) = cardinal_symbol_point(relative, rotation, mirrored)?;
            Ok(Point::new(
                i64::try_from(i128::from(destination.x) + x)
                    .map_err(|_| invalid("Group coordinate overflow"))?,
                i64::try_from(i128::from(destination.y) + y)
                    .map_err(|_| invalid("Group coordinate overflow"))?,
            ))
        };
        let mut symbols = self.symbols.clone();
        for symbol in &mut symbols {
            if pose_changed {
                if symbol.rotation.rem_euclid(90) != 0 {
                    return Err(invalid(
                        "Group rotation requires exact cardinal source poses",
                    ));
                }
                symbol.rotation = (rotation
                    + if mirrored {
                        -symbol.rotation.rem_euclid(360)
                    } else {
                        symbol.rotation.rem_euclid(360)
                    })
                .rem_euclid(360);
                symbol.mirrored ^= mirrored;
            }
            symbol.position = transform(symbol.position)?;
            for pin in &mut symbol.pins {
                pin.position = transform(pin.position)?;
            }
            for field in &mut symbol.fields {
                field.position = field.position.map(transform).transpose()?;
            }
        }
        let wires = self.contacts.moved_symbols(&symbols)?;
        if wires
            .iter()
            .any(|wire| self.locked_wires.contains(&wire.uuid))
        {
            return Err(invalid(
                "Group transform would change a locked attached Wire",
            ));
        }
        Ok(SymbolMoveSetDestination { symbols, wires })
    }
}

pub(super) fn capture(
    snapshot: &ElectricalSelectionSnapshot,
    ids: &[Uuid],
    path: &[Uuid],
    enforce_locks: bool,
) -> Result<SymbolMoveSetBasis, EngineError> {
    let ids_set: BTreeSet<_> = ids.iter().copied().collect();
    if ids_set.is_empty() || ids_set.len() != ids.len() {
        return Err(invalid(
            "Transform requires a nonempty complete set of distinct Symbols",
        ));
    }
    let authored = snapshot
        .authored_sources()
        .map_err(|e| invalid(format!("Symbol occurrence basis unavailable: {e:?}")))?;
    let schematic = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Native schematic source unavailable"))?;
    let mut sheet_id = None;
    for id in &ids_set {
        let occurrences: Vec<_> = authored
            .iter()
            .filter(|s| {
                s.domain == SelectionSourceDomain::Schematic
                    && s.occurrence.class == "symbols"
                    && s.occurrence.source_id == *id
            })
            .collect();
        if occurrences.len() != 1 || occurrences[0].occurrence.instance_path != path {
            return Err(invalid(
                "Symbol move requires exactly one complete matching occurrence",
            ));
        }
        let owners: Vec<_> = schematic
            .sheets
            .values()
            .filter(|s| s.symbols.contains_key(id))
            .collect();
        if owners.len() != 1 || sheet_id.is_some_and(|sheet| sheet != owners[0].uuid) {
            return Err(invalid(
                "Transform requires one complete source sheet occurrence",
            ));
        }
        sheet_id = Some(owners[0].uuid);
    }
    let sheet = &schematic.sheets[&sheet_id.unwrap()];
    if sheet.symbols.iter().any(|(id, s)| *id != s.uuid) {
        return Err(invalid("Symbol source identity is inconsistent"));
    }
    let mut pins = BTreeSet::new();
    if sheet
        .symbols
        .values()
        .flat_map(|s| &s.pins)
        .any(|p| !pins.insert(p.uuid))
    {
        return Err(invalid("Symbol pin source identity is ambiguous"));
    }
    let symbols: Vec<_> = ids_set.iter().map(|id| sheet.symbols[id].clone()).collect();
    let mut fields = BTreeSet::new();
    for symbol in &symbols {
        if symbol.fields.iter().any(|f| !fields.insert(f.uuid)) {
            return Err(invalid("Symbol field source identity is ambiguous"));
        }
        if symbol.pin_overrides.iter().any(|p| p.position.is_some()) {
            return Err(invalid(
                "Symbol move with positioned pin overrides is not supported",
            ));
        }
    }
    let mut locked_wires = BTreeSet::new();
    if enforce_locks {
        // This is the same source predicate as the shared selection view:
        // explicit raw `locked: true`; no inferred library or binding lock.
        let mut raw_sheet = None;
        for shard in snapshot.source_basis() {
            let value = snapshot
                .source_shard_value(snapshot.revision(), shard.shard_id)
                .map_err(|e| invalid(format!("Symbol lock source unavailable: {e:?}")))?;
            if value.get("uuid") == Some(&serde_json::json!(sheet.uuid)) {
                if raw_sheet.replace(value).is_some() {
                    return Err(invalid("Symbol source sheet is ambiguous"));
                }
            }
        }
        let raw = raw_sheet.ok_or_else(|| invalid("Symbol lock source unavailable"))?;
        for symbol in &symbols {
            let payload = &raw["symbols"][symbol.uuid.to_string()];
            if serde_json::from_value::<PlacedSymbol>(payload.clone())? != *symbol {
                return Err(invalid("Symbol source payload changed"));
            }
            if payload.get("locked").and_then(serde_json::Value::as_bool) == Some(true) {
                return Err(invalid("Complete selection contains a locked Symbol"));
            }
        }
        for wire in sheet.wires.values() {
            if raw["wires"][wire.uuid.to_string()]
                .get("locked")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
            {
                locked_wires.insert(wire.uuid);
            }
        }
    }
    let contacts = SymbolContactBasis::capture_symbols(
        sheet,
        &symbols.iter().map(|s| s.uuid).collect::<Vec<_>>(),
    )?;
    Ok(SymbolMoveSetBasis {
        sheet_id: sheet.uuid,
        symbols,
        contacts,
        locked_wires,
    })
}

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    snapshot: &ElectricalSelectionSnapshot,
    params: SymbolMoveSetRequest,
) -> Result<PreparedWrite, EngineError> {
    let basis = SymbolMoveSetBasis::capture(snapshot, &params.symbols, &params.instance_path)?;
    let destination = basis.destination_with_transform(
        params.reference,
        Point::new(params.x_nm, params.y_nm),
        params.rotation_deg,
        params.mirrored,
    )?;
    if destination.symbols == basis.symbols {
        return Err(invalid("Symbol group transform is unchanged"));
    }
    transform_write(
        context,
        provenance,
        basis.sheet_id,
        &basis.symbols,
        &destination.symbols,
        destination.wires,
        &basis.contacts,
    )
}
#[cfg(test)]
#[path = "schematic_symbol_move_set_tests.rs"]
mod tests;
