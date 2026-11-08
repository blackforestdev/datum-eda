//! Read-only pool revisions authenticated by the accepted canonical journal.
//! No model replay, source writes or independently persisted history lives here.
use super::super::{DesignModel, Operation};
use super::*;
use serde_json::Value;
#[path = "pool_history_restore.rs"]
mod restore;

#[derive(Clone, Copy)]
pub(in crate::substrate) struct PoolHistoryBasis<'a> {
    pub model: &'a DesignModel,
    pub operations: &'a [Operation],
    pub applied_operations: &'a [Operation],
}

impl PoolHistoryBasis<'_> {
    pub(in crate::substrate) fn definition_ref_matches(
        &self,
        staged: &BTreeMap<ObjectId, DomainObject>,
        id: ObjectId,
        revision: ObjectRevision,
        kind: &str,
    ) -> bool {
        if !matches!(kind, "symbols" | "parts" | "pin_pad_maps") {
            return false;
        }
        self.projected_definition_ref_matches(staged, id, revision, kind)
            .unwrap_or(false)
    }
    fn projected_definition_ref_matches(
        &self,
        staged: &BTreeMap<ObjectId, DomainObject>,
        id: ObjectId,
        revision: ObjectRevision,
        kind: &str,
    ) -> Result<bool, EngineError> {
        let Some(current) = staged
            .get(&id)
            .filter(|o| o.domain == "pool" && o.kind == kind)
        else {
            return Ok(false);
        };
        let history = PoolHistory {
            shards: &self.model.source_shards,
            journal: &self.model.journal,
        };
        if history.has_ambiguous_epoch(id) || self.operations.iter().any(|op|matches!(op,Operation::DeletePoolLibraryObject {object_id,..} if *object_id==id)) {return Ok(false);}
        let before = self.model.objects.get(&id).filter(|o| {
            o.source_shard_id == current.source_shard_id && o.domain == "pool" && o.kind == kind
        });
        let pending_source_change = self.operations.iter().any(|op| match op {
            Operation::SetPoolLibraryObject { object_id, .. }
            | Operation::CreatePoolLibraryObject { object_id, .. }
            | Operation::BumpObjectRevision { object_id } => *object_id == id,
            _ => false,
        });
        if !pending_source_change
            && !history.has_restored_epoch(id)
            && before.is_some_and(|before| before.object_revision == revision)
            && current.object_revision == revision
        {
            return Ok(true);
        }
        let (path, state, counter) = if let Some(before) = before {
            let path = &self
                .model
                .source_shards
                .iter()
                .find(|s| s.shard_id == before.source_shard_id && s.kind == SourceShardKind::Pool)
                .ok_or_else(|| invalid("accepted source shard is missing"))?
                .relative_path;
            let raw = self
                .model
                .materialized_source_shard_value_by_relative_path(path)?;
            (path.as_str(), Some(raw), before.object_revision.0)
        } else {
            let path = self
                .applied_operations
                .iter()
                .find_map(|op| match op {
                    Operation::CreatePoolLibraryObject {
                        object_id,
                        relative_path,
                        object_kind,
                        ..
                    } if *object_id == id && object_kind == kind => Some(relative_path.as_str()),
                    _ => None,
                })
                .ok_or_else(|| invalid("new definition has no applied creation basis"))?;
            let shard = Uuid::new_v5(
                &Uuid::NAMESPACE_URL,
                format!("datum-eda:source-shard:{path}").as_bytes(),
            );
            if shard != current.source_shard_id {
                return Ok(false);
            }
            (path, None, 0)
        };
        let mut projection =
            restore::RevisionProjection::new(id, path, kind, revision, state, counter)?;
        if let Some(before) = before
            && let Some(past) = history.payload(before, revision)?
        {
            capture(&mut projection.captured, &past)?;
        }
        for operation in self.applied_operations {
            projection.apply(operation)?;
        }
        if projection.counter != current.object_revision.0 || projection.state.is_none() {
            return Ok(false);
        }
        // Availability is established only by accepted history or operations
        // already applied. Later pending operations can invalidate that match,
        // never grant a forward reference.
        let available = projection.captured.is_some();
        for operation in &self.operations[self.applied_operations.len()..] {
            projection.apply(operation)?;
        }
        Ok(available)
    }
}

