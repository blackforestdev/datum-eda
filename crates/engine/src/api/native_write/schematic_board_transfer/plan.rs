//! Qualified eligibility and complete logical-net coverage; no identity allocation.
use super::*;
use crate::pool::{Footprint, Part, PinPadMap};
use crate::schematic::PlacedSymbol;
use crate::substrate::{
    ComponentInstance, ComponentInstanceAuthority, DesignModel, ElectricalIdentity,
    ElectricalSelectionSnapshot, LibraryBindingRole, SourceShardKind, SourceShardTaxon,
    component_pad_source as source,
};
use std::collections::{BTreeMap, BTreeSet};

fn pool<T: serde::de::DeserializeOwned>(
    model: &DesignModel,
    id: Uuid,
    taxon: SourceShardTaxon,
) -> Result<T, EngineError> {
    let pin = source::capture_root(model, id, taxon)?;
    Ok(serde_json::from_value(source::source_value(model, &pin)?)?)
}

pub fn plan_transfer(
    model: &DesignModel,
    request: &TransferRequest,
) -> Result<TransferPlan, EngineError> {
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Native source unavailable: {e:?}")))?;
    let schematic = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("No native schematic"))?;
    super::super::schematic_edit::require_unique_occurrence(schematic, request.sheet, &[])?;
    let sheet = schematic
        .sheets
        .get(&request.sheet)
        .ok_or_else(|| invalid("Root sheet unavailable"))?;
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let mut choices = BTreeMap::new();
    for choice in &request.choices {
        if choices
            .insert(choice.component_instance, choice.pin_pad_map)
            .is_some()
        {
            return Err(invalid("A component has more than one explicit Map choice"));
        }
    }
    let mut seen = BTreeSet::new();
    let mut components = Vec::new();
    for symbol in sheet.symbols.values() {
        let owners: Vec<_> = model
            .component_instances
            .values()
            .filter(|ci| {
                ci.authority == ComponentInstanceAuthority::Authored
                    && ci.placed_symbol_refs.contains(&symbol.uuid)
            })
            .collect();
        let [ci] = owners.as_slice() else {
            components.push(unbound(symbol));
            continue;
        };
        if !seen.insert(ci.id) {
            continue;
        }
        let members: Vec<_> = ci
            .placed_symbol_refs
            .iter()
            .filter_map(|id| sheet.symbols.get(id))
            .collect();
        let mut row = TransferComponent {
            component_instance: Some(ci.id),
            symbols: ci.placed_symbol_refs.clone(),
            reference: members
                .iter()
                .map(|s| s.reference.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            value: None,
            part: ci.part_ref,
            package: None,
            footprint: None,
            pin_pad_map: choices.get(&ci.id).copied(),
            map_options: Vec::new(),
            connections: Vec::new(),
            existing_packages: ci.placed_package_refs.clone(),
            status: TransferStatus::Unresolved,
            reason: None,
        };
        let result = qualify(model, &snapshot, ci, &members, &mut row);
        if let Err(error) = result {
            row.reason = Some(error.to_string());
        }
        components.push(row);
    }
    if choices.keys().any(|id| !seen.contains(id)) {
        return Err(invalid(
            "Map choice targets a component outside the captured root sheet",
        ));
    }
    // A partial mapped subset must not be advertised as a complete Net. Use
    // actual canonical occurrences, including hierarchy, never reference names.
    let mut net_pins: BTreeMap<Uuid, BTreeSet<ElectricalOccurrence>> = BTreeMap::new();
    for item in snapshot
        .authored_sources()
        .map_err(|e| invalid(format!("Source inventory unavailable: {e:?}")))?
    {
        if item.occurrence.class == "pins" {
            if let Ok(net) = snapshot.schematic_member_net(&item.occurrence) {
                net_pins.entry(net).or_default().insert(item.occurrence);
            }
        }
    }
    loop {
        let covered: BTreeSet<_> = components
            .iter()
            .filter(|c| c.status == TransferStatus::Ready)
            .flat_map(|c| c.connections.iter().map(|p| p.schematic_terminal.clone()))
            .collect();
        let mut changed = false;
        for row in components
            .iter_mut()
            .filter(|c| c.status == TransferStatus::Ready)
        {
            if row.connections.iter().any(|p| {
                net_pins
                    .get(&p.logical_net)
                    .is_none_or(|pins| !pins.is_subset(&covered))
            }) {
                row.status = TransferStatus::Unresolved;
                row.reason = Some(
                    "Connected Net includes unselected, unresolved or unsupported component pins"
                        .into(),
                );
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    components.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then(a.component_instance.cmp(&b.component_instance))
    });
    Ok(TransferPlan {
        project_id: model.project.project_id,
        expected_model_revision: model.model_revision.clone(),
        expected_tip: crate::revision::transaction_tip(&model.journal),
        board_id: serde_json::from_value(board["uuid"].clone())?,
        sheet: request.sheet,
        requires_profile_adoption: board["schema_version"].as_u64() != Some(4),
        components,
    })
}

fn unbound(symbol: &PlacedSymbol) -> TransferComponent {
    TransferComponent {
        component_instance: None,
        symbols: vec![symbol.uuid],
        reference: symbol.reference.clone(),
        value: None,
        part: symbol.part,
        package: None,
        footprint: None,
        pin_pad_map: None,
        map_options: Vec::new(),
        connections: Vec::new(),
        existing_packages: Vec::new(),
        status: TransferStatus::Unresolved,
        reason: Some("Symbol needs one explicit authored component binding before transfer".into()),
    }
}

fn qualify(
    model: &DesignModel,
    snapshot: &ElectricalSelectionSnapshot,
    ci: &ComponentInstance,
    symbols: &[&PlacedSymbol],
    row: &mut TransferComponent,
) -> Result<(), EngineError> {
    if symbols.len() != ci.placed_symbol_refs.len() || symbols.is_empty() {
        return Err(invalid(
            "Component spans a different or unsupported sheet occurrence",
        ));
    }
    let part_id = ci
        .part_ref
        .ok_or_else(|| invalid("Component has no explicit Part"))?;
    let part_pin = source::capture_root(model, part_id, SourceShardTaxon::PoolPart)?;
    if !ci.library_bindings.values().any(|b| {
        b.binding_role == LibraryBindingRole::Part
            && b.target_object_id == part_id
            && b.pinned_object_revision == part_pin.object_revision
    }) {
        return Err(invalid(
            "Component Part binding is not current; initial transfer cannot perform a library update",
        ));
    }
    let part: Part = serde_json::from_value(source::source_value(model, &part_pin)?)?;
    row.package = Some(part.package);
    if !ci.placed_package_refs.is_empty() {
        if ci.placed_package_refs.len() != 1
            || !model.objects.contains_key(&ci.placed_package_refs[0])
        {
            return Err(invalid(
                "Declared board counterpart is missing or ambiguous",
            ));
        }
        row.status = TransferStatus::Existing;
        row.reason =
            Some("Already has a declared footprint; initial transfer leaves it unchanged".into());
        return Ok(());
    }
    let value = &symbols[0].value;
    if symbols.iter().any(|s| &s.value != value) {
        return Err(invalid(
            "Component Gate instance values disagree; make them agree before initial transfer",
        ));
    }
    row.value = Some(value.clone());
    for object in model
        .objects
        .values()
        .filter(|o| o.domain == "pool" && o.kind == "pin_pad_maps")
    {
        let Ok(map) = pool::<PinPadMap>(model, object.object_id, SourceShardTaxon::PoolPinPadMap)
        else {
            continue;
        };
        if map.part != part_id {
            continue;
        }
        let Some(id) = map.footprint else {
            continue;
        };
        let Ok(footprint) = pool::<Footprint>(model, id, SourceShardTaxon::PoolFootprint) else {
            continue;
        };
        if footprint.package == part.package {
            row.map_options.push(TransferMapOption {
                pin_pad_map: map.uuid,
                footprint: id,
                footprint_name: footprint.name,
            });
        }
    }
    row.map_options.sort_by_key(|m| m.pin_pad_map);
    let map_id = row
        .pin_pad_map
        .ok_or_else(|| invalid("Choose an explicit Pin/Pad Map and its footprint"))?;
    let map: PinPadMap = pool(model, map_id, SourceShardTaxon::PoolPinPadMap)?;
    if map.part != part_id {
        return Err(invalid("Selected Map belongs to another Part"));
    }
    let footprint_id = map
        .footprint
        .ok_or_else(|| invalid("Selected Map does not declare a footprint"))?;
    let footprint: Footprint = pool(model, footprint_id, SourceShardTaxon::PoolFootprint)?;
    if footprint.package != part.package {
        return Err(invalid("Map footprint belongs to another Package"));
    }
    if ci
        .library_bindings
        .values()
        .filter(|b| b.binding_role == LibraryBindingRole::PinPadMap)
        .any(|b| {
            b.target_object_id != map_id
                || source::capture_root(model, map_id, SourceShardTaxon::PoolPinPadMap)
                    .is_ok_and(|pin| pin.object_revision != b.pinned_object_revision)
        })
    {
        return Err(invalid(
            "Selected Map would replace a placed library binding",
        ));
    }
    if footprint.pads.is_empty() || map.mappings.len() != footprint.pads.len() {
        return Err(invalid(
            "This initial transfer needs a complete footprint pad mapping",
        ));
    }
    let mut connections = Vec::new();
    for (pad, cell) in &map.mappings {
        if !footprint.pads.contains_key(pad) {
            return Err(invalid("Map targets a Pad outside its footprint"));
        }
        let matches: Vec<_> = symbols
            .iter()
            .filter(|s| s.gate == Some(cell.gate))
            .flat_map(|s| &s.pins)
            .filter(|p| p.library_pin == Some(cell.pin))
            .collect();
        let [pin] = matches.as_slice() else {
            return Err(invalid(
                "Map cell has no unique placed Gate/Pin correspondence",
            ));
        };
        let occurrence = ElectricalOccurrence {
            class: "pins".into(),
            source_id: pin.uuid,
            instance_path: Vec::new(),
        };
        let logical_net = snapshot
            .schematic_member_net(&occurrence)
            .map_err(|e| invalid(format!("Mapped Pin has no current logical Net: {e:?}")))?;
        if model.electrical_identities.values().any(|r| matches!(&r.identity,
            ElectricalIdentity::NetRelationship { logical_net: id, board_net: Some(_), .. } if *id == logical_net)) {
            return Err(invalid("A connected Net already has PCB correspondence; incremental updates are separate"));
        }
        connections.push(TransferConnection {
            library_pad: *pad,
            library_pin: cell.pin,
            schematic_terminal: occurrence,
            logical_net,
        });
    }
    connections.sort_by_key(|p| p.library_pad);
    row.footprint = Some(footprint_id);
    row.connections = connections;
    row.status = TransferStatus::Ready;
    Ok(())
}
