//! Complete same-pad geometry intent. Choices come from authenticated IDs and
//! the existing producer; a designer never reconstructs pool/Pad UUID graphs.
use super::*;
use crate::substrate::{
    ElectricalIdentity, ElectricalIdentityRecord, Operation,
    component_pad_source::{
        ComponentPadMaterializationStatus, PhysicalGeometryIntent, PhysicalPadContinuity,
        PhysicalPlacementIntent, PlannedPadSources,
    },
};
use serde_json::{Value, json};

pub(crate) struct GeometryPlan {
    pub(crate) intent: PhysicalGeometryIntent,
    pub(crate) updates: Vec<Value>,
    pub(crate) operations: Vec<Operation>,
}

pub(crate) fn plan_geometry(
    model: &DesignModel,
    provenance: WriteProvenance,
    target: &PlannedPadSources<'_>,
) -> Result<GeometryPlan, EngineError> {
    let mut users = BTreeMap::new();
    for impact in target.impacts() {
        let ci = impact
            .component_instance
            .ok_or_else(|| invalid("geometry update has no qualified component user"))?;
        if users
            .insert(impact.placed_package, (ci, impact.reference.clone()))
            .is_some()
        {
            return Err(invalid("geometry update has ambiguous placed ownership"));
        }
    }
    if users.is_empty() {
        return Ok(GeometryPlan {
            intent: PhysicalGeometryIntent { placements: vec![] },
            updates: vec![],
            operations: vec![],
        });
    }
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let projection = source::component_pad_projection(model, &board)?;
    let mut placements = Vec::new();
    let mut updates = Vec::new();
    let mut operations = Vec::new();
    let mut seen_components = BTreeSet::new();
    let mut electrical: BTreeMap<Uuid, (ElectricalIdentityRecord, ElectricalIdentityRecord)> =
        BTreeMap::new();
    for (package, (ci, reference)) in users {
        if !seen_components.insert(ci) {
            return Err(invalid("one component has multiple affected PCB packages"));
        }
        let placement: PlacedPackage =
            serde_json::from_value(board["packages"][package.to_string()].clone())?;
        let old: Vec<ComponentPadSource> =
            serde_json::from_value(board["component_pads"][package.to_string()].clone())?;
        if old.is_empty()
            || old.iter().any(|pad| {
                projection.statuses.get(&pad.uuid)
                    != Some(&ComponentPadMaterializationStatus::Current)
            })
        {
            return Err(invalid(
                "geometry update requires qualified current frozen pads",
            ));
        }
        let first = old[0]
            .source_binding
            .as_ref()
            .ok_or_else(|| invalid("unbound geometry cannot establish continuity"))?;
        let footprint = first.land_pattern.object_id;
        let map = first
            .pin_pad_map
            .as_ref()
            .ok_or_else(|| invalid("geometry update needs its exact PinPadMap"))?
            .object_id;
        let root = target.capture_root(footprint, T::PoolFootprint)?;
        let mut correspondences = Vec::new();
        let mut pads = Vec::new();
        let mut refs = BTreeSet::new();
        for pad in &old {
            let binding = pad
                .source_binding
                .as_ref()
                .ok_or_else(|| invalid("mixed unbound geometry is unsupported"))?;
            if binding
                .component_instance
                .as_ref()
                .is_none_or(|owner| owner.object_id != ci)
                || binding.land_pattern.object_id != footprint
                || binding.part.object_id != placement.part
                || binding.package.object_id != placement.package
                || binding
                    .pin_pad_map
                    .as_ref()
                    .is_none_or(|pin| pin.object_id != map)
            {
                return Err(invalid(
                    "geometry update changes physical or mapping identity",
                ));
            }
            let next = target.capture_pad(&root, binding.library_pad.object_id)?;
            target.continuity(&binding.library_pad, &next)?;
            correspondences.push(PlacedPadCorrespondence {
                placed_pad_id: pad.uuid,
                target_library_pad: next,
            });
            pads.push(PhysicalPadContinuity {
                placed_pad: pad.uuid,
                library_pad: binding.library_pad.object_id,
            });
            for id in [
                placement.part,
                placement.package,
                footprint,
                map,
                binding.padstack.object_id,
            ] {
                refs.insert(target.reference(id)?);
            }
        }
        correspondences.sort_by_key(|pair| pair.placed_pad_id);
        pads.sort_by_key(|pad| pad.placed_pad);
        let request = ComponentPadMaterializationRequest {
            placement,
            land_pattern: PadLandPatternSource::Footprint(footprint),
            component_instance: Some(ci),
            pin_pad_map: Some(map),
            correspondences,
            adopt_existing: false,
        };
        let prepared =
            super::build_with_sources(model, provenance.clone(), &request, Some(target))?;
        let mut review = Vec::new();
        let mut found = false;
        for op in prepared.batch.operations {
            match op {
                Operation::SetBoardPackagePackage {
                    ref materialized, ..
                } => {
                    let transition: ComponentPadTransition =
                        serde_json::from_value(materialized["pad_source_transition"].clone())?;
                    let next: Vec<ComponentPadSource> =
                        serde_json::from_value(materialized["component_pads"].clone())?;
                    if !transition.allocations.is_empty()
                        || !transition.retirements.is_empty()
                        || !transition.legacy_references.is_empty()
                        || next.iter().map(|p| p.uuid).collect::<BTreeSet<_>>()
                            != old.iter().map(|p| p.uuid).collect()
                    {
                        return Err(invalid(
                            "geometry update would allocate, retire or replace a Pad",
                        ));
                    }
                    for pad in &next {
                        let prior = old.iter().find(|prior| prior.uuid == pad.uuid).unwrap();
                        let before = &prior.source_binding.as_ref().unwrap().frozen_geometry;
                        let after = &pad.source_binding.as_ref().unwrap().frozen_geometry;
                        review.push(json!({"placed_pad":pad.uuid,"library_pad":pad.source_binding.as_ref().unwrap().library_pad.object_id,
                            "before":before,"after":after}));
                    }
                    found = true;
                    operations.push(op);
                }
                Operation::SetElectricalIdentity { previous, record } => {
                    if let Some((original, merged)) = electrical.get_mut(&record.id) {
                        if original != previous.as_ref() {
                            return Err(invalid("shared electrical preimages disagree"));
                        }
                        let (
                            ElectricalIdentity::NetRelationship {
                                evidence: destination,
                                ..
                            },
                            ElectricalIdentity::NetRelationship {
                                evidence: incoming, ..
                            },
                        ) = (&mut merged.identity, &record.identity)
                        else {
                            return Err(invalid("unsupported shared electrical identity"));
                        };
                        if destination.len() != incoming.len() {
                            return Err(invalid("shared electrical evidence length changed"));
                        }
                        for (destination, incoming) in destination.iter_mut().zip(incoming) {
                            if incoming.component_instance.object_id == ci {
                                *destination = incoming.clone();
                            }
                        }
                    } else {
                        electrical.insert(record.id, (*previous, record));
                    }
                }
                Operation::SetComponentInstance { .. } | Operation::GuardObjectRevision { .. } => {
                    operations.push(op)
                }
                _ => return Err(invalid("unexpected operation from geometry producer")),
            }
        }
        if !found {
            return Err(invalid("geometry producer omitted the affected placement"));
        }
        review.sort_by_key(|row| row["placed_pad"].as_str().unwrap().to_owned());
        updates.push(json!({"reference":reference,"placed_package":package,"component_instance":ci,"pads":review}));
        placements.push(PhysicalPlacementIntent {
            component_instance: ci,
            placed_package: package,
            target_assets: refs.into_iter().collect(),
            pads,
        });
    }
    for (_, (previous, record)) in electrical {
        operations.push(Operation::SetElectricalIdentity {
            previous: Box::new(previous),
            record,
        });
    }
    Ok(GeometryPlan {
        intent: PhysicalGeometryIntent { placements },
        updates,
        operations,
    })
}
