//! Journal-owned certified source values for one actual evaluated transaction.
//! The frame grants no publication, reviewer, standards or source-write authority.
use super::{
    DesignModel, EngineError, ObjectRevision, RevisionedRef, SourceShardKind, SourceShardRef,
    journal::StagedShardWrite,
    source_shard::{validate_source_shard_ownership_path, validate_source_shard_schema_version},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub(in crate::substrate) struct SourceFrameLimits {
    pub(in crate::substrate) per_source: usize,
    pub(in crate::substrate) total_bytes: usize,
    pub(in crate::substrate) max_sources: usize,
}

pub(in crate::substrate) struct VerifiedPendingSourceFrame {
    accepted: Arc<DesignModel>,
    candidate: Arc<DesignModel>,
    values: BTreeMap<Uuid, Value>,
    preimages: BTreeMap<Uuid, Value>,
    heads: BTreeMap<Uuid, SourceShardRef>,
    component_references: BTreeMap<(Uuid, u64), RevisionedRef>,
}

fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("pending source frame: {reason}"))
}

impl VerifiedPendingSourceFrame {
    /// Called by commit after canonical application/report refresh, before
    /// integrity publication/promotion. No model path, revision or cursor is changed.
    pub(in crate::substrate) fn capture_evaluated(
        root: &Path,
        lease: &crate::revision::ProjectWriteLease,
        accepted: Arc<DesignModel>,
        candidate: Arc<DesignModel>,
        writes: &[StagedShardWrite],
        requested: &BTreeSet<Uuid>,
        limits: SourceFrameLimits,
    ) -> Result<Arc<Self>, EngineError> {
        lease.validate(root)?;
        if [limits.per_source, limits.total_bytes, limits.max_sources].contains(&0) {
            return Err(invalid("explicit positive limits required"));
        }
        if requested.len() > limits.max_sources || writes.len() > limits.max_sources {
            return Err(invalid("source inventory exceeds configured frame policy"));
        }
        let count = accepted.journal.len();
        if accepted.journal_cursor.applied_transaction_count != count
            || candidate.project.project_id != accepted.project.project_id
            || candidate.journal_cursor != accepted.journal_cursor
            || candidate.journal.len() != count + 1
            || candidate.journal[..count] != accepted.journal[..]
        {
            return Err(invalid(
                "not one canonical pending transaction over the accepted cursor",
            ));
        }
        let transaction = &candidate.journal[count];
        if transaction.before_model_revision != accepted.model_revision
            || transaction.after_model_revision != candidate.model_revision
            || super::compute_model_revision(
                &candidate.project.project_id,
                &candidate.source_shards,
                &candidate.objects,
            ) != candidate.model_revision
        {
            return Err(invalid("canonical pending revision differs"));
        }
        let mut staged = BTreeMap::new();
        for write in writes {
            validate_source_shard_ownership_path(&write.kind, &write.relative_path)?;
            if write.destination != root.join(&write.relative_path)
                || staged.insert(write.relative_path.as_str(), write).is_some()
            {
                return Err(invalid("duplicate or foreign staged destination"));
            }
        }
        let mut heads = BTreeMap::new();
        let mut paths = BTreeSet::new();
        for head in candidate
            .source_shards
            .iter()
            .filter(|head| requested.contains(&head.shard_id))
        {
            if head.kind == SourceShardKind::LibraryContent
                || head.path != root.join(&head.relative_path)
                || !paths.insert(head.relative_path.clone())
                || heads.insert(head.shard_id, head.clone()).is_some()
            {
                return Err(invalid(
                    "requested source has a duplicate, foreign or content-only head",
                ));
            }
            validate_source_shard_ownership_path(&head.kind, &head.relative_path)?;
        }
        if heads.len() != requested.len() {
            return Err(invalid("requested source head is missing"));
        }
        let mut values = BTreeMap::new();
        let mut preimages = BTreeMap::new();
        let mut used = 0usize;
        for (id, head) in &heads {
            let previous: Vec<_> = accepted
                .source_shards
                .iter()
                .filter(|before| before.shard_id == *id)
                .collect();
            if previous.len() > 1 {
                return Err(invalid("accepted source identity is duplicated"));
            }
            if let Some(before) = previous.first() {
                if before.kind != head.kind
                    || before.relative_path != head.relative_path
                    || before.path != root.join(&before.relative_path)
                {
                    return Err(invalid("source identity changed its owning kind or path"));
                }
                let raw = read_charge(root, &before.relative_path, limits, &mut used)?;
                // Required owning extraction: existing capture hash/replay logic
                // accepts these bounded bytes; no second private parser/replay.
                let value =
                    super::journal::capture_shard_value_from_bytes(&accepted, before, &raw)?;
                preimages.insert(*id, value);
            }
            let value = if let Some(write) = staged.get(head.relative_path.as_str()) {
                if write.delete
                    || write.kind != head.kind
                    || write.content_hash != head.content_hash
                    || write.schema_version != head.schema_version
                {
                    return Err(invalid(
                        "staged source kind, hash or schema differs from evaluated head",
                    ));
                }
                let stage_relative = format!(
                    ".datum/stage/{}/{}",
                    transaction.batch_id, head.relative_path
                );
                if write.staged.as_ref() != Some(&root.join(&stage_relative)) {
                    return Err(invalid("stage path is not owned by this actual batch"));
                }
                let bytes = read_charge(root, &stage_relative, limits, &mut used)?;
                if super::sha256_hex(&bytes) != head.content_hash {
                    return Err(invalid("staged source bytes differ from evaluated hash"));
                }
                serde_json::from_slice::<Value>(&bytes)?
            } else {
                let [before] = previous.as_slice() else {
                    return Err(invalid("new source has no staged postimage"));
                };
                if *before != head {
                    return Err(invalid("changed source has no staged postimage"));
                }
                preimages
                    .get(id)
                    .ok_or_else(|| invalid("certified unchanged source missing"))?
                    .clone()
            };
            let schema = value.get("schema_version").and_then(Value::as_u64);
            validate_source_shard_schema_version(&head.kind, &head.relative_path, schema)?;
            if schema != head.schema_version {
                return Err(invalid("source schema differs from head"));
            }
            if super::canonical_json_hash(&value)? != head.content_hash
                && !previous.first().is_some_and(|before| {
                    *before == head && !staged.contains_key(head.relative_path.as_str())
                })
            {
                return Err(invalid("source value is not the certified postimage"));
            }
            values.insert(*id, value);
        }
        // Historical metadata qualification runs while the accepted files are
        // still lease-protected. Retain it only for a completely unchanged CI;
        // library adoption cannot silently promote an old Board reference.
        let mut component_references = BTreeMap::new();
        for head in heads
            .values()
            .filter(|s| s.kind == SourceShardKind::BoardRoot)
        {
            if let Some(placements) = values[&head.shard_id]
                .get("component_pads")
                .and_then(Value::as_object)
            {
                for raw in placements.values().filter_map(Value::as_array).flatten() {
                    let Some(reference) = raw.pointer("/source_binding/component_instance") else {
                        continue;
                    };
                    if reference.is_null() {
                        continue;
                    }
                    let reference: RevisionedRef = serde_json::from_value(reference.clone())?;
                    let key = (reference.object_id, reference.object_revision.0);
                    if component_references.contains_key(&key) {
                        continue;
                    }
                    if accepted.component_instances.get(&reference.object_id)
                        == candidate.component_instances.get(&reference.object_id)
                        && let Ok(current) = accepted.metadata_component_reference(&reference)
                    {
                        component_references.insert(key, current);
                    }
                }
            }
        }
        lease.validate(root)?;
        Ok(Arc::new(Self {
            accepted,
            candidate,
            values,
            preimages,
            heads,
            component_references,
        }))
    }

