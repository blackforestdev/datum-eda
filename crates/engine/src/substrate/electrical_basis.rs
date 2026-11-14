//! Canonical materialized source basis for identity validation. Object kind strings
//! are resolver index hints, not source class or hierarchy authority.
use super::{DesignModel, ElectricalOccurrence, EngineError, Operation, SourceShardKind};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
mod class_index;
use uuid::Uuid;

pub(super) struct ElectricalBasis<'basis> {
    accepted_library_basis: Option<&'basis DesignModel>,
    pending_map_changes: BTreeSet<Uuid>,
    values: BTreeMap<Uuid, Value>,
    classes: class_index::Index,
    query_pads: BTreeMap<Uuid, BTreeMap<Uuid, crate::board::query_pad::QueryPadGeometry>>,
    part_statuses: Option<(
        Uuid,
        super::ModelRevision,
        BTreeMap<
            Uuid,
            BTreeMap<Uuid, super::component_pad_source::ComponentPadMaterializationStatus>,
        >,
    )>,
}
impl ElectricalBasis<'static> {
    pub(super) fn new(model: &DesignModel, operations: &[Operation]) -> Result<Self, EngineError> {
        Self::with_reader(model, operations, &|shard| {
            super::journal::materialized_shard_value(model, shard)
        })
    }
    pub(super) fn with_reader(
        model: &DesignModel,
        operations: &[Operation],
        read: &impl Fn(&super::SourceShardRef) -> Result<Value, EngineError>,
    ) -> Result<Self, EngineError> {
        Self::capture(model, operations, read, None, None, None)
    }
}
impl<'basis> ElectricalBasis<'basis> {
    pub(super) fn with_package_basis(
        model: &DesignModel,
        operations: &[Operation],
        accepted: &DesignModel,
        preview: Option<&super::component_instance::metadata_history::CanonicalMetadataPreview<'_>>,
    ) -> Result<Self, EngineError> {
        let history = super::component_instance::pool_history::PoolHistoryBasis {
            model: accepted,
            operations,
            applied_operations: operations,
        };
        let package_basis = history
            .package_metadata_frame_matches(model)?
            .then_some(accepted);
        let metadata_basis = if let Some(proof) = preview {
            Some(proof.frame(model, operations)?)
        } else {
            history.metadata_frame(model)?
        };
        Self::capture(
            model,
            operations,
            &|shard| super::journal::materialized_shard_value(model, shard),
            None,
            package_basis,
            metadata_basis,
        )
    }
    pub(super) fn with_part_basis(
        model: &DesignModel,
        part_basis: &'basis DesignModel,
        operations: &[Operation],
    ) -> Result<Self, EngineError> {
        Self::capture(
            model,
            operations,
            &|shard| super::journal::materialized_shard_value(model, shard),
            Some(part_basis),
            None,
            None,
        )
    }
    fn capture(
        model: &DesignModel,
        operations: &[Operation],
        read: &impl Fn(&super::SourceShardRef) -> Result<Value, EngineError>,
        part_basis: Option<&'basis DesignModel>,
        package_basis: Option<&DesignModel>,
        metadata_basis: Option<super::component_instance::metadata_history::MetadataFrame<'_>>,
    ) -> Result<Self, EngineError> {
        if let Some(accepted) = part_basis {
            super::component_pad_source::validate_library_basis(model, accepted, operations)?;
        }
        let mut values = BTreeMap::new();
        let mut pending_map_changes = BTreeSet::new();
        let mut query_pads = BTreeMap::new();
        let mut part_statuses = Some({
            (
                model.project.project_id,
                model.model_revision.clone(),
                BTreeMap::new(),
            )
        });
        for shard in &model.source_shards {
            if !matches!(
                shard.kind,
                SourceShardKind::SchematicRoot
                    | SourceShardKind::SchematicSheet
                    | SourceShardKind::SchematicDefinition
                    | SourceShardKind::BoardRoot
                    | SourceShardKind::Pool
            ) {
                continue;
            }
            let pending_create = operations.iter().find_map(|op| match op {
                Operation::CreatePoolLibraryObject {
                    relative_path,
                    object,
                    ..
                } if shard.kind == SourceShardKind::Pool
                    && shard.relative_path == *relative_path =>
                {
                    Some(object.clone())
                }
                Operation::CreatePoolPadstack {
                    relative_path,
                    padstack,
                    ..
                } if shard.kind == SourceShardKind::Pool
                    && shard.relative_path == *relative_path =>
                {
                    Some(padstack.clone())
                }
                Operation::CreatePoolPackage {
                    relative_path,
                    package,
                    ..
                } if shard.kind == SourceShardKind::Pool
                    && shard.relative_path == *relative_path =>
                {
                    Some(package.clone())
                }
                Operation::CreateSchematicDefinition {
                    relative_path,
                    definition,
                    ..
                } if shard.kind == SourceShardKind::SchematicDefinition
                    && shard.relative_path == format!("schematic/{relative_path}") =>
                {
                    Some(definition.clone())
                }
                Operation::CreateSchematicSheet {
                    relative_path,
                    sheet,
                    ..
                } if shard.kind == SourceShardKind::SchematicSheet
                    && shard.relative_path == format!("schematic/{relative_path}") =>
                {
                    Some(sheet.clone())
                }
                _ => None,
            });
            let pending_creation = pending_create.is_some();
            let mut value = match pending_create {
                Some(value) => value,
                None => read(shard)?,
            };
            // Compare this source only within capture. Retain no second payload
            // cache: changed pending Map roots are unavailable as declared reads.
            let map_before = (shard.kind == SourceShardKind::Pool)
                .then(|| {
                    value
                        .get("uuid")
                        .and_then(Value::as_str)
                        .and_then(|v| v.parse::<Uuid>().ok())
                })
                .flatten()
                .filter(|id| {
                    model.objects.get(id).is_some_and(|o| {
                        o.domain == "pool"
                            && o.kind == "pin_pad_maps"
                            && o.source_shard_id == shard.shard_id
                    })
                })
                .map(|id| (id, value.clone()));
            {
                let mut board_pass = super::board_list_journal_ops::BoardPass::new(&mut value);
                for operation in operations {
                    match shard.kind {
                        SourceShardKind::SchematicRoot => {
                            super::schematic_root_journal_ops::apply_schematic_root_operation(
                                board_pass.board_mut(),
                                operation,
                            )?;
                        }
                        SourceShardKind::SchematicSheet => {
                            super::schematic_sheet_journal_ops::apply_schematic_sheet_operation(
                                board_pass.board_mut(),
                                operation,
                            )?;
                        }
                        SourceShardKind::BoardRoot => {
                            board_pass.apply(operation)?;
                        }
                        SourceShardKind::Pool => {
                            let path = match operation {
                                Operation::CreatePoolPackage { relative_path, .. }
                                | Operation::CreatePoolPadstack { relative_path, .. }
                                | Operation::CreatePoolLibraryObject { relative_path, .. }
                                | Operation::SetPoolLibraryObject { relative_path, .. }
                                | Operation::DeletePoolPackage { relative_path, .. }
                                | Operation::DeletePoolPadstack { relative_path, .. }
                                | Operation::DeletePoolLibraryObject { relative_path, .. }
                                | Operation::AttachPoolPartModel { relative_path, .. }
                                | Operation::DetachPoolPartModel { relative_path, .. } => {
                                    relative_path
                                }
                                _ => continue,
                            };
                            if *path != shard.relative_path {
                                continue;
                            }
                            super::pool_journal_ops::apply_pool_shard_operation(
                                &shard.kind,
                                board_pass.board_mut(),
                                operation,
                            )?;
                        }
                        _ => {}
                    }
                }
            }
            if let Some((id, before)) = map_before
                && (pending_creation
                    || before != value
                    || operations.iter().any(|operation| match operation {
                        Operation::SetPoolLibraryObject {
                            object_id,
                            relative_path,
                            ..
                        }
                        | Operation::CreatePoolLibraryObject {
                            object_id,
                            relative_path,
                            ..
                        }
                        | Operation::DeletePoolLibraryObject {
                            object_id,
                            relative_path,
                            ..
                        } => *object_id == id || *relative_path == shard.relative_path,
                        Operation::BumpObjectRevision { object_id } => *object_id == id,
                        _ => false,
                    }))
            {
                pending_map_changes.insert(id);
            }
            if shard.kind == SourceShardKind::BoardRoot {
                let projection = if let Some(frame) = metadata_basis.as_ref() {
                    super::component_pad_source::projection_with_metadata_basis(
                        model, frame, &value, operations,
                    )?
                } else if let Some(accepted) = part_basis {
                    super::component_pad_source::projection_with_part_basis(
                        model, accepted, &value, operations,
                    )?
                } else if let Some(accepted) = package_basis {
                    super::component_pad_source::projection_with_package_basis(
                        model, accepted, &value, operations,
                    )?
                } else {
                    super::component_pad_source::projection_with_operations(
                        model, &value, operations,
                    )?
                };
                value = super::component_pad_source::project_value(value, &projection)?;
                if let Some((_, _, statuses)) = &mut part_statuses {
                    statuses.insert(shard.shard_id, projection.statuses);
                }
                query_pads.insert(shard.shard_id, projection.query_pads);
            }
            values.insert(shard.shard_id, value);
        }
        let classes = class_index::Index::capture(&values);
        let mut basis = Self {
            values,
            classes,
            query_pads,
            part_statuses,
            accepted_library_basis: part_basis,
            pending_map_changes,
        };
        // Provisional assignments allow the existing PM053 certificate to inspect
        // geometry and map correspondence. Only complete evidence survives capture.
        let mut refused = Vec::new();
        for (shard, value) in &basis.values {
            let Some(groups) = value.get("component_pads").and_then(Value::as_object) else {
                continue;
            };
            for raw in groups.values().filter_map(Value::as_array).flatten() {
                let Some(id) = raw
                    .get("uuid")
                    .and_then(Value::as_str)
                    .and_then(|s| Uuid::parse_str(s).ok())
                else {
                    continue;
                };
                let Some(net) = basis.pad_terminal(model, id).and_then(|p| p.net) else {
                    continue;
                };
                let mut count = 0;
                let mut complete = true;
                for relation in model.electrical_identities.values() {
                    if let super::ElectricalIdentity::NetRelationship {
                        board_net: Some(board_net),
                        intent: super::NetRelationshipIntent::Implemented,
                        evidence,
                        ..
                    } = &relation.identity
                    {
                        for cell in evidence.iter().filter(|e| e.board_pad.object_id == id) {
                            count += 1;
                            complete &= board_net.object_id == net
                                && super::electrical_correspondence::certify(
                                    model,
                                    &basis,
                                    cell,
                                    Some(net),
                                )
                                .is_ok_and(|s| s == super::NetCorrespondenceStatus::Complete);
                        }
                    }
                }
                if count == 0 || !complete {
                    refused.push((*shard, id));
                }
            }
        }
        for (shard, id) in refused {
            if let Some(p) = basis
                .query_pads
                .get_mut(&shard)
                .and_then(|p| p.get_mut(&id))
            {
                p.terminal_mut().net = None;
            }
            if let Some(pad) = basis
                .values
                .get_mut(&shard)
                .and_then(|v| v.get_mut("pads"))
                .and_then(|v| v.get_mut(id.to_string()))
            {
                pad["net"] = Value::Null;
            }
        }
        Ok(basis)
    }
    pub(super) fn query_pads(
        &self,
        shard: Uuid,
    ) -> Option<&BTreeMap<Uuid, crate::board::query_pad::QueryPadGeometry>> {
        self.query_pads.get(&shard)
    }
    pub(super) fn pad_terminal(
        &self,
        model: &DesignModel,
        id: Uuid,
    ) -> Option<crate::board::PlacedPad> {
        let shard = model.objects.get(&id)?.source_shard_id;
        if let Some(pad) = self.query_pads.get(&shard).and_then(|p| p.get(&id)) {
            return Some(pad.terminal().clone());
        }
        serde_json::from_value(self.object(model, id, "pads")?.clone()).ok()
    }
    /// Captured ownership is independent of physical geometry availability.
    /// Component bindings supersede a plain adopted-pad payload; conflicting
    /// source-bound parents remain unavailable rather than becoming standalone.
    pub(super) fn pad_owners(
        &self,
        model: &DesignModel,
        ids: &BTreeSet<Uuid>,
    ) -> BTreeMap<Uuid, Uuid> {
        let shards: BTreeSet<_> = ids
            .iter()
            .filter_map(|id| model.objects.get(id))
            .map(|object| object.source_shard_id)
            .collect();
        let mut result = BTreeMap::new();
        for shard in shards {
            let Some(board) = self.values.get(&shard) else {
                continue;
            };
            let requested = |id: &Uuid| {
                ids.contains(id)
                    && model
                        .objects
                        .get(id)
                        .is_some_and(|object| object.source_shard_id == shard)
            };
            let plain = board.get("pads");
            let entries = plain
                .and_then(Value::as_object)
                .into_iter()
                .flat_map(|values| values.values())
                .chain(
                    plain
                        .and_then(Value::as_array)
                        .into_iter()
                        .flat_map(|values| values.iter()),
                );
            for raw in entries {
                let id = raw
                    .get("uuid")
                    .and_then(Value::as_str)
                    .and_then(|id| Uuid::parse_str(id).ok());
                let parent = raw
                    .get("package")
                    .and_then(Value::as_str)
                    .and_then(|id| Uuid::parse_str(id).ok());
                if let (Some(id), Some(parent)) = (id, parent)
                    && requested(&id)
                {
                    result.entry(id).or_insert(parent);
                }
            }
            let mut bound = BTreeMap::new();
            let mut ambiguous = BTreeSet::new();
            if let Some(packages) = board.get("component_pads").and_then(Value::as_object) {
                for (package, pads) in packages {
                    let Some(parent) = Uuid::parse_str(package).ok() else {
                        continue;
                    };
                    for raw in pads.as_array().into_iter().flatten() {
                        let Some(id) = raw
                            .get("uuid")
                            .and_then(Value::as_str)
                            .and_then(|id| Uuid::parse_str(id).ok())
                        else {
                            continue;
                        };
                        if !requested(&id) || raw.get("source_binding").is_none_or(Value::is_null) {
                            continue;
                        }
                        let Ok(source) = serde_json::from_value::<
                            super::component_pad_source::ComponentPadSource,
                        >(raw.clone()) else {
                            ambiguous.insert(id);
                            continue;
                        };
                        if parent.is_nil()
                            || source
                                .source_binding
                                .as_ref()
                                .is_none_or(|binding| binding.package_id != parent)
                            || source.uuid != id
                        {
                            ambiguous.insert(id);
                            continue;
                        }
                        if bound
                            .insert(id, parent)
                            .is_some_and(|prior| prior != parent)
                        {
                            ambiguous.insert(id);
                        }
                    }
                }
            }
            result.extend(bound);
            if let Some(pads) = self.query_pads.get(&shard) {
                for (id, pad) in pads {
                    if requested(id) {
                        result.insert(*id, pad.terminal().package);
                    }
                }
            }
            for id in ambiguous {
                result.remove(&id);
            }
        }
        result
    }
    pub(super) fn component_pad_status(
        &self,
        model: &DesignModel,
        id: Uuid,
    ) -> Option<super::component_pad_source::ComponentPadMaterializationStatus> {
        let object = model.objects.get(&id)?;
        // Reuse this same qualified projection's result for its exact view;
        // never recompute Part history against the unpublished view journal.
        if let Some((project, revision, statuses)) = &self.part_statuses
            && *project == model.project.project_id
            && *revision == model.model_revision
        {
            return statuses.get(&object.source_shard_id)?.get(&id).copied();
        }
        let board = self.values.get(&object.source_shard_id)?;
        for (package, pads) in board.get("component_pads")?.as_object()? {
            for raw in pads.as_array()? {
                if raw.get("uuid").and_then(Value::as_str) != Some(id.to_string().as_str())
                    || raw.get("source_binding").is_none_or(Value::is_null)
                {
                    continue;
                }
                let pad =
                    serde_json::from_value::<super::component_pad_source::ComponentPadSource>(
                        raw.clone(),
                    )
                    .ok()?;
                let placed: crate::board::PlacedPackage =
                    serde_json::from_value(board["packages"][package].clone()).ok()?;
                return Some(pad.materialization_status(model, placed.part, placed.package));
            }
        }
        None
    }
    pub(super) fn shard_value(&self, id: Uuid) -> Option<&Value> {
        self.values.get(&id)
    }
    pub(super) fn class(&self, model: &DesignModel, id: Uuid, class: &str, domain: &str) -> bool {
        model
            .objects
            .get(&id)
            .filter(|o| o.domain == domain)
            .and_then(|o| self.values.get(&o.source_shard_id).map(|v| (o, v)))
            .is_some_and(|(o, v)| {
                self.classes
                    .contains(o.source_shard_id, class, id)
                    .unwrap_or_else(|| find_class(v, class, id).is_some())
                    || (domain == "pool"
                        && model.objects[&id].kind == class
                        && v.get("uuid").and_then(Value::as_str) == Some(id.to_string().as_str()))
            })
    }
    pub(super) fn object(&self, model: &DesignModel, id: Uuid, class: &str) -> Option<&Value> {
        let object = model.objects.get(&id)?;
        let value = self.values.get(&object.source_shard_id)?;
        find_class(value, class, id).or_else(|| {
            (value.get("uuid").and_then(Value::as_str) == Some(id.to_string().as_str())
                && object.kind == class)
                .then_some(value)
        })
    }
    /// Read the declared map from this captured frame through the shared
    /// authenticated history reader. Held snapshots never reread live files.
    pub(super) fn pinned_map(
        &self,
        model: &DesignModel,
        reference: &super::RevisionedRef,
    ) -> Result<Value, EngineError> {
        if self.pending_map_changes.contains(&reference.object_id) {
            return Err(invalid(
                "declared Map revision is unavailable in a pending source-change frame",
            ));
        }
        let basis = if let Some(accepted) = self.accepted_library_basis {
            if !self
                .part_statuses
                .as_ref()
                .is_some_and(|(project, revision, _)| {
                    *project == model.project.project_id && *revision == model.model_revision
                })
                || model.objects.get(&reference.object_id)
                    != accepted.objects.get(&reference.object_id)
            {
                return Err(invalid(
                    "pinned Map differs from the qualified prospective source basis",
                ));
            }
            accepted
        } else {
            model
        };
        basis.pool_object_revision_with_reader(
            reference.object_id,
            "pin_pad_maps",
            reference.object_revision,
            &|shard| {
                if !model.source_shards.iter().any(|view| view == shard) {
                    return Err(invalid(
                        "pinned Map source shard differs from the captured basis",
                    ));
                }
                self.values
                    .get(&shard.shard_id)
                    .cloned()
                    .ok_or_else(|| invalid("pinned Map source is absent from the captured basis"))
            },
        )
    }
    pub(super) fn occurrence(
        &self,
        model: &DesignModel,
        reference: &ElectricalOccurrence,
    ) -> Result<(), EngineError> {
        let domain = match reference.class.as_str() {
            "wires" | "labels" | "ports" | "junctions" | "pins" | "buses" | "bus_entries"
            | "net_declarations" => "schematic",
            "tracks" | "vias" | "pads" | "zones" => "board",
            _ => return Err(invalid("unsupported occurrence class")),
        };
        if !self.class(model, reference.source_id, &reference.class, domain) {
            return Err(invalid("occurrence source class mismatch"));
        }
        if domain == "board" && !reference.instance_path.is_empty() {
            return Err(invalid("board occurrence has schematic path"));
        }
        let mut previous_sheet = None;
        let mut seen = std::collections::BTreeSet::new();
        for instance in &reference.instance_path {
            if !seen.insert(*instance) {
                return Err(invalid("cyclic instance path"));
            }
            let value = model
                .objects
                .get(instance)
                .filter(|o| o.domain == "schematic")
                .and_then(|o| self.values.get(&o.source_shard_id))
                .and_then(|v| find_class(v, "instances", *instance))
                .ok_or_else(|| invalid("missing SheetInstance"))?;
            let instance: crate::schematic::SheetInstance = serde_json::from_value(value.clone())?;
            if let Some(parent) = previous_sheet {
                if instance.parent_sheet != Some(parent) {
                    return Err(invalid("unordered instance ancestry"));
                }
            } else if instance
                .parent_sheet
                .is_some_and(|parent| !model.objects.contains_key(&parent))
            {
                return Err(invalid("missing root parent sheet"));
            }
            let definition = model
                .objects
                .get(&instance.definition)
                .and_then(|o| self.values.get(&o.source_shard_id))
                .ok_or_else(|| invalid("missing SheetDefinition"))?;
            let definition: crate::schematic::SheetDefinition =
                serde_json::from_value(definition.clone())?;
            if definition.uuid != instance.definition {
                return Err(invalid("definition identity mismatch"));
            }
            previous_sheet = Some(definition.root_sheet);
        }
        if let Some(sheet) = previous_sheet {
            let source = &model.objects[&reference.source_id];
            if self
                .values
                .get(&source.source_shard_id)
                .and_then(|v| v.get("uuid"))
                .and_then(Value::as_str)
                != Some(sheet.to_string().as_str())
            {
                return Err(invalid(
                    "occurrence does not belong to final instance sheet",
                ));
            }
        }
        Ok(())
    }
}

pub(super) fn find_class<'a>(value: &'a Value, class: &str, id: Uuid) -> Option<&'a Value> {
    fn find_id(value: &Value, id: Uuid) -> Option<&Value> {
        if value.get("uuid").and_then(Value::as_str) == Some(id.to_string().as_str()) {
            return Some(value);
        }
        match value {
            Value::Array(a) => a.iter().find_map(|v| find_id(v, id)),
            Value::Object(o) => o.values().find_map(|v| find_id(v, id)),
            _ => None,
        }
    }
    match value {
        Value::Object(o) => o
            .get(class)
            .and_then(|v| find_id(v, id))
            .or_else(|| o.values().find_map(|v| find_class(v, class, id))),
        Value::Array(a) => a.iter().find_map(|v| find_class(v, class, id)),
        _ => None,
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("invalid electrical source basis: {reason}"))
}
