//! Native source projection uses the same immutable basis as selection authority.
use crate::selection_resolution::{NativeSelectionResolution, SelectionAuthority};
use crate::{BoardReviewSceneV1, LiveReviewRequest};
use eda_engine::substrate::ProjectResolver;

pub(crate) fn load(request: &LiveReviewRequest) -> anyhow::Result<Option<BoardReviewSceneV1>> {
    let model = ProjectResolver::new(&request.project_root).resolve()?;
    project_native_schematic_scene(&model, &request.project_root)
}

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
    let mut scene = crate::schematic_scene_import::project_schematic_source(
        source,
        &root.join("schematic/schematic.json"),
    )?;
    scene.project_uuid = authority.project().to_string();
    scene.project_name = model.project.name.clone();
    scene.source_revision = authority.revision().0.clone();
    Ok(Some(scene))
}
