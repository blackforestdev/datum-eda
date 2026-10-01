//! Materialize immutable pre/final schematic input through canonical source owners.
use super::{DesignModel, EngineError, Operation, SourceShardKind};
use crate::schematic::{Schematic, Sheet, SheetDefinition, SheetInstance};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

pub(super) fn schematic(
    model: &DesignModel,
    operations: &[Operation],
) -> Result<Schematic, EngineError> {
    let mut root = None;
    let mut sheets = BTreeMap::new();
    let mut definitions = BTreeMap::new();
    for shard in &model.source_shards {
        if !matches!(
            shard.kind,
            SourceShardKind::SchematicRoot
                | SourceShardKind::SchematicSheet
                | SourceShardKind::SchematicDefinition
        ) {
            continue;
        }
        let mut value = super::journal::materialized_shard_value(model, shard)?;
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
                _ => {}
            }
        }
        match shard.kind {
            SourceShardKind::SchematicRoot => {
                if root.replace(value).is_some() {
                    return Err(invalid("multiple roots"));
                }
            }
            SourceShardKind::SchematicSheet => {
                let sheet: Sheet = serde_json::from_value(value)?;
                sheets.insert(sheet.uuid, sheet);
            }
            SourceShardKind::SchematicDefinition => {
                let definition: SheetDefinition = serde_json::from_value(value)?;
                definitions.insert(definition.uuid, definition);
            }
            _ => {}
        }
    }
    for operation in operations {
        match operation {
            Operation::CreateSchematicSheet { sheet, .. } => {
                let mut value = sheet.clone();
                for operation in operations {
                    super::schematic_sheet_journal_ops::apply_schematic_sheet_operation(
                        &mut value, operation,
                    )?;
                }
                let sheet: Sheet = serde_json::from_value(value)?;
                sheets.insert(sheet.uuid, sheet);
            }
            Operation::CreateSchematicDefinition { definition, .. } => {
                let definition: SheetDefinition = serde_json::from_value(definition.clone())?;
                definitions.insert(definition.uuid, definition);
            }
            _ => {}
        }
    }
    let root = root.ok_or_else(|| invalid("missing root"))?;
    let selected = |key: &str| -> Result<Vec<Uuid>, EngineError> {
        root.get(key)
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| invalid("missing source index"))?
            .keys()
            .map(|s| Uuid::parse_str(s).map_err(|_| invalid("invalid source index UUID")))
            .collect()
    };
    let sheets: HashMap<_, _> = selected("sheets")?
        .into_iter()
        .map(|id| {
            sheets
                .remove(&id)
                .map(|v| (id, v))
                .ok_or_else(|| invalid("missing indexed sheet"))
        })
        .collect::<Result<_, _>>()?;
    let sheet_definitions: HashMap<_, _> = selected("definitions")?
        .into_iter()
        .map(|id| {
            definitions
                .remove(&id)
                .map(|v| (id, v))
                .ok_or_else(|| invalid("missing indexed definition"))
        })
        .collect::<Result<_, _>>()?;
    let mut sheet_instances = HashMap::new();
    let instances: Vec<SheetInstance> = serde_json::from_value(
        root.get("instances")
            .cloned()
            .ok_or_else(|| invalid("missing instance index"))?,
    )?;
    for instance in instances {
        if sheet_instances.insert(instance.uuid, instance).is_some() {
            return Err(invalid("duplicate SheetInstance"));
        }
    }
    Ok(Schematic {
        uuid: serde_json::from_value(root["uuid"].clone())?,
        sheets,
        sheet_definitions,
        sheet_instances,
        variants: HashMap::new(),
        waivers: vec![],
    })
}

pub(crate) fn partitions(
    model: &DesignModel,
    operations: &[Operation],
) -> Result<Vec<std::collections::BTreeSet<super::ElectricalOccurrence>>, EngineError> {
    crate::connectivity::partitions(&schematic(model, operations)?)
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("invalid topology source: {reason}"))
}
