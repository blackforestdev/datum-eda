//! Native source projection uses the same immutable basis as selection authority.
use crate::BoardReviewSceneV1;
use crate::selection_resolution::{NativeSelectionResolution, SelectionAuthority};

pub fn project_native_schematic_scene(
    model: &eda_engine::substrate::DesignModel,
    root: &std::path::Path,
) -> anyhow::Result<Option<BoardReviewSceneV1>> {
    let authority = NativeSelectionResolution::capture(model)
        .map_err(|error| anyhow::anyhow!("native selection source unavailable: {error:?}"))?;
    let Some(source) = authority.snapshot().schematic_source() else {
        return Ok(None);
    };
    if source.sheets.is_empty() {
        return Ok(None);
    }
    // Prefer an actual top-level source occurrence over an instantiated
    // definition template. Qualified child membership stays complete even
    // while the native workspace has no child-occurrence projection.
    let instantiated: std::collections::BTreeSet<_> = source
        .sheet_instances
        .values()
        .filter_map(|instance| source.sheet_definitions.get(&instance.definition))
        .map(|definition| definition.root_sheet)
        .collect();
    let sheet = source
        .sheets
        .values()
        .filter(|sheet| !instantiated.contains(&sheet.uuid))
        .min_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    let schematic_file = root.join("schematic/schematic.json");
    let mut scene = if let Some(sheet) = sheet {
        crate::schematic_scene_import::project_schematic_sheet_source(
            source,
            sheet,
            &schematic_file,
        )?
    } else {
        crate::schematic_scene_import::project_schematic_source(source, &schematic_file)?
    };
    scene.project_uuid = authority.project().to_string();
    scene.project_name = model.project.name.clone();
    scene.source_revision = authority.revision().0.clone();
    Ok(Some(scene))
}
