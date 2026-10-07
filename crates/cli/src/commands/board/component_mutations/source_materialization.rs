//! PM057 source adoption stays separate from the existing placement write.
use super::*;
use eda_engine::api::native_write::board_components::pad_materialization::{
    ComponentPadMaterializationRequest, PadLandPatternSource, prepare_component_pad_materialization,
};
use eda_engine::api::native_write::board_layout::source_profile::build_adopt_board_source_profile;
use eda_engine::substrate::{SourceShardKind, SourceShardTaxon};
pub(super) fn source_payload(
    root: &Path,
    component: &PlacedPackage,
    materialized: &serde_json::Value,
    selected: Option<PadLandPatternSource>,
) -> Result<(serde_json::Value, Option<Uuid>)> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let payload = qualified_payload(&model, component, materialized, selected.clone())?;
    if payload.get("pad_source_transition").is_none() {
        return Ok((payload, None));
    }
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let adoption = if board["schema_version"].as_u64() != Some(4) {
        let board_id = serde_json::from_value(board["uuid"].clone())?;
        let write = build_adopt_board_source_profile(
            &model,
            cli_provenance("adopt profile-4 before placed Pad source write")?,
            board_id,
        )?;
        Some(
            commit_prepared(&mut model, root, write)?
                .transaction
                .transaction_id,
        )
    } else {
        None
    };
    Ok((
        qualified_payload(&model, component, materialized, selected)?,
        adoption,
    ))
}
pub(crate) fn component_pad_status(
    project: &LoadedNativeProject,
    component: &PlacedPackage,
) -> String {
    use eda_engine::substrate::component_pad_source::{
        ComponentPadMaterializationStatus as Status, ComponentPadSource,
    };
    let Ok(model) = ProjectResolver::new(&project.root).resolve() else {
        return "unverified".into();
    };
    let Some(pads) = project
        .board
        .component_pads
        .get(&component.uuid.to_string())
    else {
        return "unverified".into();
    };
    let mut status = Status::Current;
    if pads.is_empty() {
        return "unverified".into();
    }
    for pad in pads {
        let parsed = serde_json::to_value(pad)
            .ok()
            .and_then(|v| serde_json::from_value::<ComponentPadSource>(v).ok());
        let current = parsed.map_or(Status::Unverified, |p| {
            p.materialization_status(&model, component.part, component.package)
        });
        if current == Status::Stale {
            status = Status::Stale;
            break;
        }
        if current == Status::Unverified {
            status = current;
        }
    }
    serde_json::to_value(status)
        .unwrap()
        .as_str()
        .unwrap()
        .into()
}

pub(super) fn placement_payload(
    root: &Path,
    component: &PlacedPackage,
) -> Result<(serde_json::Value, Option<Uuid>)> {
    let mut project = load_native_project_with_resolved_board(root)?;
    clear_loaded_component_materialization(&mut project, &component.uuid.to_string());
    let selected =
        super::super::super::pool::materialize_supported_pool_package_graphics_with_source(
            &mut project,
            component,
        )?;
    let legacy = component_materialization_payload(&project, &component.uuid.to_string());
    source_payload(root, component, &legacy, selected)
}

pub(crate) fn place_native_project_board_component(
    root: &Path,
    part_uuid: Uuid,
    package_uuid: Uuid,
    reference: String,
    value: String,
    position: Point,
    layer: i32,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let component_uuid = Uuid::new_v4();
    let component = PlacedPackage {
        uuid: component_uuid,
        part: part_uuid,
        package: package_uuid,
        reference,
        value,
        position,
        rotation: 0,
        layer,
        locked: false,
    };
    let (materialized, adoption) = placement_payload(root, &component)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let prepared = build_place_board_package(
        &model,
        cli_provenance("place board component")?,
        &BoardPackagePlacement {
            package: component.clone(),
            materialized,
        },
    )?;
    commit_prepared(&mut model, root, prepared)?;

    let project = load_native_project_with_resolved_board(root)?;
    let mut report =
        native_project_board_component_report("place_board_component", &project, component);
    report.source_profile_adoption_transaction = adoption.map(|id| id.to_string());
    Ok(report)
}

/// Build-only source qualification for handoff/Proposal producers. No adoption
/// or persistence happens here; execution handles any separate profile boundary.
pub(super) fn qualified_payload(
    model: &eda_engine::substrate::DesignModel,
    component: &PlacedPackage,
    materialized: &serde_json::Value,
    selected: Option<PadLandPatternSource>,
) -> Result<serde_json::Value> {
    let Some(selected) = selected else {
        return Ok(materialized.clone());
    };
    // Preserve historical incomplete pool placements as explicitly unadopted.
    // A real Part producer enables the new semantics; missing data never mints authority.
    if !model.source_shards.iter().any(|s| {
        s.taxon == Some(SourceShardTaxon::PoolPart)
            && model
                .materialized_source_shard_value_by_relative_path(&s.relative_path)
                .is_ok_and(|v| v["uuid"] == serde_json::json!(component.part))
    }) {
        return Ok(materialized.clone());
    }
    let request = ComponentPadMaterializationRequest {
        placement: component.clone(),
        land_pattern: selected,
        component_instance: None,
        pin_pad_map: None,
        correspondences: vec![],
        adopt_existing: false,
    };
    Ok(prepare_component_pad_materialization(
        model,
        &request,
        materialized,
    )?)
}

/// Called only after a complete mutation/proposal has been prepared successfully.
/// Forward-only adoption is its own journal transaction, never a preview effect.
pub(crate) fn adopt_component_pad_profile_for_write(
    root: &Path,
    model: &mut eda_engine::substrate::DesignModel,
    batch: &eda_engine::substrate::OperationBatch,
) -> Result<Option<Uuid>> {
    let needed = batch.operations.iter().any(|op| match op {
        eda_engine::substrate::Operation::CreateBoardPackage { materialized, .. }
        | eda_engine::substrate::Operation::SetBoardPackagePackage { materialized, .. } => {
            materialized.get("pad_source_transition").is_some()
        }
        _ => false,
    });
    if !needed {
        return Ok(None);
    }
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    if board["schema_version"].as_u64() == Some(4) {
        return Ok(None);
    }
    let write = build_adopt_board_source_profile(
        model,
        cli_provenance("separate profile-4 adoption before placed Pad producer write")?,
        serde_json::from_value(board["uuid"].clone())?,
    )?;
    Ok(Some(
        commit_prepared(model, root, write)?
            .transaction
            .transaction_id,
    ))
}
