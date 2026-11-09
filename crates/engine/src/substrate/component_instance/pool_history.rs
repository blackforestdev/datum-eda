//! Read-only pool revisions authenticated by the accepted canonical journal.
//! No model replay, source writes or independently persisted history lives here.
use super::super::{DesignModel, Operation};
use super::*;
use serde_json::Value;
#[path = "pool_history_restore.rs"]
mod restore;
#[path = "pool_history_retained_revisions.rs"]
mod retained_revisions;
pub(in crate::substrate) use retained_revisions::part_revision_target;

#[derive(Clone, Copy)]
pub(in crate::substrate) struct PoolHistoryBasis<'a> {
    pub model: &'a DesignModel,
    pub operations: &'a [Operation],
    pub applied_operations: &'a [Operation],
}

impl PoolHistoryBasis<'_> {
    /// Qualify the actual precommit frame for descriptive Package edits only.
    /// The accepted journal is never extended to manufacture a source revision.
    pub(in crate::substrate) fn package_metadata_frame_matches(
        &self,
        staged: &DesignModel,
    ) -> Result<bool, EngineError> {
        // Ordinary board/schematic edits do not need a Package source basis.
        if !self.operations.iter().any(|op| matches!(op, Operation::SetPoolLibraryObject { object_kind, .. } if object_kind == "packages")) {
            return Ok(false);
        }
        if self.applied_operations != self.operations
            || staged.project.project_id != self.model.project.project_id
            || staged.model_revision != self.model.model_revision
            || staged.journal != self.model.journal
            || staged.journal_cursor != self.model.journal_cursor
            || staged.source_shards != self.model.source_shards
            || staged.component_instances != self.model.component_instances
            || staged.electrical_identities != self.model.electrical_identities
            || staged.objects.len() != self.model.objects.len()
        {
            return Ok(false);
        }
        let mut advances = BTreeMap::<ObjectId, u64>::new();
        for op in self.operations {
            let Operation::SetPoolLibraryObject {
                object_id,
                relative_path,
                object_kind,
                previous_object,
                object,
            } = op
            else {
                if matches!(op, Operation::GuardObjectRevision { .. }) {
                    continue;
                }
                return Ok(false);
            };
            if object_kind != "packages" {
                return Ok(false);
            }
            let Some(owner) = self
                .model
                .objects
                .get(object_id)
                .filter(|o| o.domain == "pool" && o.kind == "packages")
            else {
                return Ok(false);
            };
            let Some(shard) = self.model.source_shards.iter().find(|s| {
                s.shard_id == owner.source_shard_id
                    && s.kind == SourceShardKind::Pool
                    && s.relative_path == *relative_path
            }) else {
                return Ok(false);
            };
            // A single edit per root keeps the accepted preimage explicit.
            if advances.insert(*object_id, 1).is_some() {
                return Ok(false);
            }
            if self
                .model
                .materialized_source_shard_value_by_relative_path(&shard.relative_path)?
                != *previous_object
            {
                return Err(invalid("Package metadata precommit preimage differs"));
            }
            if serde_json::from_value::<crate::pool::Package>(object.clone()).is_err() {
                return Ok(false);
            }
            let mut old = previous_object.clone();
            let mut new = object.clone();
            for key in ["name", "package_family", "package_code", "mounting_type"] {
                let (Some(old), Some(new)) = (old.as_object_mut(), new.as_object_mut()) else {
                    return Ok(false);
                };
                old.remove(key);
                new.remove(key);
            }
            if old != new
                || !self.definition_ref_matches(
                    &staged.objects,
                    *object_id,
                    owner.object_revision,
                    "packages",
                )
            {
                return Ok(false);
            }
        }
        if advances.is_empty() {
            return Ok(false);
        }
        for (id, old) in &self.model.objects {
            let Some(new) = staged.objects.get(id) else {
                return Ok(false);
            };
            let mut expected = old.clone();
            expected.object_revision.0 = expected
                .object_revision
                .0
                .checked_add(advances.get(id).copied().unwrap_or(0))
                .ok_or_else(|| invalid("Package metadata revision overflow"))?;
            if *new != expected {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(in crate::substrate) fn definition_ref_matches(
        &self,
        staged: &BTreeMap<ObjectId, DomainObject>,
        id: ObjectId,
        revision: ObjectRevision,
        kind: &str,
    ) -> bool {
        if !matches!(kind, "symbols" | "parts" | "packages" | "pin_pad_maps") {
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

/// Existing Package roots whose revision/epoch this pending batch can change.
pub(in crate::substrate) fn package_revision_target(
    model: &DesignModel,
    op: &Operation,
) -> Option<ObjectId> {
    let (id, kind) = match op {
        Operation::SetPoolLibraryObject {
            object_id,
            object_kind,
            ..
        }
        | Operation::DeletePoolLibraryObject {
            object_id,
            object_kind,
            ..
        } => (*object_id, object_kind.as_str()),
        Operation::BumpObjectRevision { object_id } => (*object_id, ""),
        _ => return None,
    };
    (kind == "packages"
        || model
            .objects
            .get(&id)
            .is_some_and(|o| o.domain == "pool" && o.kind == "packages"))
    .then_some(id)
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
        if !matches!(kind, "symbols" | "parts" | "packages" | "pin_pad_maps") {
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
        if !matches!(
            kind,
            "units" | "symbols" | "parts" | "packages" | "pin_pad_maps"
        ) {
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

#[cfg(test)]
mod package_frame_tests {
    use super::*;
    use crate::api::native_write::{self as native, WriteProvenance, commit_prepared, library::*};
    use crate::substrate::{CommitSource, ProjectResolver};
    use serde_json::json;

    #[test]
    fn package_metadata_frame_requires_exact_accepted_source_prefix_and_candidate() {
        let root = std::env::temp_dir().join(format!("datum_package_frame_{}", Uuid::new_v4()));
        native::genesis::bootstrap_native_project(
            &root,
            native::genesis::GenesisSpec {
                project_name: "Package pending source".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let provenance = || {
            WriteProvenance::new(
                "package-frame-test",
                CommitSource::Test,
                "Qualify actual precommit Package source",
            )
        };
        let mut before = ProjectResolver::new(&root).resolve().unwrap();
        let id = Uuid::new_v4();
        let terminal = Uuid::new_v4();
        let target = PoolLibraryObjectTarget::new("pool", "packages", id);
        let mut raw = pool_package_payload(id, "Original", json!({}));
        raw["terminals"] = json!({terminal.to_string():{"uuid":terminal,"name":"1"}});
        let prepared = build_pool_library_write(
            &before,
            provenance(),
            Some("pool"),
            vec![PoolLibraryOperationSpec::Create {
                target: target.clone(),
                object: raw.clone(),
            }],
        )
        .unwrap();
        commit_prepared(&mut before, &root, prepared).unwrap();
        raw["name"] = json!("Corrected");
        let prepared = build_pool_library_write(
            &before,
            provenance(),
            Some("pool"),
            vec![PoolLibraryOperationSpec::Set {
                target,
                object: raw,
            }],
        )
        .unwrap();
        let operations = &prepared.batch.operations;
        let mut candidate = before.clone();
        crate::substrate::operation_application_batch::apply_operations(
            &mut candidate,
            operations,
            &mut crate::substrate::CommitDiff::default(),
        )
        .unwrap();
        let basis = PoolHistoryBasis {
            model: &before,
            operations,
            applied_operations: operations,
        };
        assert!(basis.package_metadata_frame_matches(&candidate).unwrap());
        // Fault clones exercise the private guard only, never durable admission.
        let mut bad = candidate.clone();
        bad.project.project_id = Uuid::new_v4();
        assert!(!basis.package_metadata_frame_matches(&bad).unwrap());
        let mut bad = candidate.clone();
        bad.journal_cursor.applied_transaction_count += 1;
        assert!(!basis.package_metadata_frame_matches(&bad).unwrap());
        let mut bad = candidate.clone();
        bad.journal.clear();
        assert!(!basis.package_metadata_frame_matches(&bad).unwrap());
        let mut bad = candidate.clone();
        bad.objects.get_mut(&terminal).unwrap().object_revision.0 += 1;
        assert!(!basis.package_metadata_frame_matches(&bad).unwrap());
        let mut bad = candidate.clone();
        bad.objects.get_mut(&id).unwrap().object_revision.0 += 1;
        assert!(!basis.package_metadata_frame_matches(&bad).unwrap());
        let unapplied = PoolHistoryBasis {
            model: &before,
            operations,
            applied_operations: &[],
        };
        assert!(
            !unapplied
                .package_metadata_frame_matches(&candidate)
                .unwrap()
        );
        for fault in 0..4 {
            let mut bad = operations.clone();
            for operation in &mut bad {
                if let Operation::SetPoolLibraryObject {
                    previous_object,
                    object,
                    object_kind,
                    ..
                } = operation
                {
                    match fault {
                        0 => previous_object["name"] = json!("Foreign preimage"),
                        1 => object["body_dimensions"] = json!({"x_nm":5000000}),
                        2 => *object_kind = "parts".into(),
                        _ => object["name"] = json!(17),
                    }
                }
            }
            let bad = PoolHistoryBasis {
                model: &before,
                operations: &bad,
                applied_operations: &bad,
            };
            if fault == 0 {
                assert!(
                    bad.package_metadata_frame_matches(&candidate)
                        .unwrap_err()
                        .to_string()
                        .contains("preimage differs")
                );
            } else {
                assert!(!bad.package_metadata_frame_matches(&candidate).unwrap());
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
