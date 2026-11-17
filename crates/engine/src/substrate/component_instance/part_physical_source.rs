//! Physical source intent at a canonical Part adoption, read from accepted history.
use super::super::{DesignModel, Operation, TransactionKind};
use super::declaration_history::{authenticate_assignment, effective_history};
use super::*;
use serde_json::Value;
use std::collections::BTreeMap;

fn invalid(message: &str) -> EngineError {
    EngineError::Validation(format!("Component physical choice: {message}"))
}
fn uuid(value: &Value) -> Result<Uuid, EngineError> {
    value
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| invalid("declared library identity is unavailable"))
}
fn part_signature(raw: &Value) -> Result<Option<(Uuid, ObjectRevision)>, EngineError> {
    if raw["part_ref"].is_null() {
        return Ok(None);
    }
    let part = uuid(&raw["part_ref"]["object_id"])?;
    let revision: ObjectRevision =
        serde_json::from_value(raw["part_ref"]["object_revision"].clone())?;
    let bindings = raw["library_bindings"]
        .as_object()
        .ok_or_else(|| invalid("Part bindings are unavailable"))?;
    let parts: Vec<LibraryBinding> = bindings
        .values()
        .map(|raw| serde_json::from_value(raw.clone()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|b: &LibraryBinding| b.binding_role == LibraryBindingRole::Part)
        .collect();
    if parts.len() != 1
        || parts[0].target_object_id != part
        || parts[0].pinned_object_revision != revision
    {
        return Err(invalid("Part declaration is missing or ambiguous"));
    }
    Ok(Some((part, revision)))
}

impl DesignModel {
    fn component_part_adoption(&self, component: Uuid, part: Uuid) -> Result<usize, EngineError> {
        self.component_part_adoption_for_scope(component, part, false)
    }

    fn component_part_adoption_for_scope(
        &self,
        component: Uuid,
        part: Uuid,
        bound_metadata: bool,
    ) -> Result<usize, EngineError> {
        if self.journal_cursor.applied_transaction_count != self.journal.len() {
            return Err(invalid("journal tip differs from the current source"));
        }
        let ci = self
            .component_instances
            .get(&component)
            .filter(|ci| {
                ci.authority == ComponentInstanceAuthority::Authored
                    && if bound_metadata {
                        ci.placed_package_refs.len() == 1 && ci.placed_symbol_refs.len() == 1
                    } else {
                        ci.placed_package_refs.is_empty()
                    }
            })
            .ok_or_else(|| {
                invalid("requires an authored component awaiting initial PCB placement")
            })?;
        if ci.part_ref != Some(part) || ci.placed_symbol_refs.is_empty() {
            return Err(invalid(
                "current component Part or Symbol membership differs",
            ));
        }
        let owner = self
            .objects
            .get(&component)
            .ok_or_else(|| invalid("component source is unavailable"))?;
        let shard = self
            .source_shards
            .iter()
            .find(|s| s.shard_id == owner.source_shard_id)
            .ok_or_else(|| invalid("component source owner is unavailable"))?;
        let raw = self.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        let mut current = Some(raw["component_instance"].clone());
        let expected = part_signature(current.as_ref().unwrap())?
            .filter(|(id, _)| *id == part)
            .ok_or_else(|| invalid("component has no matching Part declaration"))?;
        let history = effective_history(self)?;
        let births = self.journal.iter().filter(|tx| tx.transaction_kind == TransactionKind::Normal)
            .flat_map(|tx| &tx.operations).filter(|op| matches!(op,
                Operation::CreateComponentInstance { component_instance_id, .. } if *component_instance_id == component)).count();
        if births != 1 {
            return Err(invalid("component birth epoch is missing or ambiguous"));
        }
        if let Some(adoption) =
            crate::substrate::library_asset_journal_ops::history::component_adoption(
                self,
                component,
                part,
                current.as_ref().unwrap(),
            )?
        {
            return Ok(adoption);
        }
        for tx in self.journal.iter().rev() {
            let changes: Vec<_> = tx
                .operations
                .iter()
                .filter(|op| {
                    matches!(op,
                Operation::CreateComponentInstance { component_instance_id, .. }
                | Operation::SetComponentInstance { component_instance_id, .. }
                | Operation::DeleteComponentInstance { component_instance_id, .. }
                if *component_instance_id == component)
                })
                .collect();
            if changes.len() > 1 {
                return Err(invalid("ambiguous component source transaction"));
            }
            let Some(op) = changes.first() else {
                continue;
            };
            match op {
                Operation::CreateComponentInstance {
                    component_instance: next,
                    ..
                }
                | Operation::SetComponentInstance {
                    component_instance: next,
                    ..
                } => {
                    if current.as_ref() != Some(next) {
                        return Err(invalid(
                            "component source postimage differs from its history",
                        ));
                    }
                    let origin = *history
                        .origins
                        .get(&tx.transaction_id)
                        .ok_or_else(|| invalid("component compensation origin is unavailable"))?;
                    let position = *history
                        .positions
                        .get(&origin)
                        .ok_or_else(|| invalid("component adoption transaction is unavailable"))?;
                    let source = &self.journal[position];
                    if history.effective.contains(&origin) {
                        for adoption in &source.operations {
                            let (next, before) = match adoption {
                                Operation::CreateComponentInstance {
                                    component_instance_id,
                                    component_instance,
                                } if *component_instance_id == component => {
                                    (component_instance, None)
                                }
                                Operation::SetComponentInstance {
                                    component_instance_id,
                                    component_instance,
                                    previous_component_instance,
                                } if *component_instance_id == component => {
                                    (component_instance, Some(previous_component_instance))
                                }
                                _ => continue,
                            };
                            if part_signature(next)? == Some(expected)
                                && before.map(part_signature).transpose()?.flatten()
                                    != Some(expected)
                            {
                                let member = next["placed_symbol_refs"]
                                    .as_array()
                                    .and_then(|members| members.first())
                                    .ok_or_else(|| {
                                        invalid("adopted Symbol membership is unavailable")
                                    })?;
                                let placed = uuid(&member["object_id"])?;
                                let birth = before.is_none() && source.operations.iter().filter(|op| matches!(op,
                                    Operation::CreateSchematicSymbol { symbol_id, .. } if *symbol_id == placed)).count() == 1;
                                if !birth {
                                    authenticate_assignment(self, source, placed)?;
                                }
                                return Ok(position);
                            }
                        }
                    }
                    current = match op {
                        Operation::SetComponentInstance {
                            previous_component_instance,
                            ..
                        } => Some(previous_component_instance.clone()),
                        _ => None,
                    };
                }
                Operation::DeleteComponentInstance {
                    component_instance: deleted,
                    ..
                } => {
                    if current.is_some() {
                        return Err(invalid("component deletion postimage differs"));
                    }
                    if tx.inverse_operations.iter().filter(|op| matches!(op,
                        Operation::CreateComponentInstance { component_instance_id, component_instance }
                        if *component_instance_id == component && component_instance == deleted)).count() != 1
                    { return Err(invalid("component restoration preimage is unavailable")); }
                    current = Some(deleted.clone());
                }
                _ => unreachable!(),
            }
        }
        Err(invalid("authenticated Part adoption is unavailable"))
    }

    pub(crate) fn declared_component_part_sources(
        &self,
        component: Uuid,
        part: Uuid,
    ) -> Result<BTreeMap<Uuid, ObjectRevision>, EngineError> {
        let adoption = self.component_part_adoption(component, part)?;
        self.component_part_source_graph(part, adoption, false)
    }

    /// Metadata propagation retains the original complete logical adoption even
    /// after physical placement. It neither rebinds nor chooses a newer library
    /// epoch. Explicit frozen physical/Map checks remain the caller's duty.
    pub(crate) fn declared_bound_component_part_sources(
        &self,
        component: Uuid,
        part: Uuid,
    ) -> Result<BTreeMap<Uuid, ObjectRevision>, EngineError> {
        let adoption = self.component_part_adoption_for_scope(component, part, true)?;
        self.component_part_source_graph(part, adoption, true)
    }

    fn component_part_source_graph(
        &self,
        part: Uuid,
        adoption: usize,
        legacy_optional_defaults: bool,
    ) -> Result<BTreeMap<Uuid, ObjectRevision>, EngineError> {
        let mut pending = vec![(part, "parts")];
        let mut sources = BTreeMap::new();
        let mut kinds = BTreeMap::new();
        while let Some((id, kind)) = pending.pop() {
            if let Some(previous) = kinds.insert(id, kind) {
                if previous != kind {
                    return Err(invalid("component source identity has conflicting kinds"));
                }
                continue;
            }
            let (raw, revision) = self.pool_source_at_transaction(id, kind, adoption)?;
            sources.insert(id, revision);
            let mut add = |value: &Value, kind| -> Result<(), EngineError> {
                pending.push((uuid(value)?, kind));
                Ok(())
            };
            match kind {
                "parts" => {
                    add(&raw["entity"], "entities")?;
                    add(&raw["package"], "packages")?;
                    if !legacy_optional_defaults || !raw["default_footprint"].is_null() {
                        add(&raw["default_footprint"], "footprints")?;
                    }
                    if !legacy_optional_defaults || !raw["default_pin_pad_map"].is_null() {
                        add(&raw["default_pin_pad_map"], "pin_pad_maps")?;
                    }
                }
                "entities" => {
                    for gate in raw["gates"]
                        .as_object()
                        .ok_or_else(|| invalid("component Gates are unavailable"))?
                        .values()
                    {
                        add(&gate["unit"], "units")?;
                        add(&gate["symbol"], "symbols")?;
                    }
                }
                "symbols" => add(&raw["unit"], "units")?,
                "pin_pad_maps" => {
                    add(&raw["part"], "parts")?;
                    add(&raw["footprint"], "footprints")?;
                }
                "footprints" => {
                    add(&raw["package"], "packages")?;
                    for pad in raw["pads"]
                        .as_object()
                        .ok_or_else(|| invalid("component Pads are unavailable"))?
                        .values()
                    {
                        add(&pad["padstack"], "padstacks")?;
                    }
                }
                _ => {}
            }
        }
        Ok(sources)
    }
}
