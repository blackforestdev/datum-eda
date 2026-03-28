use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::board::PlacedPackage;
use uuid::Uuid;

use super::{
    NativeProjectBoardComponentMutationReportView, native_project_board_component_report,
    write_canonical_json,
};

pub(crate) fn set_native_project_board_component_reference(
    root: &Path,
    component_uuid: Uuid,
    reference: String,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = super::load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.reference = reference;
    project.board.packages.insert(
        key,
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "set_board_component_reference",
        &project,
        component,
    ))
}