    pub(in crate::substrate) fn candidate(&self) -> &Arc<DesignModel> {
        &self.candidate
    }
    pub(in crate::substrate) fn accepted(&self) -> &Arc<DesignModel> {
        &self.accepted
    }
    pub(in crate::substrate) fn heads(&self) -> &BTreeMap<Uuid, SourceShardRef> {
        &self.heads
    }
    pub(in crate::substrate) fn value(&self, head: &SourceShardRef) -> Result<Value, EngineError> {
        if self.heads.get(&head.shard_id) != Some(head) {
            return Err(invalid("read request differs from certified source head"));
        }
        self.values
            .get(&head.shard_id)
            .cloned()
            .ok_or_else(|| invalid("source was not captured"))
    }
    pub(in crate::substrate) fn values(&self) -> &BTreeMap<Uuid, Value> {
        &self.values
    }
    pub(in crate::substrate) fn accepted_value(
        &self,
        head: &SourceShardRef,
    ) -> Result<Value, EngineError> {
        if !self
            .accepted
            .source_shards
            .iter()
            .any(|before| before == head)
        {
            return Err(invalid("read request differs from accepted source"));
        }
        self.preimages
            .get(&head.shard_id)
            .cloned()
            .ok_or_else(|| invalid("accepted source was not captured"))
    }
    pub(in crate::substrate) fn component_reference(
        &self,
        recorded: &RevisionedRef,
    ) -> Result<RevisionedRef, EngineError> {
        if self
            .candidate
            .component_instances
            .get(&recorded.object_id)
            .is_some_and(|ci| ci.object_revision == recorded.object_revision)
        {
            return Ok(recorded.clone());
        }
        self.component_references
            .get(&(recorded.object_id, recorded.object_revision.0))
            .cloned()
            .ok_or_else(|| invalid("Board binding requires an explicit final component reference"))
    }
    pub(in crate::substrate) fn pinned_definition(
        &self,
        id: Uuid,
        kind: &str,
        revision: ObjectRevision,
    ) -> Result<Value, EngineError> {
        let operations = &self
            .candidate
            .journal
            .last()
            .ok_or_else(|| invalid("actual pending transaction missing"))?
            .operations;
        let basis = super::component_instance::pool_history::PoolHistoryBasis {
            model: &self.accepted,
            operations,
            applied_operations: operations,
        };
        basis.projected_definition_value_with_reader(&self.candidate.objects,id,revision,kind,&|head|self.accepted_value(head))?
            .ok_or_else(||invalid("requested pin is not authenticated by accepted history and actual applied operations"))
    }
}

