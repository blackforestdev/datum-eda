//! Explicit manual source choices use the existing guarded Pad producer.
use super::*;
use eda_engine::api::native_write::board_components::pad_materialization::{
    ComponentPadMaterializationRequest, PadLandPatternSource, PlacedPadCorrespondence,
    build_materialize_component_pad_sources, capture_component_library_pad,
};
use std::collections::BTreeSet;

impl SetBoardComponentPackageArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let report = match (self.component_instance, self.pin_pad_map) {
            (Some(instance), Some(map)) => mapped_materialization(
                &self.path,
                self.component_uuid,
                self.package_uuid,
                instance,
                map,
                &self.pad_correspondences,
                self.adopt_existing,
            )?,
            (None, None) => set_native_project_board_component_package(
                &self.path,
                self.component_uuid,
                self.package_uuid,
            )?,
            _ => anyhow::bail!("--component-instance and --pin-pad-map must be supplied together"),
        };
        Ok((
            render_report(
                format,
                &report,
                render_native_project_board_component_mutation_text,
            ),
            0,
        ))
    }
}

fn mapped_materialization(
    root: &Path,
    component_id: Uuid,
    package_id: Uuid,
    component_instance: Uuid,
    map_id: Uuid,
    correspondences: &[String],
    adopt_existing: bool,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let board =
        model.materialized_source_shard_value(eda_engine::substrate::SourceShardKind::BoardRoot)?;
    let mut placement: PlacedPackage =
        serde_json::from_value(board["packages"][component_id.to_string()].clone())
            .with_context(|| format!("board component not found: {component_id}"))?;
    placement.package = package_id;
    let object = model.objects.get(&map_id).context("PinPadMap not found")?;
    let map: eda_engine::pool::PinPadMap = serde_json::from_value(
        model.materialized_pool_object_revision(map_id, "pin_pad_maps", object.object_revision)?,
    )?;
    let footprint = map
        .footprint
        .context("selected PinPadMap has no Footprint; no default is inferred")?;
    let land_pattern = PadLandPatternSource::Footprint(footprint);
    let mut placed = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let correspondences = correspondences
        .iter()
        .map(|text| {
            let (old, target) = text.split_once('=').context(
                "pad correspondence must be <placed-pad-uuid>=<target-library-pad-uuid>",
            )?;
            let placed_pad_id = Uuid::parse_str(old).context("invalid placed Pad UUID")?;
            let target = Uuid::parse_str(target).context("invalid target library Pad UUID")?;
            anyhow::ensure!(
                placed.insert(placed_pad_id) && targets.insert(target),
                "pad correspondences must be one-to-one"
            );
            Ok(PlacedPadCorrespondence {
                placed_pad_id,
                target_library_pad: capture_component_library_pad(
                    &model,
                    package_id,
                    &land_pattern,
                    target,
                )?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let request = ComponentPadMaterializationRequest {
        placement: placement.clone(),
        land_pattern,
        component_instance: Some(component_instance),
        pin_pad_map: Some(map_id),
        correspondences,
        adopt_existing,
    };
    let prepared = build_materialize_component_pad_sources(
        &model,
        cli_provenance("explicit mapped board component materialization")?,
        &request,
    )?;
    commit_prepared(&mut model, root, prepared)?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_component_report(
        "materialize_mapped_board_component",
        &project,
        placement,
    ))
}
