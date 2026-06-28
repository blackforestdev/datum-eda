use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::board::PlacedPackage;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};
use uuid::Uuid;

use super::{
    NativeProjectBoardComponentMutationReportView,
    command_project_operation_guards::guarded_object_operations,
    load_native_project_with_resolved_board, native_project_board_component_report,
};

pub(crate) fn set_native_project_board_component_reference(
    root: &Path,
    component_uuid: Uuid,
    reference: String,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let expected_model_revision = model.model_revision.clone();
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(expected_model_revision),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: "set board component reference".to_string(),
            },
            operations: guarded_object_operations(
                &model,
                component_uuid,
                vec![Operation::SetBoardPackageReference {
                    package_id: component_uuid,
                    reference,
                }],
            )?,
        },
    )?;

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
        "set_board_component_reference",
        &project,
        component,
    ))
}
