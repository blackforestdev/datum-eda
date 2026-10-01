//! Canonical materialized source basis for identity validation. Object kind strings
//! are resolver index hints, not source class or hierarchy authority.
use super::{DesignModel, ElectricalOccurrence, EngineError, Operation, SourceShardKind};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

pub(super) struct ElectricalBasis {
    values: BTreeMap<Uuid, Value>,
}
impl ElectricalBasis {
    pub(super) fn new(model: &DesignModel, operations: &[Operation]) -> Result<Self, EngineError> {
        let mut values = BTreeMap::new();
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
            let mut value = match pending_create {
                Some(value) => value,
                None => super::journal::materialized_shard_value(model, shard)?,
            };
            for operation in operations {
                match shard.kind {
                    SourceShardKind::SchematicRoot => {
                        super::schematic_root_journal_ops::apply_schematic_root_operation(
                            &mut value, operation,
                        )?;
                    }
                    SourceShardKind::SchematicSheet => {
                        super::schematic_sheet_journal_ops::apply_schematic_sheet_operation(
                            &mut value, operation,
                        )?;
                    }
                    SourceShardKind::BoardRoot => {
                        super::board_journal_ops::apply_board_operation(&mut value, operation)?;
                    }
                    SourceShardKind::Pool => {
                        super::pool_journal_ops::apply_pool_shard_operation(
                            &shard.kind,
                            &mut value,
                            operation,
                        )?;
                    }
                    _ => {}
                }
            }
            values.insert(shard.shard_id, value);
        }
        Ok(Self { values })
    }
    pub(super) fn class(&self, model: &DesignModel, id: Uuid, class: &str, domain: &str) -> bool {
        model
            .objects
            .get(&id)
            .filter(|o| o.domain == domain)
            .and_then(|o| self.values.get(&o.source_shard_id))
            .is_some_and(|v| {
                find_class(v, class, id).is_some()
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
    pub(super) fn occurrence(
        &self,
        model: &DesignModel,
        reference: &ElectricalOccurrence,
    ) -> Result<(), EngineError> {
        let domain = match reference.class.as_str() {
            "wires" | "labels" | "ports" | "junctions" | "pins" | "buses" | "bus_entries" => {
                "schematic"
            }
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
