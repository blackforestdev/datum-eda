//! Proposal rebuild and receipt over the same explicit Gate materialization.
use super::*;
use crate::api::native_write::context::{PreparedWrite, WriteProvenance, build_batch};
use crate::substrate::CommitSource;

pub(in crate::api::native_write) fn build(
    model: &DesignModel,
    request: &GatePlacementRequest,
) -> Result<PreparedWrite, EngineError> {
    let preview = preview(model, request)?;
    let owner = owner_symbol(model, request.component_instance)?;
    let (id, previous) = component::current_set(model, &BTreeSet::from([owner]))?
        .remove(&owner)
        .ok_or_else(|| invalid("Component source unavailable"))?;
    let mut next = previous.clone();
    next["object_revision"] = json!(
        model.objects[&id]
            .object_revision
            .0
            .checked_add(1)
            .ok_or_else(|| invalid("Component revision overflow"))?
    );
    next["placed_symbol_refs"]
        .as_array_mut()
        .unwrap()
        .push(json!({"object_id":preview.symbol.uuid,"object_revision":0}));
    if next.get("placed_symbol_roles").is_none() {
        next["placed_symbol_roles"] = json!({});
    }
    next["placed_symbol_roles"]
        .as_object_mut()
        .ok_or_else(|| invalid("Component roles are malformed"))?
        .insert(preview.symbol.uuid.to_string(), json!({"role":"gate"}));
    let operations = vec![
        Operation::CreateSchematicSymbol {
            sheet_id: request.sheet,
            symbol_id: preview.symbol.uuid,
            symbol: serde_json::to_value(&preview.symbol)?,
        },
        Operation::SetComponentInstance {
            component_instance_id: id,
            previous_component_instance: previous,
            component_instance: next,
        },
    ];
    let mut batch = build_batch(
        model,
        WriteProvenance::new("datum-gui", CommitSource::Manual, REASON),
        operations,
    )?;
    // Electrical births derive their IDs from the batch. Bind it before planning
    // topology so the immutable proposal rebuild allocates the same identities.
    batch.batch_id = request.proposal_id;
    let mut prepared = super::super::schematic_edit::prepare_native_birth(
        model,
        PreparedWrite {
            batch,
            primary_object_id: Some(preview.symbol.uuid),
        },
    )?;
    // Draft creation composes adopted topology too. Rebuild the same complete
    // operation set, retaining the composer's explicit-adoption boundary.
    prepared.batch = crate::substrate::compose_electrical_transaction(
        model,
        prepared.batch,
        crate::substrate::TransactionKind::Normal,
    )?;
    Ok(prepared)
}
struct Recorded {
    request: GatePlacementRequest,
    symbol: PlacedSymbol,
    previous_member: Uuid,
}
fn recorded(proposal: &Proposal) -> Result<Recorded, EngineError> {
    let mut source = None;
    let mut component = None;
    for op in &proposal.batch.operations {
        match op {
            Operation::CreateSchematicSymbol {
                sheet_id,
                symbol_id,
                symbol,
            } if source.is_none() => source = Some((*sheet_id, *symbol_id, symbol)),
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance,
                component_instance,
            } if component.is_none() => {
                component = Some((
                    *component_instance_id,
                    previous_component_instance,
                    component_instance,
                ))
            }
            Operation::GuardObjectRevision { .. }
            | Operation::CreateElectricalIdentity { .. }
            | Operation::SetElectricalIdentity { .. } => {}
            _ => {
                return Err(invalid(
                    "Gate placement contains unrelated or duplicate operations",
                ));
            }
        }
    }
    let (sheet, id, payload) = source.ok_or_else(|| invalid("Gate source unavailable"))?;
    let (component_instance, old, next) =
        component.ok_or_else(|| invalid("Gate owner unavailable"))?;
    let symbol: PlacedSymbol = serde_json::from_value(payload.clone())?;
    if symbol.uuid != id
        || old["uuid"] != json!(component_instance)
        || old["placed_package_refs"]
            .as_array()
            .is_none_or(|r| !r.is_empty())
        || old["placed_symbol_refs"]
            .as_array()
            .is_none_or(|r| r.is_empty())
        || old["part_ref"]["object_id"] != json!(symbol.part)
    {
        return Err(invalid("Gate owner/source identity differs"));
    }
    let mut expected = old.clone();
    expected["object_revision"] = json!(
        old["object_revision"]
            .as_u64()
            .and_then(|v| v.checked_add(1))
            .ok_or_else(|| invalid("Component revision unavailable"))?
    );
    expected["placed_symbol_refs"]
        .as_array_mut()
        .unwrap()
        .push(json!({"object_id":id,"object_revision":0}));
    if expected.get("placed_symbol_roles").is_none() {
        expected["placed_symbol_roles"] = json!({});
    }
    expected["placed_symbol_roles"][id.to_string()] = json!({"role":"gate"});
    if expected != *next {
        return Err(invalid(
            "Gate placement changes unrelated component payload",
        ));
    }
    Ok(Recorded {
        previous_member: serde_json::from_value(old["placed_symbol_refs"][0]["object_id"].clone())?,
        request: GatePlacementRequest {
            proposal_id: proposal.proposal_id,
            component_instance,
            sheet,
            instance_path: vec![],
            gate: symbol.gate.ok_or_else(|| invalid("Gate unavailable"))?,
            library_symbol: symbol
                .lib_id
                .as_deref()
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| invalid("Gate native body unavailable"))?,
            reference: symbol.reference.clone(),
            value: symbol.value.clone(),
            x_nm: symbol.position.x,
            y_nm: symbol.position.y,
            rotation_deg: symbol.rotation,
            mirrored: symbol.mirrored,
        },
        symbol,
    })
}
pub(in crate::api::native_write) fn validate_recorded(
    proposal: &Proposal,
) -> Result<(), EngineError> {
    recorded(proposal).map(|_| ())
}
pub(in crate::api::native_write) fn summary(
    model: &DesignModel,
    proposal: &Proposal,
    current: bool,
) -> Result<GatePlacementSummary, EngineError> {
    let Recorded {
        mut request,
        symbol,
        previous_member,
    } = recorded(proposal)?;
    if proposal.status == crate::substrate::ProposalStatus::Rejected
        && proposal.prepared_against != model.model_revision
    {
        return Err(invalid("Rejected Gate source revision is unavailable"));
    }
    let (sheet, path) = occurrence(
        model,
        if model.objects.contains_key(&previous_member) {
            previous_member
        } else {
            symbol.uuid
        },
    )?;
    if sheet != request.sheet {
        return Err(invalid("Gate source sheet changed"));
    }
    request.instance_path = path;

    if current {
        if proposal.prepared_against != model.model_revision
            || proposal.batch.expected_model_revision.as_ref() != Some(&model.model_revision)
        {
            return Err(invalid("Design changed; prepare Gate placement again"));
        }
        if build(model, &request)?.batch.operations != proposal.batch.operations {
            return Err(invalid(
                "Prepared Gate placement differs from current owner or library",
            ));
        }
    }
    let part = symbol
        .part
        .ok_or_else(|| invalid("Gate Part unavailable"))?;
    let entity = symbol
        .entity
        .ok_or_else(|| invalid("Gate Entity unavailable"))?;
    let body = materialized_pool_object(model, request.library_symbol, "symbols")?;
    let entity_raw = materialized_pool_object(model, entity, "entities")?;
    let gate = &entity_raw["gates"][request.gate.to_string()];
    if gate["symbol"] != json!(request.library_symbol) || gate["unit"] != body["unit"] {
        return Err(invalid("Gate library declaration differs"));
    }
    Ok(GatePlacementSummary {
        component_instance: request.component_instance,
        sheet: request.sheet,
        instance_path: request.instance_path,
        part,
        entity,
        choice: GatePlacementChoice {
            gate: request.gate,
            gate_name: gate["name"].as_str().unwrap_or("").into(),
            unit: id(&body, "unit")?,
            library_symbol: request.library_symbol,
            symbol_name: body["name"].as_str().unwrap_or("").into(),
        },
        symbol: symbol.uuid,
        reference: request.reference,
        value: request.value,
        position: symbol.position,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
    })
}
