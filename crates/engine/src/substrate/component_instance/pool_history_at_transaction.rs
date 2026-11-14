//! Selective source lookup at an authenticated declaration, without model replay.
use super::*;

impl DesignModel {
    pub(in crate::substrate) fn pool_source_at_transaction(
        &self,
        id: ObjectId,
        kind: &str,
        transaction: usize,
    ) -> Result<(Value, ObjectRevision), EngineError> {
        if self.journal_cursor.applied_transaction_count != self.journal.len()
            || transaction >= self.journal.len()
        {
            return Err(invalid("declaration is outside the current journal basis"));
        }
        let object = self
            .objects
            .get(&id)
            .filter(|o| o.domain == "pool" && o.kind == kind)
            .ok_or_else(|| invalid("declared source is missing or has another kind"))?;
        let shard = self
            .source_shards
            .iter()
            .find(|s| s.shard_id == object.source_shard_id && s.kind == SourceShardKind::Pool)
            .ok_or_else(|| invalid("declared source ownership is unavailable"))?;
        let current =
            self.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        crate::substrate::pool_journal_ops::validate_pool_library_object(
            id,
            &shard.relative_path,
            kind,
            &current,
        )?;
        let history = PoolHistory {
            shards: &self.source_shards,
            journal: &self.journal,
        };
        if history.has_ambiguous_epoch(id) || history.has_restored_epoch(id) {
            return Err(invalid(
                "declared source crosses an unsupported deletion/restoration epoch",
            ));
        }
        let mut counter = object.object_revision.0;
        let mut changed = false;
        for tx in &self.journal[transaction + 1..] {
            for op in &tx.operations {
                match op {
                    Operation::SetPoolLibraryObject {
                        object_id,
                        relative_path,
                        object_kind,
                        ..
                    } if *object_id == id => {
                        if relative_path != &shard.relative_path || object_kind != kind {
                            return Err(invalid("declared source history ownership differs"));
                        }
                        counter = counter
                            .checked_sub(1)
                            .ok_or_else(|| invalid("declared source counter underflow"))?;
                        changed = true;
                    }
                    Operation::BumpObjectRevision { object_id } if *object_id == id => {
                        counter = counter
                            .checked_sub(1)
                            .ok_or_else(|| invalid("declared source counter underflow"))?;
                    }
                    Operation::CreatePoolLibraryObject { object_id, .. }
                    | Operation::DeletePoolLibraryObject { object_id, .. }
                        if *object_id == id =>
                    {
                        return Err(invalid("source did not retain its declaration epoch"));
                    }
                    _ => {}
                }
            }
        }
        if let Some(recorded) = history.payload(object, object.object_revision)? {
            if recorded != current {
                return Err(invalid(
                    "declared source current postimage differs from its journal",
                ));
            }
        }
        let revision = ObjectRevision(counter);
        let value = if changed {
            history
                .payload(object, revision)?
                .ok_or_else(|| invalid("authenticated declaration source is unavailable"))?
        } else {
            current
        };
        crate::substrate::pool_journal_ops::validate_pool_library_object(
            id,
            &shard.relative_path,
            kind,
            &value,
        )?;
        Ok((value, revision))
    }
}
