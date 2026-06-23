use super::*;
use crate::NativeProjectProductionProjectionView;
use crate::command_project::load_native_project_with_resolved_board_and_model;
use eda_engine::import::ids_sidecar::compute_source_hash_bytes;

pub(super) struct NativeExcellonDrillProjection {
    pub(super) project_root: String,
    pub(super) board_path: String,
    pub(super) excellon: String,
    pub(super) via_count: usize,
    pub(super) component_pad_count: usize,
    pub(super) hit_count: usize,
    pub(super) tool_count: usize,
    pub(super) tools: Vec<NativeProjectExcellonDrillToolView>,
    pub(super) production_projection: NativeProjectProductionProjectionView,
}

fn production_projection_view(
    projection_kind: &str,
    projection_contract: &str,
    model_revision: String,
    bytes: &[u8],
) -> NativeProjectProductionProjectionView {
    NativeProjectProductionProjectionView {
        projection_kind: projection_kind.to_string(),
        projection_contract: projection_contract.to_string(),
        model_revision,
        byte_count: bytes.len(),
        sha256: compute_source_hash_bytes(bytes),
    }
}

pub(super) fn render_native_project_excellon_drill_projection(
    root: &Path,
) -> Result<NativeExcellonDrillProjection> {
    let (project, _) = load_native_project_with_resolved_board_and_model(root)?;
    let drill_hits = query_native_project_drill_hits(&project)?;
    render_native_project_excellon_drill_projection_from_hits(
        root,
        &drill_hits,
        "excellon_drill",
        "datum.production_projection.excellon_drill.v1",
    )
}

pub(super) fn render_native_project_excellon_drill_projection_from_hits(
    root: &Path,
    drill_hits: &[NativeDrillHit],
    projection_kind: &str,
    projection_contract: &str,
) -> Result<NativeExcellonDrillProjection> {
    let (project, model) = load_native_project_with_resolved_board_and_model(root)?;
    let (via_count, component_pad_count) = drill_hit_counts(&drill_hits);
    let tools = build_excellon_tool_views_for_drill_hits(&drill_hits);
    let tool_count = tools.len();
    let excellon = render_excellon_for_drill_hits(&drill_hits)
        .context("failed to render native board drill hits as Excellon drill")?;
    let production_projection = production_projection_view(
        projection_kind,
        projection_contract,
        model.model_revision.0,
        excellon.as_bytes(),
    );
    Ok(NativeExcellonDrillProjection {
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        excellon,
        via_count,
        component_pad_count,
        hit_count: drill_hits.len(),
        tool_count,
        tools,
        production_projection,
    })
}
