//! Explicit PM057 Pad adoption/rematerialization/replacement through the existing
//! package source owner. Supplied correspondence, never names or geometry, keeps IDs.
use super::*;
use crate::pool::{Footprint, Package, Pad, Padstack, Part};
use crate::substrate::{
    SourceShardTaxon as T,
    component_pad_source::{
        self as source, ComponentPadSource, ComponentPadSourceBinding, FrozenPadGeometry,
        PinnedPadLibrarySource,
    },
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PadLandPatternSource {
    Footprint(Uuid),
    LegacyPackage,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedPadCorrespondence {
    pub placed_pad_id: Uuid,
    pub target_library_pad: PinnedPadLibrarySource,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentPadMaterializationRequest {
    pub placement: PlacedPackage,
    pub land_pattern: PadLandPatternSource,
    pub component_instance: Option<Uuid>,
    pub pin_pad_map: Option<Uuid>,
    pub correspondences: Vec<PlacedPadCorrespondence>,
    pub adopt_existing: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentPadTransition {
    pub target_part: PinnedPadLibrarySource,
    pub target_package: PinnedPadLibrarySource,
    pub target_land_pattern: PinnedPadLibrarySource,
    pub project_id: Uuid,
    pub board_id: Uuid,
    pub package_id: Uuid,
    pub correspondences: Vec<PlacedPadCorrespondence>,
    pub allocations: BTreeMap<Uuid, Uuid>,
    pub retirements: BTreeSet<Uuid>,
    pub legacy_references: BTreeSet<Uuid>,
    pub library_source_registrations: BTreeMap<Uuid, crate::substrate::DomainObject>,
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("component Pad materialization: {reason}"))
}
/// Build an existing placement's explicit adoption, replacement or
/// rematerialization. The returned journal payload carries every allocation,
/// retirement and old/new binding; no source write occurs while preparing.
pub fn build_materialize_component_pad_sources(
    model: &DesignModel,
    provenance: WriteProvenance,
    request: &ComponentPadMaterializationRequest,
) -> Result<PreparedWrite, EngineError> {
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    if board["schema_version"].as_u64() != Some(4) {
        return Err(invalid("separate profile-4 adoption is required"));
    }
    let key = request.placement.uuid.to_string();
    let current: PlacedPackage = serde_json::from_value(board["packages"][&key].clone())?;
    let mut expected = current.clone();
    expected.package = request.placement.package;
    if expected != request.placement {
        return Err(invalid(
            "request differs from the guarded placement/Part preimage",
        ));
    }
    let (pads, transition) = materialize(model, request, &board)?;
    let mut previous = serde_json::Map::new();
    // Preserve the full historical materialization, including unaffected graphics.
    for (name, map) in board
        .as_object()
        .ok_or_else(|| invalid("invalid BoardRoot"))?
    {
        if name.starts_with("component_")
            && let Some(value) = map.get(&key)
        {
            previous.insert(name.clone(), value.clone());
        }
    }
    let guards: BTreeSet<_> = pads
        .iter()
        .flat_map(|p| {
            let b = p.source_binding.as_ref().unwrap();
            [
                b.part.owner_id,
                b.package.owner_id,
                b.land_pattern.owner_id,
                b.padstack.owner_id,
            ]
        })
        .chain(pads.iter().filter_map(|p| {
            p.source_binding
                .as_ref()?
                .pin_pad_map
                .as_ref()
                .map(|p| p.owner_id)
        }))
        .collect();
    let mut next = previous.clone();
    next.insert("component_pads".into(), serde_json::to_value(pads)?);
    next.insert(
        "pad_source_transition".into(),
        serde_json::to_value(transition)?,
    );
    let mut composer = BatchComposer::compose(model, provenance)
        .push_op(Operation::SetBoardPackagePackage {
            package_id: request.placement.uuid,
            package_ref_id: request.placement.package,
            previous_materialized: serde_json::Value::Object(previous),
            materialized: serde_json::Value::Object(next),
        })
        .primary_object(request.placement.uuid);
    if let Some(id) = request.component_instance {
        let object = model
            .objects
            .get(&id)
            .ok_or_else(|| invalid("missing ComponentInstance registration"))?;
        composer = composer.push_op(Operation::GuardObjectRevision {
            object_id: id,
            expected_object_revision: object.object_revision,
        });
    }
    for id in guards {
        let object = model
            .objects
            .get(&id)
            .filter(|o| o.domain == "pool")
            .ok_or_else(|| invalid("missing library source-owner registration"))?;
        composer = composer.push_op(Operation::GuardObjectRevision {
            object_id: id,
            expected_object_revision: object.object_revision,
        });
    }
    composer.finish()
}

/// Capture the actual target producer for an explicitly requested correspondence.
/// This query returns a pin; it never supplies a placed-Pad identity or continuity.
pub fn capture_component_library_pad(
    model: &DesignModel,
    package_id: Uuid,
    land_pattern: &PadLandPatternSource,
    pad_id: Uuid,
) -> Result<PinnedPadLibrarySource, EngineError> {
    let owner = match land_pattern {
        PadLandPatternSource::Footprint(id) => source::capture_root(model, *id, T::PoolFootprint)?,
        PadLandPatternSource::LegacyPackage => {
            source::capture_root(model, package_id, T::PoolPackage)?
        }
    };
    source::capture_pad(model, &owner, pad_id)
}
/// Prepare journal-owned Pad sources alongside an existing materialization's
/// other geometry. This is build-only; profile adoption remains separate.
pub fn prepare_component_pad_materialization(
    model: &DesignModel,
    request: &ComponentPadMaterializationRequest,
    materialized: &serde_json::Value,
) -> Result<serde_json::Value, EngineError> {
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let (pads, transition) = materialize(model, request, &board)?;
    let mut value = materialized
        .as_object()
        .cloned()
        .ok_or_else(|| invalid("materialization must be an object"))?;
    value.insert("component_pads".into(), serde_json::to_value(pads)?);
    value.insert(
        "pad_source_transition".into(),
        serde_json::to_value(transition)?,
    );
    Ok(serde_json::Value::Object(value))
}

fn materialize(
    model: &DesignModel,
    request: &ComponentPadMaterializationRequest,
    board: &serde_json::Value,
) -> Result<(Vec<ComponentPadSource>, ComponentPadTransition), EngineError> {
    let placement = &request.placement;
    let board_id: Uuid = serde_json::from_value(board["uuid"].clone())?;
    let part_pin = source::capture_root(model, placement.part, T::PoolPart)?;
    let part: Part = serde_json::from_value(source::source_value(model, &part_pin)?)?;
    let package_pin = source::capture_root(model, placement.package, T::PoolPackage)?;
    let package: Package = serde_json::from_value(source::source_value(model, &package_pin)?)?;
    let (owner, pads): (PinnedPadLibrarySource, Vec<Pad>) = match request.land_pattern {
        PadLandPatternSource::Footprint(id) => {
            let owner = source::capture_root(model, id, T::PoolFootprint)?;
            let footprint: Footprint =
                serde_json::from_value(source::source_value(model, &owner)?)?;
            if footprint.package != placement.package {
                return Err(invalid("selected Footprint belongs to another Package"));
            }
            (owner, footprint.pads.into_values().collect())
        }
        PadLandPatternSource::LegacyPackage => {
            (package_pin.clone(), package.pads.into_values().collect())
        }
    };
    let ci = request
        .component_instance
        .map(|id| {
            let ci = model
                .component_instances
                .get(&id)
                .filter(|ci| {
                    ci.placed_package_refs.contains(&placement.uuid)
                        && ci.part_ref == Some(placement.part)
                })
                .ok_or_else(|| invalid("foreign or stale ComponentInstance relationship"))?;
            Ok::<_, EngineError>(crate::substrate::RevisionedRef {
                object_id: id,
                object_revision: ci.object_revision,
            })
        })
        .transpose()?;
    let pin_pad_map = request
        .pin_pad_map
        .map(|id| source::capture_root(model, id, T::PoolPinPadMap))
        .transpose()?;
    if let Some(map) = &pin_pad_map {
        let map_value = source::source_value(model, map)?;
        let selected_footprint = match request.land_pattern {
            PadLandPatternSource::Footprint(id) => Some(id),
            PadLandPatternSource::LegacyPackage => None,
        };
        if map_value["footprint"] != serde_json::json!(selected_footprint) {
            return Err(invalid("PinPadMap belongs to another land pattern"));
        }
        if map_value["part"] != serde_json::json!(placement.part) {
            return Err(invalid("PinPadMap belongs to another Part"));
        }
        if let Some(ci) = &ci
            && !model.component_instances[&ci.object_id]
                .library_bindings
                .values()
                .any(|b| {
                    b.binding_role == crate::substrate::LibraryBindingRole::PinPadMap
                        && b.target_object_id == map.object_id
                        && b.pinned_object_revision == map.object_revision
                })
        {
            return Err(invalid(
                "ComponentInstance does not pin the selected PinPadMap",
            ));
        }
    }
    let previous: Vec<ComponentPadSource> = serde_json::from_value(
        board["component_pads"][placement.uuid.to_string()]
            .as_array()
            .map_or_else(
                || serde_json::json!([]),
                |v| serde_json::Value::Array(v.clone()),
            ),
    )?;
    let old: BTreeMap<_, _> = previous.iter().map(|p| (p.uuid, p)).collect();
    if old.len() != previous.len() {
        return Err(invalid("duplicate old Pad references"));
    }
    let mut supplied = BTreeMap::new();
    let mut kept = BTreeSet::new();
    for pair in &request.correspondences {
        let old_pad = old
            .get(&pair.placed_pad_id)
            .ok_or_else(|| invalid("correspondence targets a foreign placed Pad"))?;
        old_pad.validate_frozen()?;
        let binding = old_pad.source_binding.as_ref().unwrap();
        if binding.package_id != placement.uuid
            || binding.project_id != model.project.project_id
            || binding.board_id != board_id
        {
            return Err(invalid("foreign old source binding"));
        }
        source::continuity(model, &binding.library_pad, &pair.target_library_pad)?;
        if pair.target_library_pad.owner_id != owner.object_id
            || pair.target_library_pad.relative_path != owner.relative_path
            || supplied
                .insert(pair.target_library_pad.object_id, pair.placed_pad_id)
                .is_some()
            || !kept.insert(pair.placed_pad_id)
        {
            return Err(invalid(
                "correspondence is not one-to-one in the selected land pattern",
            ));
        }
    }
    if request.adopt_existing
        && (!request.correspondences.is_empty()
            || previous.iter().any(|p| p.source_binding.is_some()))
    {
        return Err(invalid(
            "adoption requires legacy unbound geometry and fresh allocations",
        ));
    }
    let mut transition = ComponentPadTransition {
        target_part: part_pin.clone(),
        target_package: package_pin.clone(),
        target_land_pattern: owner.clone(),
        project_id: model.project.project_id,
        board_id,
        package_id: placement.uuid,
        correspondences: request.correspondences.clone(),
        allocations: BTreeMap::new(),
        retirements: old
            .iter()
            .filter(|(id, p)| p.source_binding.is_some() && !kept.contains(id))
            .map(|(id, _)| *id)
            .collect(),
        library_source_registrations: BTreeMap::new(),
        legacy_references: old
            .values()
            .filter(|p| p.source_binding.is_none())
            .map(|p| p.uuid)
            .collect(),
    };
    let mut result = Vec::new();
    let mut library_ids = BTreeSet::new();
    for pad in pads {
        if !library_ids.insert(pad.uuid) {
            return Err(invalid("duplicate library Pad"));
        }
        let library_pad = source::capture_pad(model, &owner, pad.uuid)?;
        if let Some(pair) = request
            .correspondences
            .iter()
            .find(|p| p.target_library_pad.object_id == pad.uuid)
            && pair.target_library_pad != library_pad
        {
            return Err(invalid(
                "correspondence target source pin differs from the actual producer",
            ));
        }
        let padstack_pin = source::capture_root(model, pad.padstack, T::PoolPadstack)?;
        let padstack: Padstack =
            serde_json::from_value(source::source_value(model, &padstack_pin)?)?;
        let aperture = padstack
            .aperture
            .clone()
            .ok_or_else(|| invalid("Padstack has no exact supported aperture"))?;
        let geometry = FrozenPadGeometry {
            name: pad.name.clone(),
            position: pad.position,
            layer: pad.layer,
            aperture: aperture.clone(),
            drill_nm: padstack.drill_nm,
            layer_span: padstack.layer_span,
            plated: padstack.plated,
        };
        let (shape, diameter_nm, width_nm, height_nm) = match aperture {
            crate::pool::PadstackAperture::Circle { diameter_nm } => {
                (crate::board::PadShape::Circle, diameter_nm, 0, 0)
            }
            crate::pool::PadstackAperture::Rect {
                width_nm,
                height_nm,
            } => (crate::board::PadShape::Rect, 0, width_nm, height_nm),
        };
        if request.adopt_existing {
            let matches: Vec<_> = previous
                .iter()
                .filter(|old| {
                    old.uuid == pad.uuid
                        && old.name == pad.name
                        && old.position == pad.position
                        && old.padstack == pad.padstack
                        && old.layer == pad.layer
                        && old.drill_nm == padstack.drill_nm
                        && old.shape == Some(shape)
                        && old.diameter_nm == diameter_nm
                        && old.width_nm == width_nm
                        && old.height_nm == height_nm
                })
                .collect();
            if matches.len() != 1 {
                return Err(invalid(
                    "legacy frozen geometry has no exact pinned source correspondence",
                ));
            }
        }
        let uuid = if let Some(id) = supplied.get(&pad.uuid) {
            *id
        } else {
            let id = Uuid::new_v4();
            if model.objects.contains_key(&id)
                || model.journal.iter().any(|t| t.diff.created.contains(&id))
            {
                return Err(invalid(
                    "allocation collides with current or historical identity",
                ));
            }
            transition.allocations.insert(pad.uuid, id);
            id
        };
        result.push(ComponentPadSource {
            uuid,
            name: pad.name,
            position: pad.position,
            padstack: pad.padstack,
            layer: pad.layer,
            drill_nm: padstack.drill_nm,
            shape: Some(shape),
            diameter_nm,
            width_nm,
            height_nm,
            source_binding: Some(ComponentPadSourceBinding {
                project_id: model.project.project_id,
                board_id,
                package_id: placement.uuid,
                materialized_against: model.model_revision.clone(),
                part: part_pin.clone(),
                package: package_pin.clone(),
                land_pattern: owner.clone(),
                library_pad,
                padstack: padstack_pin,
                component_instance: ci.clone(),
                pin_pad_map: pin_pad_map.clone(),
                frozen_geometry: geometry,
            }),
        });
    }
    if supplied.keys().any(|id| !library_ids.contains(id))
        || (request.adopt_existing && previous.len() != result.len())
    {
        return Err(invalid(
            "correspondence does not account for the whole materialization",
        ));
    }
    if part.uuid != placement.part {
        return Err(invalid("Part source UUID mismatch"));
    }
    for pad in &result {
        let p = &pad.source_binding.as_ref().unwrap().library_pad;
        if model
            .objects
            .get(&p.object_id)
            .is_none_or(|o| o.domain != "pool" || o.source_shard_id != p.source_shard_id)
        {
            transition.library_source_registrations.insert(
                p.object_id,
                crate::substrate::DomainObject {
                    object_id: p.object_id,
                    object_revision: p.object_revision,
                    source_shard_id: p.source_shard_id,
                    domain: "pool".into(),
                    kind: p.object_id.to_string(),
                },
            );
        }
    }
    result.sort_by_key(|p| p.uuid);
    for pad in &result {
        pad.validate_frozen()?;
    }
    Ok((result, transition))
}

#[cfg(test)]
mod tests;
