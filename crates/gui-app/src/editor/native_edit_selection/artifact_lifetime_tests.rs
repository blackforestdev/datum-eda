#[path = "../../../../test-harness/src/development_workspace.rs"]
mod development_workspace;
use super::*;
use datum_gui_protocol::{LiveReviewRequest, SelectionSubject};
use std::{path::PathBuf, process::Command};

const FINGERPRINT: &str = "sha256:260611ced8897cecba4f669ef163ded751b6644af9e32efa7c0a74294e60ab29";

fn root() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = base.join("crates/test-harness/testdata/quality/route_strategy_curated_baseline_v1/same-outcome-default");
    let root = development_workspace::path("diagnostic-prepared-selection-tests")
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
    root.canonicalize().unwrap()
}

fn canonical(root: &std::path::Path, args: &[&str]) -> serde_json::Value {
    let cli = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/datum-eda");
    let output = Command::new(cli)
        .args(["--format", "json"])
        .args(args)
        .arg(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn workspace(
    root: &std::path::Path,
) -> (
    datum_gui_protocol::ReviewWorkspaceState,
    Arc<eda_engine::substrate::DesignModel>,
) {
    datum_gui_protocol::load_board_editor_workspace_with_model(&LiveReviewRequest {
        project_root: root.into(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .unwrap()
}

fn source_history(root: &std::path::Path) -> Vec<Option<Vec<u8>>> {
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

fn selected(
    state: &datum_gui_protocol::ReviewWorkspaceState,
    model: &Arc<eda_engine::substrate::DesignModel>,
    subject: SelectionSubject,
) -> SelectionTarget {
    let authority = NativeSelectionResolution::capture(model).unwrap();
    SelectionTarget::Native(Box::new(
        build_workspace_selection_view(model, &authority, subject, state).unwrap(),
    ))
}

fn prepare(
    state: &mut datum_gui_protocol::ReviewWorkspaceState,
    model: Arc<eda_engine::substrate::DesignModel>,
    prior: SelectionTarget,
    previous: Arc<NativeSelectionResolution>,
) -> SelectionTarget {
    let inputs = Inputs {
        selected: prior,
        filters: state.ui.filters.clone(),
        layout: state.ui.layout.clone(),
        navigation: state.schematic_documents.navigation(),
        prior: Some(previous),
    };
    prepare_from_model(state, inputs, model, Default::default())
        .unwrap()
        .selected
}

fn empty_report(target: &SelectionTarget, notice: &str) {
    let SelectionTarget::Native(view) = target else {
        panic!("Current empty artifact report must survive preparation")
    };
    assert_eq!(view.subject, SelectionSubject::None);
    assert_eq!(view.notice.as_deref(), Some(notice));
    assert!(view.members.is_empty() && view.related.is_empty() && view.dropped.is_empty());
    assert!(view.board_keys.is_empty() && view.schematic_keys.is_empty());
    assert!(view.board_presentations.is_empty() && view.schematic_presentations.is_empty());
    assert!(view.authored_anchors.is_empty());
    assert!(!view.stale);
    assert!(
        view.inspection
            .iter()
            .all(|group| !group.fields.contains_key("fingerprint"))
    );
}

#[test]
fn actual_check_producer_survives_equivalent_run_reports_disappearance_and_does_not_resurrect() {
    let root = root();
    canonical(&root, &["check", "run", "--profile", "native-combined"]);
    let (state, model) = workspace(&root);
    assert!(
        state
            .checks
            .findings
            .iter()
            .any(|f| f.fingerprint == FINGERPRINT)
    );
    let prior = selected(
        &state,
        &model,
        SelectionSubject::Diagnostic(FINGERPRINT.into()),
    );
    let initial_authority = Arc::new(NativeSelectionResolution::capture(&model).unwrap());
    canonical(&root, &["check", "run", "--profile", "native-combined"]);
    let (mut next, same_model) = workspace(&root);
    let before = source_history(&root);
    let survived = prepare(&mut next, same_model.clone(), prior, initial_authority);
    assert_eq!(source_history(&root), before);
    assert!(
        matches!(&survived, SelectionTarget::Native(v) if v.subject == SelectionSubject::Diagnostic(FINGERPRINT.into()) && v.notice.is_none())
    );

    let same_authority = Arc::new(NativeSelectionResolution::capture(&same_model).unwrap());
    let track = canonical(
        &root,
        &[
            "project",
            "draw-board-track",
            "--net",
            "00000000-0000-0000-0000-00000000c200",
            "--from-x-nm",
            "500000",
            "--from-y-nm",
            "600000",
            "--to-x-nm",
            "4500000",
            "--to-y-nm",
            "2400000",
            "--width-nm",
            "200000",
            "--layer",
            "1",
        ],
    );
    canonical(&root, &["check", "run", "--profile", "native-combined"]);
    let (mut next, changed) = workspace(&root);
    assert!(
        !next
            .checks
            .findings
            .iter()
            .any(|f| f.fingerprint == FINGERPRINT)
    );
    let before = source_history(&root);
    let report = prepare(&mut next, changed.clone(), survived, same_authority);
    assert_eq!(source_history(&root), before);
    empty_report(
        &report,
        "Producing artifact invalidated its selected identity; selection cleared",
    );
    let changed_authority = Arc::new(NativeSelectionResolution::capture(&changed).unwrap());
    canonical(
        &root,
        &[
            "project",
            "delete-board-track",
            "--track",
            track["track_uuid"].as_str().unwrap(),
        ],
    );
    canonical(&root, &["check", "run", "--profile", "native-combined"]);
    let (mut next, restored) = workspace(&root);
    assert!(
        next.checks
            .findings
            .iter()
            .any(|f| f.fingerprint == FINGERPRINT)
    );
    let before = source_history(&root);
    assert!(matches!(
        prepare(&mut next, restored, report, changed_authority),
        SelectionTarget::None
    ));
    assert_eq!(source_history(&root), before);
}

#[test]
fn prepared_review_producer_change_reports_empty_without_authored_drop() {
    let root = root();
    let (_, model) = workspace(&root);
    // The existing route-review fixture supplies real whole action IDs. This
    // isolated consumer control binds it to the resolved native model.
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    state.supervision.journal.accepted_transaction_tip =
        eda_engine::revision::transaction_tip(&model.journal).map(|id| id.to_string());
    state.review.project_uuid = Some(model.project.project_id.to_string());
    state.review.producer_revision = Some(model.model_revision.0.clone());
    let action = state
        .review
        .proposal_actions
        .first()
        .expect("Existing review fixture has an action")
        .action_id
        .clone();
    let prior = selected(&state, &model, SelectionSubject::Review(action));
    state.review.selected_policy = Some("different-producer-policy".into());
    let before = source_history(&root);
    let authority = Arc::new(NativeSelectionResolution::capture(&model).unwrap());
    let report = prepare(&mut state, model.clone(), prior, authority);
    assert_eq!(source_history(&root), before);
    empty_report(
        &report,
        "Selected action producer changed; selection cleared",
    );
}
