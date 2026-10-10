//! Source-qualified unconnected Symbol move through canonical Set and history.
use super::{
    context::{PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, parse_verb_params},
    schematic_symbols::build_set_schematic_symbol,
};
use crate::{
    connectivity::SymbolContactBasis,
    error::EngineError,
    ir::geometry::Point,
    schematic::PlacedSymbol,
    substrate::{ElectricalSelectionSnapshot, SelectionSourceDomain},
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
        if symbol.fields.iter().any(|f| f.position.is_some())
            || symbol.pin_overrides.iter().any(|p| p.position.is_some())
        {
            return Err(invalid(
                "Symbol move with positioned fields or pin overrides is not supported",
            ));
        }
        let contacts = SymbolContactBasis::capture(sheet, symbol_uuid)?;
        contacts.require_detached(&symbol)?;
        Ok(Self {
            sheet_id: sheet.uuid,
            symbol,
            contacts,
        })
    }

    pub fn destination(&self, position: Point) -> Result<PlacedSymbol, EngineError> {
        // i128 deltas allow the full i64 origin domain; only the final stored
        // coordinates must fit. No rounding or newly invented tolerance.
        let dx = i128::from(position.x) - i128::from(self.symbol.position.x);
        let dy = i128::from(position.y) - i128::from(self.symbol.position.y);
        let mut symbol = self.symbol.clone();
        symbol.position = position;
        for pin in &mut symbol.pins {
            pin.position = Point {
                x: i64::try_from(i128::from(pin.position.x) + dx)
                    .map_err(|_| invalid("Symbol pin coordinate overflow"))?,
                y: i64::try_from(i128::from(pin.position.y) + dy)
                    .map_err(|_| invalid("Symbol pin coordinate overflow"))?,
            };
        }
        self.contacts.require_detached(&self.symbol)?;
        self.contacts.require_detached(&symbol)?;
        Ok(symbol)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveParams {
    symbol: Uuid,
    instance_path: Vec<Uuid>,
    x_nm: i64,
    y_nm: i64,
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
    let symbol = basis.destination(Point {
        x: params.x_nm,
        y: params.y_nm,
    })?;
    if symbol.position == basis.symbol.position {
        return Err(invalid("Symbol move destination is unchanged"));
    }
    build_set_schematic_symbol(context.model, provenance, basis.sheet_id, &symbol)
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}

#[cfg(test)]
#[path = "schematic_symbol_move_tests.rs"]
mod tests;
