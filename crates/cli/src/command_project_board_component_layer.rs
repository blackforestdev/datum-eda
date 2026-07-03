use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::api::native_write::board_components::{
    BoardPackageEdit, build_edit_board_package,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::board::PlacedPackage;
use eda_engine::substrate::{CommitSource, ProjectResolver};
use uuid::Uuid;

use super::{
    NativeProjectBoardComponentMutationReportView, load_native_project_with_resolved_board,
    native_project_board_component_report,
};

pub(crate) fn set_native_project_board_component_layer(
    root: &Path,
    component_uuid: Uuid,
    layer: i32,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let prepared = build_edit_board_package(
        &model,
        WriteProvenance::new(
            "datum-eda-cli",
            CommitSource::Cli,
            "set board component layer",
        ),
        component_uuid,
        BoardPackageEdit::Side { layer },
    )?;
    commit_prepared(&mut model, root, prepared)?;

    let project = load_native_project_with_resolved_board(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    Ok(native_project_board_component_report(
        "set_board_component_layer",
        &project,
        component,
    ))
}
