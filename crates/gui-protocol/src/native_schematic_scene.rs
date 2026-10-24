//! Native source projection uses the same immutable basis as selection authority.
use crate::BoardReviewSceneV1;
use crate::selection_resolution::{NativeSelectionResolution, SelectionAuthority};

pub fn project_native_schematic_documents(
    model: &eda_engine::substrate::DesignModel,
    root: &std::path::Path,
    layout: &crate::WorkspaceLayout,
) -> anyhow::Result<Option<crate::schematic_documents::SchematicDocuments>> {
    let authority = NativeSelectionResolution::capture(model)
        .map_err(|error| anyhow::anyhow!("native document source unavailable: {error:?}"))?;
    let Some(source) = authority.snapshot().schematic_source() else {
        return Ok(None);
    };
    let mut documents = crate::schematic_documents::SchematicDocuments::from_source(
        authority.project(),
        authority.revision().clone(),
        model.project.name.clone(),
        source.clone(),
    )?;
    documents.bind_initial_panes(layout);
    let keys: std::collections::BTreeSet<_> = documents.panes.values().cloned().collect();
    for key in keys {
        documents.prepare_scene(&key, root, model)?;
    }
    Ok(Some(documents))
}

/// Compatibility entry for the initial native document; all projection comes
/// from the same qualified document owner used by live multi-pane navigation.
pub fn project_native_schematic_scene(
    model: &eda_engine::substrate::DesignModel,
    root: &std::path::Path,
) -> anyhow::Result<Option<BoardReviewSceneV1>> {
    let layout = crate::WorkspaceLayout::board_schematic();
    Ok(
        project_native_schematic_documents(model, root, &layout)?.and_then(|documents| {
            layout
                .leaves()
                .into_iter()
                .find_map(|pane| documents.scene_for_pane(pane).cloned())
        }),
    )
}
