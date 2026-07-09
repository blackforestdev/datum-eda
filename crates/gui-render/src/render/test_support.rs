#[allow(dead_code)]
fn sample_artifact_preview_primitives()
-> Vec<datum_gui_protocol::ProductionArtifactPreviewPrimitive> {
    use datum_gui_protocol::{
        ProductionArtifactPreviewPoint as P, ProductionArtifactPreviewPrimitive as Prim,
    };
    vec![
        Prim {
            kind: "stroke".to_string(),
            aperture_diameter_nm: Some(250_000),
            aperture_width_nm: None,
            aperture_height_nm: None,
            tool: None,
            diameter_mm: None,
            points: vec![
                P { x_nm: 0, y_nm: 0 },
                P {
                    x_nm: 1_000_000,
                    y_nm: 1_000_000,
                },
            ],
        },
        Prim {
            kind: "flash".to_string(),
            aperture_diameter_nm: Some(400_000),
            aperture_width_nm: None,
            aperture_height_nm: None,
            tool: None,
            diameter_mm: None,
            points: vec![P {
                x_nm: 500_000,
                y_nm: 250_000,
            }],
        },
    ]
}

#[allow(dead_code)]
fn panel_vertices_without_artifact_preview(mut state: ReviewWorkspaceState) -> usize {
    if let Some(artifact) = state.production.focused_artifact.as_mut()
        && let Some(preview) = artifact.focused_preview.as_mut()
    {
        preview.primitives.clear();
    }
    PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &RetainedScene::from_workspace(&state, 1280, 800),
    )
    .panel_vertices()
    .len()
}

#[allow(dead_code)]
fn artifact_preview_adds_panel_vertices(
    prepared: &PreparedScene,
    state: ReviewWorkspaceState,
) -> bool {
    prepared.panel_vertices().len() > panel_vertices_without_artifact_preview(state)
}

#[allow(dead_code)]
fn prepared_has_artifact_preview_controls(prepared: &PreparedScene) -> bool {
    let has_zoom = prepared
        .hit_regions
        .iter()
        .any(|region| matches!(region.target, HitTarget::ArtifactPreviewZoomIn));
    let has_geometry = prepared
        .hit_regions
        .iter()
        .any(|region| matches!(region.target, HitTarget::ToggleArtifactPreviewGeometry));
    let has_viewport = prepared
        .hit_regions
        .iter()
        .any(|region| matches!(region.target, HitTarget::ArtifactPreviewViewport));
    has_zoom && has_geometry && has_viewport
}

#[allow(dead_code)]
fn outputs_dock_renders_csv_preview_table(mut state: ReviewWorkspaceState) -> bool {
    if let Some(artifact) = state.production.focused_artifact.as_mut()
        && let Some(preview) = artifact.focused_preview.as_mut()
    {
        preview.preview_kind = "bom_csv".to_string();
        preview.primitive_count = 0;
        preview.primitives.clear();
        preview.row_count = Some(2);
        preview.csv_columns = vec!["ref".to_string(), "value".to_string()];
        preview.csv_rows = vec![
            vec!["R1".to_string(), "10k".to_string()],
            vec!["C1".to_string(), "100n".to_string()],
        ];
    }
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &RetainedScene::from_workspace(&state, 1280, 800),
    );
    let text = prepared
        .text_runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<Vec<_>>();
    text.iter().any(|value| value.contains("TABLE 2 ROWS"))
        && text.iter().any(|value| value.contains("R1 | 10k"))
}

