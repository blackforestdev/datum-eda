use std::collections::BTreeMap;

use super::*;
use eda_engine::api::native_write::WriteProvenance;
use eda_engine::api::native_write::forward_annotation::{
    ForwardAnnotationProposalAction, build_forward_annotation_accepted_proposal,
};
use eda_engine::board::PlacedPackage;
use eda_engine::substrate::{
    ProjectResolver, Proposal, apply_accepted_proposal,
    commit_proposal_metadata_journaled,
};

pub(crate) fn build_forward_annotation_proposal(
    root: &Path,
    actions: &[NativeProjectForwardAnnotationProposalActionView],
    reviews: &[NativeProjectForwardAnnotationReviewActionView],
) -> Result<Option<Proposal>> {
    let review_by_id = reviews
        .iter()
        .map(|review| (review.action_id.as_str(), review.decision.as_str()))
        .collect::<BTreeMap<_, _>>();
    let model = ProjectResolver::new(root).resolve()?;
    let mut proposal_actions = Vec::new();
    for action in actions {
        if matches!(
            review_by_id.get(action.action_id.as_str()),
            Some(&"rejected" | &"deferred")
        ) {
            continue;
        }
        if action.action == "update_component" && action.reason == "value_mismatch" {
            let package_id = Uuid::parse_str(
                action
                    .component_uuid
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing component UUID"))?,
            )
            .context("invalid component UUID in forward-annotation proposal")?;
            proposal_actions.push(ForwardAnnotationProposalAction::SetComponentValue {
                package_id,
                value: action
                    .schematic_value
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing schematic value"))?,
            });
        } else if action.action == "remove_component"
            && action.reason == "board_component_missing_in_schematic"
        {
            let package_id = component_uuid_for_action(action)?;
            let project = load_native_project_with_resolved_board(root)?;
            let key = package_id.to_string();
            let package = project.board.packages.get(&key).cloned().ok_or_else(|| {
                anyhow::anyhow!("board component not found in native project: {package_id}")
            })?;
            proposal_actions.push(ForwardAnnotationProposalAction::RemoveComponent {
                package_id,
                package,
                materialized: component_materialization_payload(&project, &key),
            });
        }
    }

    let action_ids = actions
        .iter()
        .map(|action| action.action_id.as_str())
        .collect::<Vec<_>>();
    Ok(build_forward_annotation_accepted_proposal(
        &model,
        WriteProvenance::new(
            "datum-eda-cli",
            cli_commit_source()?,
            "forward annotation accepted proposal",
        ),
        "forward annotation self-sufficient board updates".to_string(),
        &action_ids,
        proposal_actions,
    )?)
}

pub(crate) fn can_apply_with_embedded_proposal(
    actions: &[NativeProjectForwardAnnotationProposalActionView],
) -> bool {
    actions
        .iter()
        .all(|action| action.action == "update_component" && action.reason == "value_mismatch")
        || actions.iter().all(|action| {
            (action.action == "update_component" && action.reason == "value_mismatch")
                || (action.action == "remove_component"
                    && action.reason == "board_component_missing_in_schematic")
        })
}

pub(crate) fn apply_forward_annotation_proposal(
    root: &Path,
    proposal: Proposal,
    actions: &[NativeProjectForwardAnnotationProposalActionView],
) -> Result<Vec<NativeProjectForwardAnnotationApplyReportView>> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let proposal_id = proposal.proposal_id;
    let before = load_native_project_with_resolved_board(root)?;
    commit_proposal_metadata_journaled(&mut model, root, proposal)?;
    apply_accepted_proposal(&mut model, root, proposal_id)?;

    let after = load_native_project_with_resolved_board(root)?;
    actions
        .iter()
        .map(|action| {
            let report_project = if action.action == "remove_component" {
                &before
            } else {
                &after
            };
            let component = parse_component(report_project, component_uuid_for_action(action)?)?;
            Ok(NativeProjectForwardAnnotationApplyReportView {
                action: "apply_forward_annotation_action".to_string(),
                action_id: action.action_id.clone(),
                proposal_action: action.action.clone(),
                reason: action.reason.clone(),
                component_report: native_project_board_component_report(
                    if action.action == "remove_component" {
                        "delete_board_component"
                    } else {
                        "set_board_component_value"
                    },
                    report_project,
                    component,
                ),
            })
        })
        .collect()
}

fn component_uuid_for_action(
    action: &NativeProjectForwardAnnotationProposalActionView,
) -> Result<Uuid> {
    Uuid::parse_str(
        action
            .component_uuid
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("proposal action missing component UUID"))?,
    )
    .context("invalid component UUID in forward-annotation proposal")
}

fn parse_component(
    project: &super::LoadedNativeProject,
    component_uuid: Uuid,
) -> Result<PlacedPackage> {
    let value = project
        .board
        .packages
        .get(&component_uuid.to_string())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("board component not found in native project: {component_uuid}")
        })?;
    serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })
}

fn component_materialization_payload(
    project: &super::LoadedNativeProject,
    key: &str,
) -> serde_json::Value {
    let mut payload = serde_json::Map::new();
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen",
        &project.board.component_silkscreen,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen_texts",
        &project.board.component_silkscreen_texts,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen_arcs",
        &project.board.component_silkscreen_arcs,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen_circles",
        &project.board.component_silkscreen_circles,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen_polygons",
        &project.board.component_silkscreen_polygons,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_silkscreen_polylines",
        &project.board.component_silkscreen_polylines,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_lines",
        &project.board.component_mechanical_lines,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_texts",
        &project.board.component_mechanical_texts,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_polygons",
        &project.board.component_mechanical_polygons,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_polylines",
        &project.board.component_mechanical_polylines,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_circles",
        &project.board.component_mechanical_circles,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_mechanical_arcs",
        &project.board.component_mechanical_arcs,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_pads",
        &project.board.component_pads,
        key,
    );
    insert_component_materialization_map(
        &mut payload,
        "component_models_3d",
        &project.board.component_models_3d,
        key,
    );
    serde_json::Value::Object(payload)
}

fn insert_component_materialization_map<T: serde::Serialize>(
    payload: &mut serde_json::Map<String, serde_json::Value>,
    field: &str,
    map: &BTreeMap<String, Vec<T>>,
    key: &str,
) {
    if let Some(value) = map.get(key) {
        payload.insert(
            field.to_string(),
            serde_json::to_value(value)
                .expect("component materialization payload serialization must succeed"),
        );
    }
}