fn read_charge(
    root: &Path,
    relative: &str,
    limits: SourceFrameLimits,
    used: &mut usize,
) -> Result<Vec<u8>, EngineError> {
    let remaining = limits
        .total_bytes
        .checked_sub(*used)
        .ok_or_else(|| invalid("raw read budget exhausted"))?;
    let bytes = super::source_shard::io::read_regular_file(
        root,
        relative,
        limits.per_source.min(remaining),
    )?;
    *used = used
        .checked_add(bytes.capacity())
        .ok_or_else(|| invalid("raw read size overflow"))?;
    if *used > limits.total_bytes {
        return Err(invalid("raw read capacity exceeds frame policy"));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::native_write::{self as native, WriteProvenance, library::*};
    use crate::substrate::{CommitSource, OperationBatch, ProjectResolver};

    fn provenance() -> WriteProvenance {
        WriteProvenance::new(
            "frame-test",
            CommitSource::Test,
            "Create exact source frame",
        )
    }
    fn fixture() -> (std::path::PathBuf, DesignModel, native::PreparedWrite, Uuid) {
        let root = std::env::temp_dir().join(format!("datum_pending_source_{}", Uuid::new_v4()));
        native::genesis::bootstrap_native_project(
            &root,
            native::genesis::GenesisSpec {
                project_name: "Pending source control".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let accepted = ProjectResolver::new(&root).resolve().unwrap();
        let id = Uuid::new_v4();
        let prepared = build_pool_library_write(
            &accepted,
            provenance(),
            Some("pool"),
            vec![PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "units", id),
                object: pool_unit_payload(id, "Before", ""),
            }],
        )
        .unwrap();
        (root, accepted, prepared, id)
    }
    fn evaluate(
        root: &Path,
        accepted: &DesignModel,
        batch: &OperationBatch,
    ) -> (DesignModel, Vec<StagedShardWrite>) {
        let writes =
            crate::substrate::journal::stage_operation_shard_writes(root, accepted, batch).unwrap();
        let mut candidate = accepted.clone();
        crate::substrate::journal::update_staged_source_hashes(
            &mut candidate.source_shards,
            &writes,
        )
        .unwrap();
        crate::substrate::electrical_transaction::retain_preimage_hashes(accepted, &mut candidate);
        let mut report = candidate
            .commit_without_direct_policy(batch.clone())
            .unwrap();
        crate::substrate::journal::update_staged_source_hashes(
            &mut candidate.source_shards,
            &writes,
        )
        .unwrap();
        crate::substrate::commit::refresh_journaled_report_revision(
            &mut candidate,
            &mut report,
            None,
        )
        .unwrap();
        (candidate, writes)
    }
    fn capture(
        root: &Path,
        lease: &crate::revision::ProjectWriteLease,
        accepted: &DesignModel,
        candidate: &DesignModel,
        writes: &[StagedShardWrite],
        id: Uuid,
    ) -> Result<Arc<VerifiedPendingSourceFrame>, EngineError> {
        // Test limits derive from this fixture's actual stage sizes. They are
        // never a service configuration or a product performance threshold.
        let total = writes
            .iter()
            .filter_map(|w| w.staged.as_ref())
            .map(|path| std::fs::metadata(path).unwrap().len() as usize)
            .sum::<usize>();
        VerifiedPendingSourceFrame::capture_evaluated(
            root,
            lease,
            Arc::new(accepted.clone()),
            Arc::new(candidate.clone()),
            writes,
            &BTreeSet::from([candidate.objects[&id].source_shard_id]),
            SourceFrameLimits {
                per_source: total,
                total_bytes: total,
                max_sources: writes.len(),
            },
        )
    }

    #[test]
    fn actual_pending_new_source_and_held_history_survive_later_publication() {
        let (root, mut accepted, prepared, id) = fixture();
        let lease = crate::revision::ProjectWriteLease::acquire(&root).unwrap();
        let (candidate, writes) = evaluate(&root, &accepted, &prepared.batch);
        let frame = capture(&root, &lease, &accepted, &candidate, &writes, id).unwrap();
        assert_eq!(candidate.journal_cursor, accepted.journal_cursor);
        assert_eq!(candidate.journal.len(), accepted.journal.len() + 1);
        let head = frame.heads().values().next().unwrap();
        assert!(!head.path.exists());
        let captured = frame.value(head).unwrap();
        assert_eq!(captured["name"], "Before");
        assert_eq!(
            frame
                .pinned_definition(id, "units", ObjectRevision(0))
                .unwrap(),
            captured
        );
        drop(lease);
        // Publish via the ordinary native/journal route, then make a real edit.
        native::commit_prepared(&mut accepted, &root, prepared).unwrap();
        let changed = build_pool_library_write(
            &accepted,
            provenance(),
            None,
            vec![PoolLibraryOperationSpec::Set {
                target: PoolLibraryObjectTarget::new("pool", "units", id),
                object: pool_unit_payload(id, "After", ""),
            }],
        )
        .unwrap();
        native::commit_prepared(&mut accepted, &root, changed).unwrap();
        assert_eq!(frame.value(head).unwrap(), captured);
        assert_eq!(
            frame
                .pinned_definition(id, "units", ObjectRevision(0))
                .unwrap(),
            captured
        );
        assert_eq!(
            accepted
                .materialized_source_shard_value_by_relative_path(&head.relative_path)
                .unwrap()["name"],
            "After"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_frame_refuses_tampered_stage_and_published_cursor_faults() {
        let (root, accepted, prepared, id) = fixture();
        let lease = crate::revision::ProjectWriteLease::acquire(&root).unwrap();
        let (candidate, writes) = evaluate(&root, &accepted, &prepared.batch);
        assert!(capture(&root, &lease, &accepted, &candidate, &writes, id).is_ok());
        let path = writes
            .iter()
            .find(|write| write.kind == SourceShardKind::Pool)
            .unwrap()
            .staged
            .as_ref()
            .unwrap();
        let bytes = std::fs::read(path).unwrap();
        std::fs::write(path, b"{}\n").unwrap();
        assert!(capture(&root, &lease, &accepted, &candidate, &writes, id).is_err());
        std::fs::write(path, bytes).unwrap();
        let mut fault = candidate.clone();
        // Negative injection only: a published watermark is never accepted as
        // proof of a pending transaction or used to manufacture a success.
        fault.journal_cursor.applied_transaction_count = fault.journal.len();
        assert!(capture(&root, &lease, &accepted, &fault, &writes, id).is_err());
        assert!(capture(&root, &lease, &accepted, &candidate, &writes, id).is_ok());
        drop(lease);
        std::fs::remove_dir_all(root).unwrap();
    }
}
