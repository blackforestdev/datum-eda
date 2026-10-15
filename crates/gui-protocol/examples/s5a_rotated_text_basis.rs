//! Bounded read-only C04 basis extraction; no native input qualification.
#[path = "../../test-harness/src/development_workspace.rs"]
mod development_workspace;
use anyhow::{Context, Result, ensure};
use datum_gui_protocol::{LiveReviewRequest, load_board_editor_workspace_with_model};
use serde_json::json;

fn main() -> Result<()> {
    let root = development_workspace::path("target/s5a-c03-u5-annotations-v2").canonicalize()?;
    ensure!(
        std::env::args_os().len() == 1,
        "No alternate fixture arguments"
    );
    let cli = std::env::var_os("EDA_CLI_BIN").context("Pin the existing companion CLI")?;
    ensure!(
        std::path::Path::new(&cli).is_file(),
        "Pinned CLI is unavailable"
    );
    let request = LiveReviewRequest {
        project_root: root,
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let (workspace, model) = load_board_editor_workspace_with_model(&request)?;
    let scene = &workspace.scene;
    ensure!(scene.project_uuid == model.project.project_id.to_string());
    ensure!(scene.source_revision == model.model_revision.0);
    let source_id = "f044663d-9336-4218-bcad-c483dadf6a1b";
    let texts: Vec<_> = scene
        .board_texts
        .iter()
        .filter(|t| t.text_uuid == source_id || t.source_object_uuid.as_deref() == Some(source_id))
        .collect();
    ensure!(
        texts.len() == 1,
        "Target source Text is unavailable or ambiguous"
    );
    let geometries: Vec<_> = scene
        .board_text_geometries
        .iter()
        .filter(|g| g.object_id == texts[0].object_id)
        .collect();
    ensure!(
        geometries.len() == 1,
        "Target Text layout is unavailable or ambiguous"
    );
    let geometry = geometries[0];
    let corners = datum_gui_protocol::selection_geometry_catalog::layout_polygon(geometry)
        .context("Target Text has no exact captured layout basis")?;
    let rational_corners = corners.map(|point| {
        point.map(|r| json!({"numerator":r.numerator.to_string(), "denominator":r.denominator.to_string()}))
    });
    println!(
        "{}",
        serde_json::to_string(&json!({
            "schema":"datum.s5a.rotated-text-basis/v1",
            "project_id":scene.project_uuid,
            "model_revision":scene.source_revision,
            "source_text_uuid":source_id,
            "render_object_id":geometry.object_id,
            "world_transform_nm":geometry.world_transform_nm,
            "aligned_block_bbox_em_nm":geometry.block_bbox_em_nm,
            "layout_corners_nm":rational_corners
        }))?
    );
    Ok(())
}