#[derive(Clone, Copy)]
pub(in crate::substrate) struct PoolHistory<'a> {
    pub shards: &'a [SourceShardRef],
    pub journal: &'a [TransactionRecord],
}

fn invalid(message: &str) -> EngineError {
    EngineError::Validation(format!("Pinned library source: {message}"))
}

impl PoolHistory<'_> {
    pub(in crate::substrate) fn has_ambiguous_epoch(&self, id: ObjectId) -> bool {
        self.journal.iter().any(|tx| tx.transaction_kind == super::super::TransactionKind::Normal
            && tx.operations.iter().any(|op| matches!(op, Operation::DeletePoolLibraryObject {object_id,..} if *object_id == id)))
            || self.journal.iter().filter(|tx| tx.transaction_kind == super::super::TransactionKind::Normal)
                .flat_map(|tx| &tx.operations).filter(|op| matches!(op, Operation::CreatePoolLibraryObject {object_id,..} if *object_id == id)).count() > 1
    }

    fn has_restored_epoch(&self, id: ObjectId) -> bool {
        self.journal.iter().flat_map(|tx| &tx.operations).any(|op| matches!(op, Operation::DeletePoolLibraryObject {object_id,..} if *object_id == id))
    }
    pub(in crate::substrate) fn payload(
        &self,
        object: &DomainObject,
        revision: ObjectRevision,
    ) -> Result<Option<Value>, EngineError> {
        if self.has_ambiguous_epoch(object.object_id) {
            return Err(invalid(
                "definition deletion/restoration has no unique pinned revision epoch",
            ));
        }
        let path = &self
            .shards
            .iter()
            .find(|s| s.shard_id == object.source_shard_id && s.kind == SourceShardKind::Pool)
            .ok_or_else(|| invalid("pool source is unavailable"))?
            .relative_path;
        if self.has_restored_epoch(object.object_id) {
            return self.restored_payload(object, revision, path);
        }
        let mut captured = None;
        let mut counter = object.object_revision.0;
        let mut state: Option<Value> = None;
        let mut requested_run = false;
        for transaction in self.journal.iter().rev() {
            for op in transaction.operations.iter().rev() {
                match op {
                    Operation::SetPoolLibraryObject {
                        object_id,
                        relative_path,
                        object_kind,
                        previous_object,
                        object: next,
                    } if *object_id == object.object_id => {
                        if relative_path != path || object_kind != &object.kind {
                            return Err(invalid("historical pool source identity differs"));
                        }
                        super::super::pool_journal_ops::validate_pool_library_object(
                            *object_id,
                            relative_path,
                            object_kind,
                            previous_object,
                        )?;
                        super::super::pool_journal_ops::validate_pool_library_object(
                            *object_id,
                            relative_path,
                            object_kind,
                            next,
                        )?;
                        if state.as_ref().is_some_and(|value| value != next) {
                            return Err(invalid("historical pool postimage differs"));
                        }
                        if counter == revision.0 || requested_run {
                            capture(&mut captured, next)?;
                        }
                        counter = counter.checked_sub(1).ok_or_else(|| {
                            invalid("pool revision history does not match the resolved counter")
                        })?;
                        state = Some(previous_object.clone());
                        if counter == revision.0 {
                            capture(&mut captured, previous_object)?;
                        }
                        requested_run = false;
                    }
                    Operation::BumpObjectRevision { object_id }
                        if *object_id == object.object_id =>
                    {
                        if counter == revision.0 {
                            requested_run = true;
                        }
                        counter = counter.checked_sub(1).ok_or_else(|| {
                            invalid("pool bump history does not match the resolved counter")
                        })?;
                        if counter == revision.0 {
                            requested_run = true;
                        }
                        if requested_run && let Some(value) = &state {
                            capture(&mut captured, value)?;
                        }
                    }
                    Operation::CreatePoolLibraryObject {
                        object_id,
                        relative_path,
                        object_kind,
                        object: value,
                    } if *object_id == object.object_id => {
                        if relative_path != path || object_kind != &object.kind {
                            return Err(invalid("historical creation source differs"));
                        }
                        super::super::pool_journal_ops::validate_pool_library_object(
                            *object_id,
                            relative_path,
                            object_kind,
                            value,
                        )?;
                        if counter != super::super::object_revision_for(value).0
                            || state.as_ref().is_some_and(|old| old != value)
                        {
                            return Err(invalid(
                                "pool creation revision or payload differs from accepted history",
                            ));
                        }
                        if counter == revision.0 || requested_run {
                            capture(&mut captured, value)?;
                        }
                        return Ok(captured);
                    }
                    Operation::DeletePoolLibraryObject { object_id, .. }
                        if *object_id == object.object_id =>
                    {
                        return Err(invalid(
                            "definition deletion/restoration history has no unique retained revision basis",
                        ));
                    }
                    _ => {}
                }
            }
        }
        Ok(captured)
    }

    pub(in crate::substrate) fn definition_ref_matches(
        &self,
        objects: &BTreeMap<ObjectId, DomainObject>,
        id: ObjectId,
        revision: ObjectRevision,
        kind: &str,
    ) -> bool {
        if !matches!(kind, "symbols" | "parts" | "pin_pad_maps") {
            return false;
        }
        objects
            .get(&id)
            .filter(|o| o.domain == "pool" && o.kind == kind)
            .is_some_and(|o| {
                !self.has_ambiguous_epoch(id)
                    && ((o.object_revision == revision && !self.has_restored_epoch(id))
                        || self.payload(o, revision).ok().flatten().is_some())
            })
    }
}

