//! Canonical nongraphical source births and exact compensating preimages.
use super::schematic_sheet_maps::{
    insert_sheet_map_value, remove_sheet_map_value, sheet_map_value, sheet_uuid,
};
use super::{EngineError, Operation};
use serde_json::{Value, json};

pub(super) fn apply(
    sheet: &mut Value,
    operation: &Operation,
    inverse: Option<&mut Vec<Operation>>,
) -> Result<Option<bool>, EngineError> {
    match operation {
        Operation::CreateSchematicNetDeclaration {
            sheet_id,
            declaration_id,
            declaration,
        } => {
            if sheet_uuid(sheet) != Some(*sheet_id) {
                return Ok(Some(false));
            }
            let typed: crate::schematic::NetDeclaration =
                serde_json::from_value(declaration.clone())?;
            if typed.uuid != *declaration_id
                || typed.uuid.is_nil()
                || typed.name.is_empty()
                || typed.name.trim() != typed.name
                || typed.name.chars().any(char::is_control)
            {
                return Err(invalid("invalid declaration UUID or scalar name"));
            }
            let absent = sheet.get("net_declarations").is_none();
            if !absent && sheet["net_declarations"].as_object().is_none() {
                return Err(invalid("declarations must be a source map"));
            }
            if sheet["net_declarations"]
                .get(declaration_id.to_string())
                .is_some()
            {
                return Err(invalid("declaration source already exists"));
            }
            if let Some(inverse) = inverse {
                inverse.push(Operation::DeleteSchematicNetDeclaration {
                    sheet_id: *sheet_id,
                    declaration_id: *declaration_id,
                    declaration: declaration.clone(),
                    restore_collection_absence: absent,
                });
            }
            if absent {
                sheet["net_declarations"] = json!({});
            }
            insert_sheet_map_value(
                sheet,
                "net_declarations",
                *declaration_id,
                declaration.clone(),
            )?;
            Ok(Some(true))
        }
        Operation::DeleteSchematicNetDeclaration {
            sheet_id,
            declaration_id,
            declaration,
            restore_collection_absence,
        } => {
            if sheet_uuid(sheet) != Some(*sheet_id) {
                return Ok(Some(false));
            }
            let previous = sheet_map_value(sheet, "net_declarations", *declaration_id)?.clone();
            if previous != *declaration {
                return Err(invalid("declaration compensation preimage changed"));
            }
            if *restore_collection_absence
                && sheet["net_declarations"]
                    .as_object()
                    .is_none_or(|map| map.len() != 1)
            {
                return Err(invalid(
                    "cannot restore absent collection with surviving declarations",
                ));
            }
            if let Some(inverse) = inverse {
                inverse.push(Operation::CreateSchematicNetDeclaration {
                    sheet_id: *sheet_id,
                    declaration_id: *declaration_id,
                    declaration: previous,
                });
            }
            remove_sheet_map_value(sheet, "net_declarations", *declaration_id)?;
            if *restore_collection_absence {
                sheet.as_object_mut().unwrap().remove("net_declarations");
            }
            Ok(Some(true))
        }
        _ => Ok(None),
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("scalar declaration source refused: {reason}"))
}

/// Prepare the complete final source once, then expose its recorded membership
/// to the Bus builder. Commit revalidates the same explicit identity operations.
pub(crate) fn plan(
    model: &super::DesignModel,
    batch: super::OperationBatch,
) -> Result<
    (
        super::OperationBatch,
        std::collections::BTreeMap<super::ElectricalOccurrence, uuid::Uuid>,
    ),
    EngineError,
> {
    let batch = super::electrical_transaction::bootstrap_empty_native_topology(model, batch)?;
    let batch =
        super::electrical_transaction::compose(model, batch, super::TransactionKind::Normal)?;
    let mut candidate = model.clone();
    for op in &batch.operations {
        match op {
            Operation::CreateElectricalIdentity { record }
            | Operation::SetElectricalIdentity { record, .. } => {
                candidate
                    .electrical_identities
                    .insert(record.id, record.clone());
            }
            _ => {}
        }
    }
    let groups = super::electrical_topology_source::partitions(model, &batch.operations)?;
    let membership = super::electrical_transaction::previous(&candidate, &groups)?
        .into_iter()
        .flat_map(|group| {
            group
                .members
                .into_iter()
                .map(move |member| (member, group.net_id))
        })
        .collect();
    Ok((batch, membership))
}

