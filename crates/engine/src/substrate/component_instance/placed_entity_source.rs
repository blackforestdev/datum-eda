//! Read-only Entity evidence at an effective canonical placement or assignment.
use super::super::{DesignModel, Operation, TransactionKind};
use super::declaration_history::authenticate_assignment;
use super::*;
use serde_json::Value;

fn invalid(message: &str) -> EngineError {
    EngineError::Validation(format!("Placed Entity source: {message}"))
}
fn chain(raw: &Value) -> Result<[Option<Uuid>; 3], EngineError> {
    let mut result = [None; 3];
    for (index, field) in ["entity", "gate", "lib_id"].iter().enumerate() {
        if !raw[*field].is_null() {
            result[index] = Some(
                raw[*field]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .ok_or_else(|| invalid("placed chain identity is malformed"))?,
            );
        }
    }
    Ok(result)
}
impl DesignModel {
    pub(crate) fn materialized_placed_entity_definition(
        &self,
        placed: ObjectId,
        entity: ObjectId,
        gate: ObjectId,
        symbol: ObjectId,
    ) -> Result<Value, EngineError> {
        self.materialized_placed_symbol_definition(placed, symbol)?
            .ok_or_else(|| invalid("authenticated placed Symbol basis is unavailable"))?;
        // Validate established compensation ordering, then identify which normal
        // adoptions remain effective. Geometry/annotation Sets never adopt a chain.
        let history = super::declaration_history::effective_history(self)?;
        let origins = history.origins;
        let effective = history.effective;
        let births = self.journal.iter()
            .filter(|tx| tx.transaction_kind == TransactionKind::Normal)
            .flat_map(|tx| &tx.operations)
            .filter(|op| matches!(op, Operation::CreateSchematicSymbol { symbol_id, .. } if *symbol_id == placed))
            .count();
        if births > 1 {
            return Err(invalid("canonical placement birth is ambiguous"));
        }
        let placed_owner = self
            .objects
            .get(&placed)
            .ok_or_else(|| invalid("placed source is missing"))?;
        let placed_shard = self
            .source_shards
            .iter()
            .find(|s| s.shard_id == placed_owner.source_shard_id)
            .ok_or_else(|| invalid("placed source shard is missing"))?;
        let source =
            self.materialized_source_shard_value_by_relative_path(&placed_shard.relative_path)?;
        let key = placed.to_string();
        let mut rows = Vec::new();
        if let Some(raw) = source.get("symbols").and_then(|s| s.get(&key)) {
            rows.push((source["uuid"].clone(), raw.clone()));
        }
        if let Some(sheets) = source.get("sheets").and_then(Value::as_object) {
            rows.extend(sheets.values().filter_map(|s| {
                s.get("symbols")
                    .and_then(|m| m.get(&key))
                    .map(|r| (s["uuid"].clone(), r.clone()))
            }));
        }
        if rows.len() != 1 {
            return Err(invalid("placed membership is ambiguous or unavailable"));
        }
        let (sheet, placed_raw) = rows.pop().unwrap();
        let sheet: Uuid = serde_json::from_value(sheet)?;
        let expected = [Some(entity), Some(gate), Some(symbol)];
        if chain(&placed_raw)? != expected {
            return Err(invalid("placed source chain differs"));
        }
        let mut placed_raw = Some(placed_raw);
        let owner = self
            .objects
            .get(&entity)
            .filter(|o| o.domain == "pool" && o.kind == "entities")
            .ok_or_else(|| invalid("Entity source is unavailable"))?;
        let shard = self
            .source_shards
            .iter()
            .find(|s| s.shard_id == owner.source_shard_id && s.kind == SourceShardKind::Pool)
            .ok_or_else(|| invalid("Entity pool source is unavailable"))?;
        let mut value =
            self.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        super::super::pool_journal_ops::validate_pool_library_object(
            entity,
            &shard.relative_path,
            "entities",
            &value,
        )?;
        for tx in self.journal.iter().rev() {
            let mut adoption = false;
            let changes: Vec<_> = tx
                .operations
                .iter()
                .filter(|op| {
                    matches!(op,
                Operation::CreateSchematicSymbol { symbol_id, .. }
                | Operation::SetSchematicSymbol { symbol_id, .. }
                | Operation::DeleteSchematicSymbol { symbol_id, .. } if *symbol_id == placed)
                })
                .collect();
            if changes.len() > 1 {
                return Err(invalid("placed transaction has ambiguous source changes"));
            }
            if let Some(op) = changes.first() {
                if let Operation::DeleteSchematicSymbol {
                    sheet_id,
                    symbol_id: _,
                    symbol: deleted,
                } = op
                {
                    // Undoing a birth temporarily removes its source. Redo of
                    // that same authenticated birth does not create a new epoch.
                    let origin = origins.get(&tx.transaction_id).and_then(|id| {
                        self.journal
                            .iter()
                            .find(|candidate| candidate.transaction_id == *id)
                    });
                    let birth_matches = origin.is_some_and(|origin| origin.operations.iter().any(|op|
                        matches!(op, Operation::CreateSchematicSymbol { sheet_id: old_sheet, symbol_id, symbol: raw }
                            if *old_sheet == sheet && *symbol_id == placed && raw == deleted)));
                    let retracted_delete_matches = origin.is_some_and(|origin|
                        !effective.contains(&origin.transaction_id)
                        && matches!(tx.transaction_kind, TransactionKind::Normal | TransactionKind::Redo)
                        && origin.operations.iter().any(|op| matches!(op,
                            Operation::DeleteSchematicSymbol { sheet_id: old_sheet, symbol_id, symbol: raw }
                            if *old_sheet == sheet && *symbol_id == placed && raw == deleted)));
                    let inverse_matches = tx.inverse_operations.iter().filter(|op|
                        matches!(op, Operation::CreateSchematicSymbol { sheet_id: old_sheet, symbol_id, symbol: raw }
                            if *old_sheet == sheet && *symbol_id == placed && raw == deleted)).count() == 1;
                    if !((tx.transaction_kind == TransactionKind::Undo && birth_matches)
                        || retracted_delete_matches)
                        || *sheet_id != sheet
                        || placed_raw.is_some()
                        || !inverse_matches
                    {
                        return Err(invalid(
                            "placed history crosses an unsupported source epoch",
                        ));
                    }
                    placed_raw = Some(deleted.clone());
                } else {
                    let (op_sheet, next, create) = match op {
                        Operation::SetSchematicSymbol {
                            sheet_id, symbol, ..
                        } => (*sheet_id, symbol, false),
                        Operation::CreateSchematicSymbol {
                            sheet_id, symbol, ..
                        } => (*sheet_id, symbol, true),
                        _ => unreachable!(),
                    };
                    if op_sheet != sheet || placed_raw.as_ref() != Some(next) {
                        return Err(invalid("placed history postimage differs"));
                    }
                    let inverses: Vec<_> = tx
                        .inverse_operations
                        .iter()
                        .filter_map(|op| match op {
                            Operation::SetSchematicSymbol {
                                sheet_id,
                                symbol_id,
                                symbol,
                            }
                            | Operation::DeleteSchematicSymbol {
                                sheet_id,
                                symbol_id,
                                symbol,
                            } if *symbol_id == placed && *sheet_id == sheet => Some((op, symbol)),
                            _ => None,
                        })
                        .collect();
                    if inverses.len() != 1 {
                        return Err(invalid("placed history preimage is missing or ambiguous"));
                    }
                    let (inverse, before) = inverses[0];
                    if create {
                        if tx.transaction_kind != TransactionKind::Normal {
                            let origin = origins.get(&tx.transaction_id).and_then(|id| {
                                self.journal
                                    .iter()
                                    .find(|candidate| candidate.transaction_id == *id)
                            });
                            let linked = origin.is_some_and(|origin| {
                                origin.operations.iter().any(|op| match op {
                                    Operation::CreateSchematicSymbol {
                                        sheet_id,
                                        symbol_id,
                                        symbol,
                                    } => {
                                        tx.transaction_kind == TransactionKind::Redo
                                            && *sheet_id == sheet
                                            && *symbol_id == placed
                                            && symbol == next
                                    }
                                    Operation::DeleteSchematicSymbol {
                                        sheet_id,
                                        symbol_id,
                                        symbol,
                                    } => {
                                        tx.transaction_kind == TransactionKind::Undo
                                            && *sheet_id == sheet
                                            && *symbol_id == placed
                                            && symbol == next
                                    }
                                    _ => false,
                                })
                            });
                            if !linked {
                                return Err(invalid(
                                    "restored placed source differs from its canonical origin",
                                ));
                            }
                        }
                        if !matches!(inverse, Operation::DeleteSchematicSymbol { .. })
                            || before != next
                        {
                            return Err(invalid("placement birth inverse differs"));
                        }
                    } else if !matches!(inverse, Operation::SetSchematicSymbol { .. }) {
                        return Err(invalid("placed Set inverse differs"));
                    }
                    adoption = tx.transaction_kind == TransactionKind::Normal
                        && effective.contains(&tx.transaction_id)
                        && chain(next)? == expected
                        && (create || chain(before)? != expected);
                    if adoption && !create {
                        authenticate_assignment(self, tx, placed)?;
                    }
                    placed_raw = if create { None } else { Some(before.clone()) };
                }
            }
            // Reverse the exact accepted Entity postimages to the adoption's
            // before-model; compensation participates just like normal writes.
            for op in tx.operations.iter().rev() {
                if let Operation::SetPoolLibraryObject {
                    object_id,
                    relative_path,
                    object_kind,
                    previous_object,
                    object,
                } = op
                    && *object_id == entity
                {
                    if relative_path != &shard.relative_path
                        || object_kind != "entities"
                        || &value != object
                    {
                        return Err(invalid(
                            "Entity history does not match its exact source postimage",
                        ));
                    }
                    value = previous_object.clone();
                }
                if matches!(op, Operation::CreatePoolLibraryObject { object_id, .. }
                    | Operation::DeletePoolLibraryObject { object_id, .. } if *object_id == entity)
                {
                    return Err(invalid(
                        "Entity history crosses an unsupported source epoch",
                    ));
                }
            }
            if adoption {
                super::super::pool_journal_ops::validate_pool_library_object(
                    entity,
                    &shard.relative_path,
                    "entities",
                    &value,
                )?;
                let gates = value["gates"]
                    .as_object()
                    .ok_or_else(|| invalid("captured gates are unavailable"))?;
                if gates
                    .keys()
                    .filter(|k| k.parse::<Uuid>().ok() == Some(gate))
                    .count()
                    != 1
                {
                    return Err(invalid("captured gate membership is ambiguous"));
                }
                return Ok(value);
            }
            if tx.transaction_kind == TransactionKind::Normal
                && changes
                    .iter()
                    .any(|op| matches!(op, Operation::CreateSchematicSymbol { .. }))
            {
                return Err(invalid(
                    "placement has no authenticated captured gate chain",
                ));
            }
        }
        Err(invalid("effective captured gate chain is unavailable"))
    }
}
