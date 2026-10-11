use super::*;
use crate::{
    SelectionSubject, selection_artifacts::build_workspace_selection_view,
    selection_resolution::NativeSelectionResolution,
};
use std::process::Command;

fn command(args: &[&str]) -> serde_json::Value {
    let cli = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/datum-eda");
    let output = Command::new(cli)
        .args(["--format", "json"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn history(root: &std::path::Path) -> Vec<Option<Vec<u8>>> {
    [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        "rules/rules.json",
        ".datum/journal/transactions.jsonl",
        ".datum/journal/cursor.json",
    ]
    .iter()
    .map(|rel| std::fs::read(root.join(rel)).ok())
    .collect()
}
#[test]
fn explicit_artifact_refresh_uses_fresh_model_and_canonical_producer_status() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = base.join("crates/test-harness/testdata/quality/route_strategy_curated_baseline_v1/same-outcome-default");
    let root = base
        .join("target/proposal-refresh-tests")
        .join(uuid::Uuid::new_v4().to_string());
    for rel in [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        "rules/rules.json",
    ] {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(source.join(rel), path).unwrap();
    }
    let artifact = root
        .parent()
        .unwrap()
        .join(format!("{}.json", uuid::Uuid::new_v4()));
    command(&[
        "project",
        "export-route-proposal",
        root.to_str().unwrap(),
        "--net",
        "00000000-0000-0000-0000-00000000c200",
        "--from-anchor",
        "00000000-0000-0000-0000-00000000c205",
        "--to-anchor",
        "00000000-0000-0000-0000-00000000c206",
        "--out",
        artifact.to_str().unwrap(),
    ]);
    let request = LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: Some(artifact.clone()),
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let bytes = std::fs::read(&artifact).unwrap();
    let before = history(&root);
    let (initial, model) = load_board_editor_workspace_with_model(&request).unwrap();
    let action = initial.review.proposal_actions[0].action_id.clone();
    let projected = project_board_editor_workspace(&request, model.clone()).unwrap();
    assert_eq!(
        initial.review.proposal_actions,
        projected.review.proposal_actions
    );
    assert_eq!(
        initial.review.producer_proposal,
        projected.review.producer_proposal
    );
    let native = NativeSelectionResolution::capture(&model).unwrap();
    let proposal = build_workspace_selection_view(
        &model,
        &native,
        SelectionSubject::Proposal(action.clone()),
        &projected,
    )
    .unwrap();
    let review = build_workspace_selection_view(
        &model,
        &native,
        SelectionSubject::Review(action.clone()),
        &projected,
    )
    .unwrap();
    assert_ne!(proposal.subject, review.subject);
    assert!(proposal.members.is_empty() && review.members.is_empty());
    assert_eq!(history(&root), before);
    assert_eq!(std::fs::read(&artifact).unwrap(), bytes);
    // Bare and incomplete native editor requests retain their existing empty
    // review projection, rather than running a route query without inputs.
    let mut bare = request.clone();
    bare.artifact_path = None;
    assert!(!has_explicit_review(&bare));
    bare.net_uuid = Some("00000000-0000-0000-0000-00000000c200".into());
    assert!(!has_explicit_review(&bare));
    assert!(
        project_board_editor_workspace(&bare, model)
            .unwrap()
            .review
            .proposal_actions
            .is_empty()
    );
    command(&[
        "project",
        "apply-route-proposal-artifact",
        root.to_str().unwrap(),
        "--artifact",
        artifact.to_str().unwrap(),
    ]);
    let applied_model = Arc::new(ProjectResolver::new(&root).resolve().unwrap());
    let applied = project_board_editor_workspace(&request, applied_model.clone()).unwrap();
    let native = NativeSelectionResolution::capture(&applied_model).unwrap();
    for subject in [
        SelectionSubject::Proposal(action.clone()),
        SelectionSubject::Review(action.clone()),
    ] {
        let authority = crate::selection_artifacts::WorkspaceSelectionAuthority {
            native: &native,
            model: &applied_model,
            workspace: &applied,
        };
        assert!(
            authority.artifact(&subject).unwrap().is_none(),
            "canonical Applied must invalidate both subjects"
        );
    }
    command(&["project", "undo", root.to_str().unwrap()]);
    let restored_model = Arc::new(ProjectResolver::new(&root).resolve().unwrap());
    let restored = project_board_editor_workspace(&request, restored_model.clone()).unwrap();
    let native = NativeSelectionResolution::capture(&restored_model).unwrap();
    let producer = restored.review.producer_proposal.as_ref().unwrap();
    assert_eq!(
        restored_model.proposals[&producer.proposal_id].status,
        eda_engine::substrate::ProposalStatus::Accepted
    );
    // Restored authority permits explicit acquisition. Selection restoration
    // itself is owned by the application reconciliation and native workflow.
    assert!(
        build_workspace_selection_view(
            &restored_model,
            &native,
            SelectionSubject::Review(action),
            &restored
        )
        .is_ok()
    );
    assert_eq!(std::fs::read(&artifact).unwrap(), bytes);
}
