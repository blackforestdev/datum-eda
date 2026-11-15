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

#[cfg(test)]
mod tests {
    #[test]
    fn product_genesis_binds_a_ready_main_schematic_in_the_default_board_schematic_layout() {
        use eda_engine::api::native_write::genesis::{
            GenesisSpec, bootstrap_native_project_with_units_receipt_value,
        };
        use eda_engine::ir::units::{
            PreFeatureProjectUnitsMigration, migrate_pre_feature_project_units,
        };
        use eda_engine::substrate::{ProjectResolver, SourceShardKind};
        let root =
            std::env::temp_dir().join(format!("datum-initial-documents-{}", uuid::Uuid::new_v4()));
        let PreFeatureProjectUnitsMigration::Create { profile, receipt } =
            migrate_pre_feature_project_units(None).unwrap()
        else {
            unreachable!()
        };
        bootstrap_native_project_with_units_receipt_value(
            &root,
            GenesisSpec {
                project_name: "Blank slate".into(),
                existing_ids: None,
            },
            profile,
            serde_json::to_value(receipt).unwrap(),
        )
        .unwrap();
        let model = ProjectResolver::new(&root).resolve().unwrap();
        let layout = crate::WorkspaceLayout::board_schematic();
        let documents = super::project_native_schematic_documents(&model, &root, &layout)
            .unwrap()
            .unwrap();
        assert_eq!(documents.rows.len(), 1);
        assert_eq!(documents.rows[0].name, "Main");
        let pane = layout
            .leaves()
            .into_iter()
            .find(|pane| layout.content_for(*pane) == Some(crate::PaneContent::Schematic))
            .unwrap();
        let key = documents
            .panes
            .get(&pane)
            .expect("Initial schematic pane must be bound");
        assert!(key.instance_path.is_empty());
        let scene = documents
            .scene_for_pane(pane)
            .expect("Initial schematic must have a qualified scene");
        assert_eq!(scene.schematic_document.as_ref(), Some(key));
        assert_eq!(scene.source_revision, model.model_revision.0);
        assert_eq!(
            crate::schematic_authoring::SchematicAuthoring::from_model(&model)
                .unwrap()
                .sheet,
            Some(key.source_sheet)
        );
        assert!(
            model
                .materialized_source_shard_value(SourceShardKind::BoardRoot)
                .is_ok()
        );
        assert!(model.journal.is_empty());
        let cold = ProjectResolver::new(&root).resolve().unwrap();
        let reopened = super::project_native_schematic_documents(&cold, &root, &layout)
            .unwrap()
            .unwrap();
        assert_eq!(reopened.panes.get(&pane), Some(key));
        std::fs::remove_dir_all(root).unwrap();
    }
}
