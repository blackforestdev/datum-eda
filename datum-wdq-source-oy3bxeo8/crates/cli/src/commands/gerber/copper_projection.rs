use super::*;
use crate::NativeProjectProductionProjectionView;
use crate::load_native_project_with_resolved_board_and_model;

pub(super) struct NativeGerberCopperProjection {
    pub(super) project_root: String,
    pub(super) board_path: String,
    pub(super) layer: i32,
    pub(super) gerber: String,
    pub(super) context: NativeCopperLayerContext,
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

pub(super) fn render_native_project_gerber_copper_projection(
    root: &Path,
    layer: i32,
) -> Result<NativeGerberCopperProjection> {
    let (project, model) = load_native_project_with_resolved_board_and_model(root)?;
    let context = resolve_native_project_copper_layer_context(&project, Some(&model), layer)?;
    let gerber = render_rs274x_copper_layer(
        layer,
        &context.pads,
        &context.tracks,
        &context.zones,
        &context.vias,
    )
    .context("failed to render native board copper layer as RS-274X")?;
    let production_projection = production_projection_view(
        "gerber_copper_layer",
        "datum.production_projection.gerber_copper_layer.v1",
        model.model_revision.0,
        gerber.as_bytes(),
    );
    Ok(NativeGerberCopperProjection {
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        layer,
        gerber,
        context,
        production_projection,
    })
}
