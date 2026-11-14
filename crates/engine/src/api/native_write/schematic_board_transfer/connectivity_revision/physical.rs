//! Qualified frozen physical sources and source-bound copper impact.
use super::{
    AssignmentView, BoardConnectivityRevision, ContactReview, CopperImpact, RetainedCopper,
    RetainedCopperKind, ZoneFillReview, guard, invalid,
};
use crate::{
    board::{PlacedPackage, occupied_copper},
    connectivity::{BoardCopperSource, PhysicalQueryFailure},
    error::EngineError,
    schematic::Sheet,
    substrate::{
        ComponentInstanceAuthority, DesignModel, ElectricalOccurrence, ElectricalQueryFailure,
        ElectricalSelectionSnapshot, LibraryBindingRole, ObjectRevision, SourceShardKind, ZoneFill,
        ZoneFillState,
        component_pad_source::{
            self as pads, ComponentPadMaterializationStatus, ComponentPadSource,
            ComponentPadSourceBinding,
        },
    },
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub(super) struct QualifiedPhysical {
    pub sources: BTreeMap<Uuid, (Uuid, ComponentPadSourceBinding)>,
    pub terminals: BTreeSet<ElectricalOccurrence>,
}

fn validate_geometry(source: &BoardCopperSource) -> Result<(), EngineError> {
    let net = |id: Uuid| source.nets.get(&id).is_some_and(|value| value.uuid == id);
    for (id, track) in &source.tracks {
        if *id != track.uuid || !net(track.net) {
            return Err(invalid("Malformed Track source identity or Net assignment"));
        }
        occupied_copper::validate_track(track)
            .and_then(|_| occupied_copper::track_layers(&source.stackup, track))
            .map_err(|e| invalid(format!("Unsupported Track source: {e:?}")))?;
    }
    for (id, via) in &source.vias {
        if *id != via.uuid || !net(via.net) {
            return Err(invalid("Malformed Via source identity or Net assignment"));
        }
        occupied_copper::validate_via(via)
            .and_then(|_| occupied_copper::via_layers(&source.stackup, via))
            .map_err(|e| invalid(format!("Unsupported Via source: {e:?}")))?;
    }
    let ids: BTreeSet<_> = source
        .pads
        .keys()
        .chain(source.query_pads.keys())
        .copied()
        .collect();
    for id in ids {
        let pad = source
            .pad_terminal(id)
            .ok_or_else(|| invalid("Pad geometry missing"))?;
        if id != pad.uuid || !pad.net.is_some_and(net) {
            return Err(invalid("Malformed or unassigned Pad source"));
        }
        occupied_copper::validate_pad(pad)
            .and_then(|_| occupied_copper::pad_layer_groups(&source.stackup, pad))
            .map_err(|e| invalid(format!("Unsupported Pad source: {e:?}")))?;
    }
    for (id, zone) in &source.zones {
        if *id != zone.uuid || !net(zone.net) {
            return Err(invalid("Malformed Zone source identity or Net assignment"));
        }
        // This validates the authored boundary only. It supplies no fill copper.
        occupied_copper::validate_polygon(&zone.polygon)
            .and_then(|_| occupied_copper::conductive_layer(&source.stackup, zone.layer))
            .map_err(|e| invalid(format!("Unsupported Zone source: {e:?}")))?;
    }
    Ok(())
}

fn contact_failure(
    failure: PhysicalQueryFailure,
    states: &BTreeMap<Uuid, Option<ZoneFillState>>,
) -> Result<Option<PhysicalQueryFailure>, EngineError> {
    match failure {
        PhysicalQueryFailure::UnavailableFill { source_id, .. } => {
            let state = states
                .get(&source_id)
                .copied()
                .ok_or_else(|| invalid("Unavailable contact names an undeclared Zone source"))?;
            Ok(Some(PhysicalQueryFailure::UnavailableFill {
                source_id,
                state,
            }))
        }
        other => Err(invalid(format!(
            "Unsupported physical contact basis: {other:?}"
        ))),
    }
}

struct RecordedFill {
    value: ZoneFill,
    shard: Uuid,
    hash: String,
}

/// Resolver-derived Stale/Unfilled entries are not persisted generation records.
fn recorded_fill(model: &DesignModel, id: Uuid) -> Result<Option<RecordedFill>, EngineError> {
    let path = format!(".datum/zone_fills/{id}.json");
    let sources: Vec<_> = model
        .source_shards
        .iter()
        .filter(|source| source.kind == SourceShardKind::ZoneFill && source.relative_path == path)
        .collect();
    if sources.is_empty() {
        return Ok(None);
    }
    let [source] = sources.as_slice() else {
        return Err(invalid("Generated fill source ownership is ambiguous"));
    };
    let value: ZoneFill =
        serde_json::from_value(model.capture_source_shard_value(source.shard_id)?)?;
    if value.zone_id != id {
        return Err(invalid("Generated fill source names a different Zone"));
    }
    Ok(Some(RecordedFill {
        value,
        shard: source.shard_id,
        hash: source.content_hash.clone(),
    }))
}

/// Describe complete source IDs and their known/unknown authority. A stale fill
/// never becomes empty copper, a current outline, or a complete physical Net.
pub(super) fn impact(
    model: &DesignModel,
    snapshot: &ElectricalSelectionSnapshot,
    copper: &BoardCopperSource,
    review: &mut BoardConnectivityRevision,
) -> Result<(), EngineError> {
    if review.changes.is_empty() {
        return Ok(());
    }
    validate_geometry(copper)?;
    let affected: BTreeSet<_> = review
        .changes
        .iter()
        .flat_map(|change| [change.previous_board_net, change.board_net])
        .collect();
    let mut contacts = BTreeSet::new();
    let mut conservative = false;
    let mut fill_states = BTreeMap::new();
    for id in copper.zones.keys() {
        guard(model, &mut review.source_revisions, *id)?;
        let recorded = recorded_fill(model, *id)?;
        let fill = recorded.as_ref().map(|record| &record.value);
        let current = model.current_zone_fill(*id).is_some();
        let source_revision = review.source_revisions[id];
        let source_current = fill.is_some_and(|fill| {
            fill.state == ZoneFillState::Filled
                && fill.model_revision == model.model_revision
                && fill.source_zone_revision == source_revision
        });
        if current != source_current || (current && model.current_zone_fill(*id) != fill) {
            return Err(invalid(
                "Fill cache differs from its authenticated source; refresh the design before reviewing",
            ));
        }
        // Existing resolver availability rule, used only for stable disclosure.
        // No fill or complete physical-query authority is changed here.
        fill_states.insert(
            *id,
            fill.map(|fill| {
                if fill.model_revision != model.model_revision
                    || fill.source_zone_revision != source_revision
                {
                    ZoneFillState::Stale
                } else {
                    fill.state
                }
            }),
        );
        conservative |= !current;
        review.zone_fills.push(ZoneFillReview {
            source_id: *id,
            source_revision,
            fill_source_shard: recorded.as_ref().map(|record| record.shard),
            fill_source_hash: recorded.as_ref().map(|record| record.hash.clone()),
            current,
            recorded_state: fill.map(|f| f.state),
            recorded_model_revision: fill.map(|f| f.model_revision.clone()),
            recorded_zone_revision: fill.map(|f| f.source_zone_revision),
            invalidated_by_revision: !review.changes.is_empty(),
        });
    }
    for assignment in [AssignmentView::Previous, AssignmentView::Proposed] {
        let mut view = copper.clone();
        if assignment == AssignmentView::Proposed {
            for change in &review.changes {
                if let Some(pad) = view.pads.get_mut(&change.pad) {
                    pad.net = Some(change.board_net);
                }
                if let Some(pad) = view.query_pads.get_mut(&change.pad) {
                    pad.terminal_mut().net = Some(change.board_net);
                }
            }
        }
        for net in &affected {
            let unavailable = match snapshot.board_net_components(&view, *net) {
                Ok(groups) => match groups.contacts {
                    Ok(values) => {
                        for contact in values {
                            contacts.extend([contact.left.source_id, contact.right.source_id]);
                        }
                        None
                    }
                    Err(failure) => contact_failure(failure, &fill_states)?,
                },
                Err(ElectricalQueryFailure::Physical(failure)) => {
                    contact_failure(failure, &fill_states)?
                }
                Err(other) => {
                    return Err(invalid(format!("Unavailable physical source: {other:?}")));
                }
            };
            conservative |= unavailable.is_some();
            review.contact_basis.push(ContactReview {
                board_net: *net,
                assignment,
                unavailable,
            });
        }
    }
    for (kind, items) in [
        (
            RetainedCopperKind::Track,
            copper
                .tracks
                .iter()
                .map(|(id, t)| (*id, t.net))
                .collect::<Vec<_>>(),
        ),
        (
            RetainedCopperKind::Via,
            copper.vias.iter().map(|(id, v)| (*id, v.net)).collect(),
        ),
        (
            RetainedCopperKind::Zone,
            copper.zones.iter().map(|(id, z)| (*id, z.net)).collect(),
        ),
    ] {
        for (id, net) in items {
            let impact = if conservative {
                CopperImpact::ConservativeUnknownContacts
            } else if affected.contains(&net) {
                CopperImpact::QualifiedNetChange
            } else if contacts.contains(&id) {
                CopperImpact::ForeignContact
            } else if kind == RetainedCopperKind::Zone {
                CopperImpact::FillBasisInvalidated
            } else {
                continue;
            };
            guard(model, &mut review.source_revisions, id)?;
            guard(model, &mut review.source_revisions, net)?;
            review.retained_copper.push(RetainedCopper {
                source_id: id,
                kind,
                net: Some(net),
                impact,
            });
        }
    }
    review.retained_copper.sort_by_key(|row| row.source_id);
    Ok(())
}

pub(super) fn qualify(
    model: &DesignModel,
    schematic_sheet: &Sheet,
    board: &Value,
    board_id: Uuid,
    guards: &mut BTreeMap<Uuid, ObjectRevision>,
) -> Result<QualifiedPhysical, EngineError> {
    let packages: BTreeMap<Uuid, PlacedPackage> =
        serde_json::from_value(board["packages"].clone())?;
    let projection = pads::component_pad_projection(model, board)?;
    let mut physical = BTreeMap::new();
    let mut expected_terminals = BTreeSet::new();
    let mut owners = BTreeSet::new();
    for component in model.component_instances.values().filter(|ci| {
        ci.placed_symbol_refs
            .iter()
            .any(|id| schematic_sheet.symbols.contains_key(id))
            && !ci.placed_package_refs.is_empty()
    }) {
        let ([symbol_id], [package_id]) = (
            component.placed_symbol_refs.as_slice(),
            component.placed_package_refs.as_slice(),
        ) else {
            return Err(invalid(
                "Requires one root Symbol and one PCB counterpart per component",
            ));
        };
        let symbol = schematic_sheet
            .symbols
            .get(symbol_id)
            .ok_or_else(|| invalid("Unsupported sheet membership"))?;
        let placement = packages
            .get(package_id)
            .ok_or_else(|| invalid("Missing PCB counterpart"))?;
        if component.authority != ComponentInstanceAuthority::Authored
            || component.part_ref != symbol.part
            || component.part_ref != Some(placement.part)
            || !owners.insert(*package_id)
            || model
                .component_instances
                .values()
                .filter(|ci| {
                    ci.placed_symbol_refs.contains(symbol_id)
                        || ci.placed_package_refs.contains(package_id)
                })
                .count()
                != 1
        {
            return Err(invalid("Ambiguous or changed component/Part ownership"));
        }
        for (id, revision) in
            model.declared_bound_component_part_sources(component.id, placement.part)?
        {
            if model
                .objects
                .get(&id)
                .is_none_or(|object| object.object_revision != revision)
            {
                return Err(invalid(
                    "A bound library source changed; physical adoption is separate",
                ));
            }
            guard(model, guards, id)?;
        }
        for binding in component.library_bindings.values() {
            if model
                .objects
                .get(&binding.target_object_id)
                .is_none_or(|object| object.object_revision != binding.pinned_object_revision)
            {
                return Err(invalid("A pinned library definition changed"));
            }
            guard(model, guards, binding.target_object_id)?;
        }
        for id in [component.id, *symbol_id, *package_id] {
            guard(model, guards, id)?;
        }
        let maps: Vec<_> = component
            .library_bindings
            .values()
            .filter(|binding| binding.binding_role == LibraryBindingRole::PinPadMap)
            .collect();
        let [declared_map] = maps.as_slice() else {
            return Err(invalid("Component needs one explicit Pin/Pad Map choice"));
        };
        let map_object = &model.objects[&declared_map.target_object_id];
        if map_object.domain != "pool" || map_object.kind != "pin_pad_maps" {
            return Err(invalid("Declared Pin/Pad Map source has the wrong type"));
        }
        let map_value: crate::pool::PinPadMap =
            serde_json::from_value(model.capture_source_shard_value(map_object.source_shard_id)?)?;
        if map_value.uuid != declared_map.target_object_id
            || Some(map_value.part) != component.part_ref
        {
            return Err(invalid(
                "Declared Map source differs from the component Part",
            ));
        }
        let footprint = map_value
            .footprint
            .ok_or_else(|| invalid("Declared Map lacks an explicit Footprint"))?;
        let footprint_object = model
            .objects
            .get(&footprint)
            .ok_or_else(|| invalid("Declared Footprint source missing"))?;
        if footprint_object.domain != "pool" || footprint_object.kind != "footprints" {
            return Err(invalid("Declared Footprint source has the wrong type"));
        }
        let footprint_revision = footprint_object.object_revision;
        let footprints: Vec<_> = component
            .library_bindings
            .values()
            .filter(|binding| binding.binding_role == LibraryBindingRole::Footprint)
            .collect();
        match footprints.as_slice() {
            [] => {} // The explicit pinned Map is the existing physical-choice authority.
            [binding]
                if binding.target_object_id == footprint
                    && binding.pinned_object_revision == footprint_revision => {}
            _ => {
                return Err(invalid(
                    "Extra or mismatched Footprint declarations require a separate component review",
                ));
            }
        }
        let sources: Vec<ComponentPadSource> =
            serde_json::from_value(board["component_pads"][package_id.to_string()].clone())?;
        if sources.is_empty() || projection.unadopted_packages.contains(package_id) {
            return Err(invalid(
                "Counterpart lacks qualified materialized Pad sources",
            ));
        }
        for pad in sources {
            if projection.statuses.get(&pad.uuid)
                != Some(&ComponentPadMaterializationStatus::Current)
            {
                return Err(invalid(
                    "Placed Pad geometry or library source is stale or unavailable",
                ));
            }
            let binding = pad
                .source_binding
                .ok_or_else(|| invalid("Pad source binding missing"))?;
            if binding.package_id != *package_id
                || binding.board_id != board_id
                || binding.component_instance.as_ref().map(|r| r.object_id) != Some(component.id)
            {
                return Err(invalid("Pad source owner differs from its component"));
            }
            for pin in [
                &binding.part,
                &binding.package,
                &binding.land_pattern,
                &binding.library_pad,
                &binding.padstack,
            ] {
                super::super::field_revision::current_pin(model, guards, pin)?;
            }
            let map = binding
                .pin_pad_map
                .as_ref()
                .ok_or_else(|| invalid("Explicit Pin/Pad Map missing"))?;
            if (
                binding.land_pattern.object_id,
                binding.land_pattern.object_revision,
            ) != (footprint, footprint_revision)
                || (map.object_id, map.object_revision)
                    != (
                        declared_map.target_object_id,
                        declared_map.pinned_object_revision,
                    )
            {
                return Err(invalid(
                    "Placed Pads mix or differ from the component's declared physical choice",
                ));
            }
            super::super::field_revision::current_pin(model, guards, map)?;
            guard(model, guards, pad.uuid)?;
            if physical.insert(pad.uuid, (component.id, binding)).is_some() {
                return Err(invalid("Pad has multiple physical owners"));
            }
        }
        expected_terminals.extend(symbol.pins.iter().map(|pin| ElectricalOccurrence {
            class: "pins".into(),
            source_id: pin.uuid,
            instance_path: vec![],
        }));
    }
    if physical.is_empty() || owners.len() != packages.len() {
        return Err(invalid(
            "Every PCB counterpart needs supported root-component ownership",
        ));
    }
    Ok(QualifiedPhysical {
        sources: physical,
        terminals: expected_terminals,
    })
}
