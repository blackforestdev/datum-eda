//! One canonical initial-create batch, including its current CI and Net evidence.
use super::*;
use crate::api::native_write::{
    BatchComposer, board_components::pad_materialization, component_instances,
};
use crate::board::{Net, PlacedPackage};
use crate::substrate::{
    ComponentInstance, DesignModel, ElectricalIdentity, ElectricalIdentityRecord,
    ElectricalSelectionSnapshot, NetCorrespondence, NetRelationshipIntent, ObjectRevision,
    Operation, OperationBatch, RevisionedRef, SourceShardKind, SourceShardTaxon,
    component_pad_source as source,
};
use std::collections::BTreeMap;

fn birth(model: &DesignModel, seed: Uuid, domain: &str, id: Uuid) -> Result<Uuid, EngineError> {
    let id = Uuid::new_v5(
        &seed,
        format!("{}:{domain}:{id}", model.project.project_id).as_bytes(),
    );
    if model.objects.contains_key(&id)
        || model
            .journal
            .iter()
            .any(|t| t.diff.created.contains(&id) || t.diff.deleted.contains(&id))
    {
        return Err(invalid(
            "Retained transfer identity is already allocated or retired; restart the review",
        ));
    }
    Ok(id)
}

pub(super) fn prepare(
    model: &DesignModel,
    provenance: WriteProvenance,
    request: &TransferRequest,
    plan: &TransferPlan,
) -> Result<PreparedWrite, EngineError> {
    let mut operations = Vec::new();
    let mut nets: BTreeMap<Uuid, Vec<NetCorrespondence>> = BTreeMap::new();
    for (index, row) in plan
        .components
        .iter()
        .filter(|r| r.status == TransferStatus::Ready)
        .enumerate()
    {
        let ci = row
            .component_instance
            .ok_or_else(|| invalid("Missing component identity"))?;
        let part = row.part.ok_or_else(|| invalid("Missing Part"))?;
        let map = row
            .pin_pad_map
            .ok_or_else(|| invalid("Missing explicit Map"))?;
        let footprint = row
            .footprint
            .ok_or_else(|| invalid("Missing explicit footprint"))?;
        let package = row.package.ok_or_else(|| invalid("Missing Package"))?;
        let placement_id = birth(model, request.edit_seed, "package", ci)?;
        let x = i64::try_from(
            i128::from(request.origin.x) + i128::from(request.pitch_nm) * index as i128,
        )
        .map_err(|_| invalid("Initial footprint positions exceed coordinate range"))?;
        let placement = PlacedPackage {
            uuid: placement_id,
            part,
            package,
            reference: row.reference.clone(),
            value: row
                .value
                .clone()
                .ok_or_else(|| invalid("Missing agreed component instance value"))?,
            position: Point::new(x, request.origin.y),
            rotation: 0,
            layer: request.layer,
            locked: false,
        };
        let binding = component_instances::initial_package_binding(model, ci, &placement, map)?;
        let ci_ref = RevisionedRef {
            object_id: ci,
            object_revision: ObjectRevision(
                model.component_instances[&ci]
                    .object_revision
                    .0
                    .checked_add(1)
                    .ok_or_else(|| invalid("Component revision overflow"))?,
            ),
        };
        let allocations: BTreeMap<_, _> = row
            .connections
            .iter()
            .map(|p| {
                birth(
                    model,
                    request.edit_seed,
                    &format!("pad:{ci}"),
                    p.library_pad,
                )
                .map(|id| (p.library_pad, id))
            })
            .collect::<Result<_, _>>()?;
        let materialized = pad_materialization::initial_materialization(
            model,
            placement.clone(),
            footprint,
            map,
            ci_ref.clone(),
            &allocations,
        )?;
        operations.push(Operation::CreateBoardPackage {
            package_id: placement_id,
            package: serde_json::to_value(&placement)?,
            materialized,
        });
        operations.push(binding);
        let map_pin = source::capture_root(model, map, SourceShardTaxon::PoolPinPadMap)?;
        let footprint_pin =
            source::capture_root(model, footprint, SourceShardTaxon::PoolFootprint)?;
        for connection in &row.connections {
            let library_pad = source::capture_pad(model, &footprint_pin, connection.library_pad)?;
            let library_pin = model
                .objects
                .get(&connection.library_pin)
                .filter(|o| o.domain == "pool")
                .ok_or_else(|| invalid("Mapped library Pin is not registered"))?;
            nets.entry(connection.logical_net)
                .or_default()
                .push(NetCorrespondence {
                    component_instance: ci_ref.clone(),
                    pin_pad_map: RevisionedRef {
                        object_id: map,
                        object_revision: map_pin.object_revision,
                    },
                    schematic_terminal: connection.schematic_terminal.clone(),
                    board_pad: RevisionedRef {
                        object_id: allocations[&connection.library_pad],
                        object_revision: ObjectRevision(0),
                    },
                    library_pad: Some(RevisionedRef {
                        object_id: library_pad.object_id,
                        object_revision: library_pad.object_revision,
                    }),
                    library_pin: Some(RevisionedRef {
                        object_id: connection.library_pin,
                        object_revision: library_pin.object_revision,
                    }),
                });
        }
    }
    for (logical_net, mut evidence) in nets {
        evidence.sort_by(|a, b| a.board_pad.object_id.cmp(&b.board_pad.object_id));
        let net_id = birth(model, request.edit_seed, "board-net", logical_net)?;
        let net = Net::new(
            net_id,
            net_name(model, request.sheet, logical_net)?,
            Uuid::nil(),
        );
        operations.push(Operation::CreateBoardNet {
            net_id,
            net: serde_json::to_value(net)?,
        });
        operations.push(Operation::CreateElectricalIdentity {
            record: ElectricalIdentityRecord {
                id: birth(model, request.edit_seed, "net-relationship", logical_net)?,
                object_revision: ObjectRevision(0),
                identity: ElectricalIdentity::NetRelationship {
                    logical_net,
                    board_net: Some(RevisionedRef {
                        object_id: net_id,
                        object_revision: ObjectRevision(0),
                    }),
                    intent: NetRelationshipIntent::Implemented,
                    evidence,
                },
            },
        });
    }
    let mut prepared = BatchComposer::compose(model, provenance)
        .push_ops(operations)
        .finish()?;
    prepared.batch.batch_id = request.edit_seed;
    Ok(prepared)
}