pub(super) fn validate(
    model: &super::DesignModel,
    operations: &[Operation],
) -> Result<(), EngineError> {
    if !model
        .source_shards
        .iter()
        .any(|s| s.kind == super::SourceShardKind::SchematicRoot)
    {
        return Ok(());
    }
    let source = super::electrical_topology_source::schematic(model, operations)?;
    // The partition owner checks source keys/names/context and explicit contacts.
    if source.sheets.values().all(|s| {
        s.net_declarations.is_empty()
            && s.bus_entries
                .values()
                .all(|e| e.member_declaration.is_none())
    }) {
        return Ok(());
    }
    let groups = crate::connectivity::partitions(&source)?;
    let membership = super::electrical_transaction::previous(model, &groups)?;
    for record in model.electrical_identities.values() {
        let super::ElectricalIdentity::Bus {
            scalar_declarations,
            representations,
            retired: false,
            ..
        } = &record.identity
        else {
            continue;
        };
        for declaration in scalar_declarations {
            let name = source
                .sheets
                .values()
                .find_map(|s| s.net_declarations.get(&declaration.source_id))
                .ok_or_else(|| invalid("Bus declaration source is missing"))?
                .name
                .as_str();
            for drawing in representations.iter().filter(|r| r.class == "buses") {
                let bus = source
                    .sheets
                    .values()
                    .find_map(|s| s.buses.get(&drawing.source_id))
                    .ok_or_else(|| invalid("Bus drawing source is missing"))?;
                if !bus.members.iter().any(|member| member == name) {
                    return Err(invalid(
                        "Bus member edit would lose a declaration binding; use the qualified Bus member edit",
                    ));
                }
            }
        }
    }
    for sheet in source.sheets.values() {
        for bus in sheet.buses.values().filter(|b| {
            b.members
                .iter()
                .any(|name| sheet.net_declarations.values().any(|d| &d.name == name))
        }) {
            let mut members = std::collections::BTreeSet::new();
            for name in &bus.members {
                let candidates: Vec<_> = sheet
                    .net_declarations
                    .values()
                    .filter(|d| &d.name == name)
                    .map(|d| ("net_declarations", d.uuid))
                    .chain(
                        sheet
                            .labels
                            .values()
                            .filter(|l| &l.name == name)
                            .map(|l| ("labels", l.uuid)),
                    )
                    .chain(
                        sheet
                            .ports
                            .values()
                            .filter(|p| &p.name == name)
                            .map(|p| ("ports", p.uuid)),
                    )
                    .collect();
                let nets: std::collections::BTreeSet<_> = candidates
                    .into_iter()
                    .filter_map(|(class, source_id)| {
                        let occurrence = super::ElectricalOccurrence {
                            class: class.into(),
                            source_id,
                            instance_path: vec![],
                        };
                        membership
                            .iter()
                            .find(|g| g.members.contains(&occurrence))
                            .map(|g| g.net_id)
                    })
                    .collect();
                if nets.len() != 1 || !members.insert(*nets.first().unwrap()) {
                    return Err(invalid(
                        "Bus members must resolve to distinct canonical scalars",
                    ));
                }
            }
        }
        for id in sheet.net_declarations.keys() {
            let occurrence = super::ElectricalOccurrence {
                class: "net_declarations".into(),
                source_id: *id,
                instance_path: vec![],
            };
            if !membership.iter().any(|g| g.members.contains(&occurrence)) {
                return Err(invalid("declaration has no recorded scalar identity"));
            }
        }
        for entry in sheet.bus_entries.values() {
            let Some(member) = &entry.member_declaration else {
                continue;
            };
            let net = membership
                .iter()
                .find(|g| g.members.contains(member))
                .ok_or_else(|| invalid("unowned entry member"))?
                .net_id;
            let origin = super::ElectricalOccurrence {
                class: "buses".into(),
                source_id: entry.bus,
                instance_path: vec![],
            };
            let owners: Vec<_> = model
                .electrical_identities
                .values()
                .filter_map(|record| match &record.identity {
                    super::ElectricalIdentity::Bus {
                        scalar_nets,
                        scalar_declarations,
                        representations,
                        retired: false,
                        ..
                    } if representations.contains(&origin) => {
                        Some((scalar_nets, scalar_declarations))
                    }
                    _ => None,
                })
                .collect();
            if owners.len() != 1 || !owners[0].0.contains(&net) || !owners[0].1.contains(member) {
                return Err(invalid(
                    "entry declaration is outside the exact semantic Bus",
                ));
            }
        }
    }
    Ok(())
}
