//! Qualified placed-symbol definition reads. Declared revisions win; native
//! unbound placements use their unique canonical birth, never a visual guess.
use super::super::{DesignModel, Operation, TransactionKind};
use super::*;
use serde_json::Value;

fn invalid(message: &str) -> EngineError {
    EngineError::Validation(format!("Placed library source: {message}"))
}

impl DesignModel {
    /// None means there is no authenticated frozen basis. Callers must not
    /// represent that result as a valid pinned binding or silently update it.
    pub fn materialized_placed_symbol_definition(
        &self,
        placed: ObjectId,
        library_symbol: ObjectId,
    ) -> Result<Option<Value>, EngineError> {
        if self.journal_cursor.applied_transaction_count != self.journal.len() {
            return Err(invalid("journal tip is not the resolved source basis"));
        }
        let object = self
            .objects
            .get(&placed)
            .filter(|o| o.domain == "schematic")
            .ok_or_else(|| invalid("placed object is missing or foreign"))?;
        let shard = self
            .source_shards
            .iter()
            .find(|s| s.shard_id == object.source_shard_id)
            .ok_or_else(|| invalid("placed source shard is unavailable"))?;
        let sheet = self.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        if !matches!(
            shard.kind,
            SourceShardKind::SchematicSheet
                | SourceShardKind::SchematicRoot
                | SourceShardKind::SchematicDefinition
        ) {
            return Err(invalid("placed source is not a schematic partition"));
        }
        // Registry kinds for UUID-keyed maps are their key, not "symbols".
        // Authenticate the typed source membership and embedded identity instead.
        let key = placed.to_string();
        let mut rows = Vec::new();
        if let Some(row) = sheet.get("symbols").and_then(|s| s.get(&key)) {
            rows.push(row);
        }
        if let Some(sheets) = sheet.get("sheets").and_then(Value::as_object) {
            rows.extend(
                sheets
                    .values()
                    .filter_map(|s| s.get("symbols").and_then(|s| s.get(&key))),
            );
        }
        if rows.len() != 1 {
            return Err(invalid(
                "placed Symbol source membership is unavailable or ambiguous",
            ));
        }
        let raw = rows[0];
        if raw
            .get("uuid")
            .and_then(Value::as_str)
            .and_then(|id| Uuid::parse_str(id).ok())
            != Some(placed)
        {
            return Err(invalid(
                "placed source identity differs from its membership",
            ));
        }
        if raw
            .get("lib_id")
            .and_then(Value::as_str)
            .and_then(|id| Uuid::parse_str(id).ok())
            != Some(library_symbol)
        {
            return Err(invalid("library identity does not match the placed source"));
        }
        let owners: Vec<_> = self
            .component_instances
            .values()
            .filter(|ci| {
                ci.authority == ComponentInstanceAuthority::Authored
                    && ci.placed_symbol_refs.contains(&placed)
            })
            .collect();
        for shard in self
            .source_shards
            .iter()
            .filter(|s| s.kind == SourceShardKind::ComponentInstance)
        {
            let raw =
                self.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
            let persisted: super::ComponentInstanceShard = serde_json::from_value(raw)?;
            let input = persisted.component_instance;
            if input
                .placed_symbol_refs
                .iter()
                .any(|r| r.object_id == placed)
                && !owners.iter().any(|owner| owner.id == input.uuid)
            {
                return Err(invalid(
                    "declared placed ownership failed canonical binding resolution",
                ));
            }
        }
        if owners.len() > 1 {
            return Err(invalid("placed binding ownership is ambiguous"));
        }
        if let Some(owner) = owners.first() {
            let bindings: Vec<_> = owner
                .library_bindings
                .values()
                .filter(|b| b.binding_role == LibraryBindingRole::Symbol)
                .collect();
            if !bindings.is_empty() {
                let mut revision = None;
                for binding in bindings
                    .into_iter()
                    .filter(|b| b.target_object_id == library_symbol)
                {
                    if !binding.local_override_refs.is_empty() {
                        return Ok(None);
                    }
                    if revision
                        .replace(binding.pinned_object_revision)
                        .is_some_and(|old| old != binding.pinned_object_revision)
                    {
                        return Err(invalid("declared Symbol revisions are ambiguous"));
                    }
                }
                let revision = revision
                    .ok_or_else(|| invalid("declared binding has a different Symbol identity"))?;
                return self
                    .materialized_pool_object_revision(library_symbol, "symbols", revision)
                    .map(Some);
            }
        }
        let mut birth = None;
        for (transaction_index, transaction) in self.journal.iter().enumerate() {
            if transaction.transaction_kind != TransactionKind::Normal {
                continue;
            }
            for (operation_index, op) in transaction.operations.iter().enumerate() {
                if let Operation::CreateSchematicSymbol {
                    symbol_id, symbol, ..
                } = op
                    && *symbol_id == placed
                {
                    if symbol
                        .get("lib_id")
                        .and_then(Value::as_str)
                        .and_then(|id| Uuid::parse_str(id).ok())
                        != Some(library_symbol)
                    {
                        return Ok(None);
                    }
                    if birth
                        .replace((transaction_index, operation_index))
                        .is_some()
                    {
                        return Err(invalid("canonical placement birth is ambiguous"));
                    }
                }
            }
        }
        let Some(birth) = birth else {
            return Ok(None);
        };
        let source = self
            .objects
            .get(&library_symbol)
            .filter(|o| o.domain == "pool" && o.kind == "symbols")
            .ok_or_else(|| invalid("pool Symbol source is unavailable"))?;
        let pool = self
            .source_shards
            .iter()
            .find(|s| s.shard_id == source.source_shard_id && s.kind == SourceShardKind::Pool)
            .ok_or_else(|| invalid("pool source shard is unavailable"))?;
        // The current shard is only the authenticated postimage from which
        // to reverse to this placement event, never a fallback definition.
        let current = self.materialized_source_shard_value_by_relative_path(&pool.relative_path)?;
        super::super::pool_journal_ops::validate_pool_library_object(
            library_symbol,
            &pool.relative_path,
            "symbols",
            &current,
        )?;
        let mut value = Some(current);
        // Undo only this library object's accepted operations through the
        // placement transaction's before-model boundary. This is an in-memory read, not model/source replay.
        for (transaction_index, transaction) in self.journal.iter().enumerate().rev() {
            for op in transaction.operations.iter().rev() {
                match op {
                    Operation::SetPoolLibraryObject {
                        object_id,
                        relative_path,
                        object_kind,
                        previous_object,
                        object,
                    } if *object_id == library_symbol => {
                        require_source(relative_path, object_kind, &pool.relative_path)?;
                        if value.as_ref() != Some(object) {
                            return Err(invalid(
                                "pool postimage does not match the resolved history basis",
                            ));
                        }
                        value = Some(previous_object.clone());
                    }
                    Operation::CreatePoolLibraryObject {
                        object_id,
                        relative_path,
                        object_kind,
                        object,
                    } if *object_id == library_symbol => {
                        require_source(relative_path, object_kind, &pool.relative_path)?;
                        if value.as_ref() != Some(object) {
                            return Err(invalid("pool creation postimage differs"));
                        }
                        value = None;
                    }
                    Operation::DeletePoolLibraryObject {
                        object_id,
                        relative_path,
                        object_kind,
                        object,
                    } if *object_id == library_symbol => {
                        require_source(relative_path, object_kind, &pool.relative_path)?;
                        if value.is_some() {
                            return Err(invalid("pool deletion history differs"));
                        }
                        value = Some(object.clone());
                    }
                    _ => {}
                }
            }
            if transaction_index == birth.0 {
                let value = value
                    .ok_or_else(|| invalid("Symbol did not exist in the placement before-model"))?;
                super::super::pool_journal_ops::validate_pool_library_object(
                    library_symbol,
                    &pool.relative_path,
                    "symbols",
                    &value,
                )?;
                return Ok(Some(value));
            }
        }
        Err(invalid("canonical placement history is unavailable"))
    }
}

fn require_source(path: &str, kind: &str, expected: &str) -> Result<(), EngineError> {
    if path != expected || kind != "symbols" {
        return Err(invalid(
            "pool identity/path differs from the qualified source",
        ));
    }
    Ok(())
}
