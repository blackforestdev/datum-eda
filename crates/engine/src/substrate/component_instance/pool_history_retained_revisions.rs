//! Retained Part/Package/Symbol root revisions are checked before atomic publication.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::substrate) fn part_revision_target(
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
    (kind == "parts"
        || model
            .objects
            .get(&id)
            .is_some_and(|o| o.domain == "pool" && o.kind == "parts"))
    .then_some(id)
}

pub(in crate::substrate) fn symbol_revision_target(
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
    (kind == "symbols"
        || model
            .objects
            .get(&id)
            .is_some_and(|o| o.domain == "pool" && o.kind == "symbols"))
    .then_some(id)
}

impl PoolHistoryBasis<'_> {
    /// Compensation must not assign different payloads to retained root counters,
    /// including legacy Part references without electrical or role bindings.
    pub(in crate::substrate) fn preserve_retained_revisions(
        &self,
        staged: &DesignModel,
    ) -> Result<(), EngineError> {
        let packages: BTreeSet<_> = self
            .operations
            .iter()
            .filter_map(|op| package_revision_target(self.model, op))
            .collect();
        let parts: BTreeSet<_> = self
            .operations
            .iter()
            .filter_map(|op| part_revision_target(self.model, op))
            .collect();
        let symbols: BTreeSet<_> = self
            .operations
            .iter()
            .filter_map(|op| symbol_revision_target(self.model, op))
            .collect();
        if packages.is_empty() && parts.is_empty() && symbols.is_empty() {
            return Ok(());
        }
        let check = |id, revision, kind| {
            let affected = match kind {
                "parts" => &parts,
                "symbols" => &symbols,
                _ => &packages,
            };
            if affected.contains(&id)
                && !self.definition_ref_matches(&staged.objects, id, revision, kind)
            {
                Err(invalid(&format!(
                    "{kind} source edit or compensation would make a retained revision ambiguous"
                )))
            } else {
                Ok(())
            }
        };
        for ci in staged.component_instances.values() {
            for binding in ci.library_bindings.values() {
                let kind = match binding.binding_role {
                    LibraryBindingRole::Package => "packages",
                    LibraryBindingRole::Part => "parts",
                    LibraryBindingRole::Symbol => "symbols",
                    _ => continue,
                };
                check(
                    binding.target_object_id,
                    binding.pinned_object_revision,
                    kind,
                )?;
                for pin in &binding.local_override_refs {
                    check(pin.object_id, pin.object_revision, kind)?;
                }
            }
        }
        if !parts.is_empty() {
            // The resolved CI omits legacy part_ref's counter. Overlay the exact
            // pending Create/Set/Delete payloads on accepted canonical sources.
            let mut sources = BTreeMap::new();
            for shard in self
                .model
                .source_shards
                .iter()
                .filter(|s| s.kind == SourceShardKind::ComponentInstance)
            {
                let raw = self
                    .model
                    .materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
                let payload = raw["component_instance"].clone();
                if let Some(id) = payload["uuid"]
                    .as_str()
                    .and_then(|s| s.parse::<Uuid>().ok())
                {
                    if sources.insert(id, payload).is_some() {
                        return Err(invalid("ambiguous canonical CI source identity"));
                    }
                }
            }
            for op in self.operations {
                if let Some((id, value, delete)) = super::super::super::component_instance_journal_ops::component_instance_operation_write(op) {
                    if delete { sources.remove(&id); } else { sources.insert(id, value.clone()); }
                }
            }
            for (id, payload) in sources {
                let Some(part) = payload["part_ref"]["object_id"]
                    .as_str()
                    .and_then(|s| s.parse::<Uuid>().ok())
                    .filter(|id| parts.contains(id))
                else {
                    continue;
                };
                let ci: PersistedComponentInstance = serde_json::from_value(payload)?;
                let owner = staged
                    .objects
                    .get(&id)
                    .filter(|o| o.domain == "component_instance" && o.kind == "component_instance")
                    .ok_or_else(|| invalid("retained Part reference has no canonical CI owner"))?;
                if ci.uuid != id
                    || ci.object_revision != owner.object_revision
                    || staged
                        .component_instances
                        .get(&id)
                        .is_none_or(|c| c.authority != ComponentInstanceAuthority::Authored)
                    || !staged.source_shards.iter().any(|s| {
                        s.kind == SourceShardKind::ComponentInstance
                            && s.shard_id == owner.source_shard_id
                    })
                {
                    return Err(invalid(
                        "retained Part reference CI source identity differs",
                    ));
                }
                check(part, ci.part_ref.unwrap().object_revision, "parts")?;
            }
        }
        for shard in self.model.source_shards.iter().filter(|s| {
            s.kind == SourceShardKind::BoardRoot && (!parts.is_empty() || !packages.is_empty())
        }) {
            let mut board = self
                .model
                .materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
            let mut pass = super::super::super::board_list_journal_ops::BoardPass::new(&mut board);
            for op in self.operations {
                pass.apply(op)?;
            }
            if let Some(placements) = board.get("component_pads").and_then(Value::as_object) {
                for pad in placements.values().filter_map(Value::as_array).flatten() {
                    for (field, kind) in [
                        ("package", "packages"),
                        ("land_pattern", "packages"),
                        ("part", "parts"),
                    ] {
                        if let Some(pin) = pad
                            .get("source_binding")
                            .and_then(|b| b.get(field))
                            .filter(|p| !p.is_null())
                        {
                            let pin: super::super::super::component_pad_source::PinnedPadLibrarySource = serde_json::from_value(pin.clone())?;
                            if kind == "parts"
                                && (parts.contains(&pin.object_id) || parts.contains(&pin.owner_id))
                                && pin.object_id != pin.owner_id
                            {
                                return Err(invalid(
                                    "retained Part pin does not name its root owner",
                                ));
                            }
                            if pin.object_id == pin.owner_id {
                                check(pin.object_id, pin.object_revision, kind)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