fn net_name(model: &DesignModel, sheet: Uuid, net: Uuid) -> Result<String, EngineError> {
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Net name basis unavailable: {e:?}")))?;
    let source = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("No schematic name basis"))?;
    let mut names: Vec<_> = source.sheets[&sheet]
        .labels
        .values()
        .filter(|label| {
            snapshot.schematic_member_net(&ElectricalOccurrence {
                class: "labels".into(),
                source_id: label.uuid,
                instance_path: vec![],
            }) == Ok(net)
        })
        .map(|label| label.name.clone())
        .collect();
    names.sort();
    Ok(names
        .into_iter()
        .next()
        .unwrap_or_else(|| format!("Net {net}")))
}

/// Validate the initial producer against the actual source and complete typed
/// create graph. Re-derivation avoids blessing arbitrary CI/Pad/Net payloads or
/// broadening the existing same-binding refresh exception.
pub(crate) fn initial_bindings(
    model: &DesignModel,
    batch: &OperationBatch,
) -> Result<BTreeMap<Uuid, ComponentInstance>, EngineError> {
    let mut placements = BTreeMap::new();
    for op in &batch.operations {
        if let Operation::CreateBoardPackage {
            package,
            materialized,
            ..
        } = op
        {
            let placement: PlacedPackage = serde_json::from_value(package.clone())?;
            let pads = materialized["component_pads"]
                .as_array()
                .ok_or_else(|| invalid("Initial footprint has no Pad sources"))?;
            let mut owners = BTreeMap::new();
            for pad in pads {
                let ci: RevisionedRef =
                    serde_json::from_value(pad["source_binding"]["component_instance"].clone())?;
                let map: Uuid = serde_json::from_value(
                    pad["source_binding"]["pin_pad_map"]["object_id"].clone(),
                )?;
                owners.insert((ci.object_id, map), ());
            }
            if owners.len() != 1 {
                return Err(invalid(
                    "Initial footprint must have one explicit CI/Map owner",
                ));
            }
            let (key, _) = owners.into_iter().next().unwrap();
            if placements.insert(key.0, (placement, key.1)).is_some() {
                return Err(invalid(
                    "Initial transfer creates more than one footprint for a component",
                ));
            }
        }
    }
    if placements.is_empty() {
        return Err(invalid("Initial transfer has no created footprints"));
    }
    let mut sheet = None;
    for ci in placements.keys() {
        let owner = model
            .component_instances
            .get(ci)
            .ok_or_else(|| invalid("Unknown initial component"))?;
        let symbol = owner
            .placed_symbol_refs
            .first()
            .ok_or_else(|| invalid("Component has no source Symbol"))?;
        let object = model
            .objects
            .get(symbol)
            .ok_or_else(|| invalid("Component Symbol is missing"))?;
        let shard = model
            .source_shards
            .iter()
            .find(|s| {
                s.shard_id == object.source_shard_id && s.kind == SourceShardKind::SchematicSheet
            })
            .ok_or_else(|| invalid("Component has no native sheet source"))?;
        let id: Uuid = serde_json::from_value(
            model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?["uuid"]
                .clone(),
        )?;
        if sheet.replace(id).is_some_and(|old| old != id) {
            return Err(invalid("Initial transfer mixes source sheets"));
        }
    }
    let mut request = TransferRequest {
        sheet: sheet.unwrap(),
        choices: placements
            .iter()
            .map(|(ci, (_, map))| TransferChoice {
                component_instance: *ci,
                pin_pad_map: *map,
            })
            .collect(),
        edit_seed: batch.batch_id,
        origin: Point::zero(),
        pitch_nm: 0,
        layer: 0,
    };
    let plan = plan_transfer(model, &request)?;
    let ready: Vec<_> = plan
        .components
        .iter()
        .filter(|r| r.status == TransferStatus::Ready)
        .collect();
    if ready.len() != placements.len() {
        return Err(invalid(
            "Initial batch does not match the complete eligible source scope",
        ));
    }
    let first = &placements[&ready[0].component_instance.unwrap()].0;
    request.origin = first.position;
    request.layer = first.layer;
    if ready.len() > 1 {
        let second = &placements[&ready[1].component_instance.unwrap()].0;
        request.pitch_nm =
            i64::try_from(i128::from(second.position.x) - i128::from(first.position.x))
                .map_err(|_| invalid("Initial pitch exceeds coordinate range"))?;
    }
    let expected = prepare(
        model,
        WriteProvenance::new(
            batch.provenance.actor.clone(),
            batch.provenance.source.clone(),
            batch.provenance.reason.clone(),
        ),
        &request,
        &plan,
    )?;
    let without_guards = |ops: Vec<Operation>| {
        ops.into_iter()
            .filter(|o| !matches!(o, Operation::GuardObjectRevision { .. }))
            .collect::<Vec<_>>()
    };
    if without_guards(expected.batch.operations) != without_guards(batch.operations.clone()) {
        return Err(invalid(
            "Initial create/CI/Pad/Net graph differs from its qualified producer",
        ));
    }
    let mut result = BTreeMap::new();
    for op in &batch.operations {
        if let Operation::SetComponentInstance {
            component_instance_id,
            component_instance,
            ..
        } = op
        {
            let mut ci = model
                .component_instances
                .get(component_instance_id)
                .ok_or_else(|| invalid("Missing initial CI source"))?
                .clone();
            ci.object_revision =
                serde_json::from_value(component_instance["object_revision"].clone())?;
            ci.placed_package_refs = vec![placements[component_instance_id].0.uuid];
            ci.placed_package_roles =
                serde_json::from_value(component_instance["placed_package_roles"].clone())?;
            ci.library_bindings =
                serde_json::from_value(component_instance["library_bindings"].clone())?;
            if result.insert(*component_instance_id, ci).is_some() {
                return Err(invalid("Duplicate initial CI operation"));
            }
        }
    }
    Ok(result)
}
