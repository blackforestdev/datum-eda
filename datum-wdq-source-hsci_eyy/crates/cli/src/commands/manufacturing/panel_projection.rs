use anyhow::{Context, Result};
use eda_engine::substrate::{ArtifactProductionProjection, DesignModel, PanelProjection};
use uuid::Uuid;

use super::super::compute_source_hash_bytes;

pub(super) fn panel_pnp_production_projection(
    model: &DesignModel,
    manufacturing_plan_id: Option<Uuid>,
    board_or_panel: Uuid,
    panel_projection: &PanelProjection,
    row_count: usize,
) -> Result<ArtifactProductionProjection> {
    let payload = serde_json::json!({
        "contract": "datum.production_projection.panel_pnp.v1",
        "project_id": model.project.project_id,
        "model_revision": model.model_revision,
        "manufacturing_plan": manufacturing_plan_id,
        "board_or_panel": board_or_panel,
        "panel_projection": panel_projection.id,
        "panel_board_instance_count": panel_projection.board_instances.len(),
        "pnp_row_count": row_count,
    });
    projection_from_payload(
        "panel_pnp",
        "datum.production_projection.panel_pnp.v1",
        model,
        &payload,
        "panel PnP",
    )
}

pub(super) fn panel_drill_csv_production_projection(
    model: &DesignModel,
    manufacturing_plan_id: Option<Uuid>,
    board_or_panel: Uuid,
    panel_projection: &PanelProjection,
    row_count: usize,
) -> Result<ArtifactProductionProjection> {
    let payload = serde_json::json!({
        "contract": "datum.production_projection.panel_drill_csv.v1",
        "project_id": model.project.project_id,
        "model_revision": model.model_revision,
        "manufacturing_plan": manufacturing_plan_id,
        "board_or_panel": board_or_panel,
        "panel_projection": panel_projection.id,
        "panel_projection_revision": panel_projection.object_revision,
        "panel_board_instance_count": panel_projection.board_instances.len(),
        "drill_csv_row_count": row_count,
    });
    projection_from_payload(
        "panel_drill_csv",
        "datum.production_projection.panel_drill_csv.v1",
        model,
        &payload,
        "panel drill CSV",
    )
}

fn projection_from_payload(
    projection_kind: &str,
    projection_contract: &str,
    model: &DesignModel,
    payload: &serde_json::Value,
    label: &str,
) -> Result<ArtifactProductionProjection> {
    let bytes = serde_json::to_vec(payload)
        .with_context(|| format!("failed to serialize {label} projection"))?;
    Ok(ArtifactProductionProjection {
        projection_kind: projection_kind.to_string(),
        projection_contract: projection_contract.to_string(),
        model_revision: model.model_revision.clone(),
        byte_count: bytes.len(),
        sha256: compute_source_hash_bytes(&bytes),
    })
}