fn capture(captured: &mut Option<Value>, value: &Value) -> Result<(), EngineError> {
    if captured.as_ref().is_some_and(|old| old != value) {
        return Err(invalid("ambiguous payload for pinned revision"));
    }
    *captured = Some(value.clone());
    Ok(())
}

impl DesignModel {
    /// Resolve an extant pool object's exact pinned revision in this Project.
    /// Missing or ambiguous historical evidence never falls back to latest.
    pub fn materialized_pool_object_revision(
        &self,
        id: ObjectId,
        kind: &str,
        revision: ObjectRevision,
    ) -> Result<Value, EngineError> {
        self.pool_object_revision_with_reader(id, kind, revision, &|shard| {
            self.materialized_source_shard_value_by_relative_path(&shard.relative_path)
        })
    }
    pub(in crate::substrate) fn pool_object_revision_with_reader(
        &self,
        id: ObjectId,
        kind: &str,
        revision: ObjectRevision,
        read: &impl Fn(&SourceShardRef) -> Result<Value, EngineError>,
    ) -> Result<Value, EngineError> {
        if self.journal_cursor.applied_transaction_count != self.journal.len() {
            return Err(invalid("journal tip is not the resolved source basis"));
        }
        let object = self
            .objects
            .get(&id)
            .filter(|o| o.domain == "pool" && o.kind == kind)
            .ok_or_else(|| invalid("object is missing or has a different kind"))?;
        let history = PoolHistory {
            shards: &self.source_shards,
            journal: &self.journal,
        };
        if history.has_ambiguous_epoch(id) {
            return Err(invalid(
                "definition deletion/restoration has no unique pinned revision epoch",
            ));
        }
        if history.has_restored_epoch(id) {
            return history
                .payload(object, revision)?
                .ok_or_else(|| invalid("authenticated restored revision is unavailable"));
        }
        if object.object_revision == revision {
            let shard = self
                .source_shards
                .iter()
                .find(|s| s.shard_id == object.source_shard_id && s.kind == SourceShardKind::Pool)
                .ok_or_else(|| invalid("pool source is unavailable"))?;
            let value = read(shard)?;
            super::super::pool_journal_ops::validate_pool_library_object(
                id,
                &shard.relative_path,
                kind,
                &value,
            )?;
            return Ok(value);
        }
        if !matches!(kind, "units" | "symbols" | "parts" | "pin_pad_maps") {
            return Err(invalid(
                "historical reads for this definition kind are unavailable",
            ));
        }
        PoolHistory {
            shards: &self.source_shards,
            journal: &self.journal,
        }
        .payload(object, revision)?
        .ok_or_else(|| invalid("authenticated pinned revision is unavailable"))
    }
}

#[cfg(test)]
#[path = "pool_history_tests.rs"]
mod tests;
