//! Authenticated payloads from the existing RevisionProjection/history owner.
use super::*;

pub(super) struct ProjectedReference {
    pub(super) available: bool,
    payload: Option<Value>,
}

impl PoolHistoryBasis<'_> {
    pub(in crate::substrate) fn projected_definition_value_with_reader(
        &self,
        staged: &BTreeMap<ObjectId, DomainObject>,
        id: ObjectId,
        revision: ObjectRevision,
        kind: &str,
        read: &impl Fn(&SourceShardRef) -> Result<Value, EngineError>,
    ) -> Result<Option<Value>, EngineError> {
        if !matches!(
            kind,
            "units"
                | "symbols"
                | "entities"
                | "parts"
                | "packages"
                | "footprints"
                | "padstacks"
                | "pin_pad_maps"
        ) {
            return Err(invalid("unsupported projected pool kind"));
        }
        Ok(self
            .projected_reference(staged, id, revision, kind, Some(read))?
            .payload)
    }
    pub(super) fn projected_reference(
        &self,
        staged: &BTreeMap<ObjectId, DomainObject>,
        id: ObjectId,
        revision: ObjectRevision,
        kind: &str,
        read: Option<&dyn Fn(&SourceShardRef) -> Result<Value, EngineError>>,
    ) -> Result<ProjectedReference, EngineError> {
        let Some(current) = staged
            .get(&id)
            .filter(|o| o.domain == "pool" && o.kind == kind)
        else {
            return Ok(ProjectedReference {
                available: false,
                payload: None,
            });
        };
        let history = PoolHistory {
            shards: &self.model.source_shards,
            journal: &self.model.journal,
        };
        if history.has_ambiguous_epoch(id)
            || self.operations.iter().any(|op| {
                matches!(op,
                Operation::DeletePoolLibraryObject { object_id, .. } if *object_id == id)
            })
        {
            return Ok(ProjectedReference {
                available: false,
                payload: None,
            });
        }
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
            let payload = if let Some(read) = read {
                let source = self
                    .model
                    .source_shards
                    .iter()
                    .find(|shard| {
                        shard.shard_id == current.source_shard_id
                            && shard.kind == SourceShardKind::Pool
                    })
                    .ok_or_else(|| invalid("accepted source shard is missing"))?;
                Some(read(source)?)
            } else {
                None
            };
            return Ok(ProjectedReference {
                available: true,
                payload,
            });
        }
        let (path, state, counter) = if let Some(before) = before {
            let path = &self
                .model
                .source_shards
                .iter()
                .find(|s| s.shard_id == before.source_shard_id && s.kind == SourceShardKind::Pool)
                .ok_or_else(|| invalid("accepted source shard is missing"))?
                .relative_path;
            let source = self
                .model
                .source_shards
                .iter()
                .find(|shard| {
                    shard.shard_id == before.source_shard_id && shard.kind == SourceShardKind::Pool
                })
                .ok_or_else(|| invalid("accepted source shard is missing"))?;
            let raw = if let Some(read) = read {
                read(source)?
            } else {
                self.model
                    .materialized_source_shard_value_by_relative_path(path)?
            };
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
                return Ok(ProjectedReference {
                    available: false,
                    payload: None,
                });
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
            return Ok(ProjectedReference {
                available: false,
                payload: None,
            });
        }
        // Availability is established only by accepted history or operations
        // already applied. Later pending operations can invalidate that match,
        // never grant a forward reference.
        let available = projection.captured.is_some();
        for operation in &self.operations[self.applied_operations.len()..] {
            projection.apply(operation)?;
        }
        Ok(ProjectedReference {
            available,
            payload: if available { projection.captured } else { None },
        })
    }
}
