#[path = "../../../../test-harness/src/development_workspace.rs"]
mod development_workspace;
use super::*;
use datum_gui_protocol::selection_resolution::SelectionAuthority;
use datum_gui_protocol::{
    AuthoredSelectionClass, AuthoredSelectionIdentity, LiveReviewRequest, SelectionSubject,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn root() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = base.join("crates/test-harness/testdata/quality/route_strategy_curated_baseline_v1/same-outcome-default");
    let root = development_workspace::path("run-prepared-selection-tests")
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

fn canonical(root: &Path, args: &[&str]) -> serde_json::Value {
    let output = Command::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/datum-eda"),
    )
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
    root: &Path,
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

fn history(root: &Path) -> Vec<Option<Vec<u8>>> {
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
fn actual_run_origin_deletion_reports_empty_and_undo_does_not_resurrect() {
    let root = root();
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
    let id = track["track_uuid"].as_str().unwrap();
    let origin = AuthoredSelectionIdentity {
        class: AuthoredSelectionClass::Track,
        id: uuid::Uuid::parse_str(id).unwrap(),
        instance_path: vec![],
    };
    let (state, model) = workspace(&root);
    let previous = Arc::new(NativeSelectionResolution::capture(&model).unwrap());
    let subject = previous
        .acquire_run(
            origin.clone(),
            Some(eda_engine::ir::geometry::Point::new(2500000, 1500000)),
        )
        .unwrap();
    assert!(matches!(&subject, SelectionSubject::Run(run) if run.origin == origin));
    let selected = SelectionTarget::Native(Box::new(
        build_workspace_selection_view(&model, &previous, subject, &state).unwrap(),
    ));
    let SelectionTarget::Native(view) = &selected else {
        unreachable!()
    };
    assert!(view.members.contains(&origin));
    canonical(&root, &["project", "delete-board-track", "--track", id]);
    let (mut next, model) = workspace(&root);
    let current = Arc::new(NativeSelectionResolution::capture(&model).unwrap());
    assert!(!current.authored_contains(&origin));
    let before = history(&root);
    let filters = next.ui.filters.clone();
    let layout = next.ui.layout.clone();
    let navigation = next.schematic_documents.navigation();
    let cleared = prepare_from_model(
        &mut next,
        Inputs {
            selected,
            filters,
            layout,
            navigation,
            prior: Some(previous),
        },
        model,
        Default::default(),
    )
    .unwrap()
    .selected;
    assert_eq!(before, history(&root));
    let SelectionTarget::Native(view) = &cleared else {
        panic!("Run origin dissolution must retain a current empty report")
    };
    assert_eq!(view.subject, SelectionSubject::None);
    assert_eq!(
        view.notice.as_deref(),
        Some("Selected Run origin no longer resolves; selection cleared.")
    );
    assert!(
        view.members.is_empty()
            && view.related.is_empty()
            && view.dropped.is_empty()
            && view.authored_anchors.is_empty()
    );
    assert!(
        view.board_keys.is_empty()
            && view.schematic_keys.is_empty()
            && view.board_presentations.is_empty()
            && view.schematic_presentations.is_empty()
    );
    assert!(!view.stale);
    canonical(&root, &["project", "undo"]);
    let (mut restored, model) = workspace(&root);
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    assert!(authority.authored_contains(&origin));
    let before = history(&root);
    let filters = restored.ui.filters.clone();
    let layout = restored.ui.layout.clone();
    let navigation = restored.schematic_documents.navigation();
    let selection = prepare_from_model(
        &mut restored,
        Inputs {
            selected: cleared,
            filters,
            layout,
            navigation,
            prior: Some(current),
        },
        model,
        Default::default(),
    )
    .unwrap()
    .selected;
    assert!(matches!(selection, SelectionTarget::None));
    assert_eq!(before, history(&root));
}
