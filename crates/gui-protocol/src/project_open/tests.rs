#[path = "../../../test-harness/src/development_workspace.rs"]
mod development_workspace;
use super::*;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/golden/text/native/text-transform-repro")
        .canonicalize()
        .unwrap()
}

#[test]
fn native_open_prepares_complete_model_without_selection_or_source_writes() {
    let root = fixture();
    let sources = [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        "rules/rules.json",
    ];
    let before: Vec<_> = sources
        .iter()
        .map(|path| std::fs::read(root.join(path)).unwrap())
        .collect();
    let prepared = PreparedProjectOpen::prepare(&root, false).unwrap();
    assert_eq!(prepared.root, root);
    assert!(matches!(
        prepared.workspace.selection,
        SelectionTarget::None
    ));
    assert!(prepared.workspace.schematic_scene.is_none());
    assert_eq!(prepared.workspace.scene.board_text_geometries.len(), 8);
    assert_eq!(prepared.workspace.scene.board_graphics.len(), 4);
    assert_eq!(
        prepared
            .workspace
            .backing
            .as_ref()
            .unwrap()
            .request
            .project_root,
        root
    );
    assert_eq!(
        prepared.workspace.scene.project_uuid,
        prepared.model.project.project_id.to_string()
    );
    prepared.revalidate().unwrap();
    for (path, bytes) in sources.iter().zip(before) {
        assert_eq!(std::fs::read(root.join(path)).unwrap(), bytes);
    }
}

#[test]
fn session_origin_is_root_and_generation_qualified_not_project_uuid_qualified() {
    let root = fixture();
    let origin = ProjectOpenOrigin::capture(&root, 7).unwrap();
    assert!(origin.matches(&root, 7));
    assert!(!origin.matches(&root, 8));
    // A different directory remains a different session even if its source
    // documents happen to carry the same Project UUID/revision.
    assert!(!origin.matches(root.parent().unwrap(), 7));
    assert!(!origin.matches(&root.join("absent"), 7));
}

#[test]
fn invalid_open_preserves_dialog_input_for_correction() {
    let mut dialog = ProjectOpenDialogState {
        open: true,
        directory: fixture().join("missing-project").display().to_string(),
        ..Default::default()
    };
    let entered = dialog.directory.clone();
    let refusal = PreparedProjectOpen::prepare(Path::new(&dialog.directory), false)
        .err()
        .unwrap();
    dialog.refusal = Some(refusal.to_string());
    assert_eq!(dialog.directory, entered);
    assert!(dialog.open_enabled());
    dialog.busy = true;
    assert!(!dialog.open_enabled());
    dialog.move_focus(false, false);
    assert_eq!(dialog.focus, ProjectOpenFocus::Cancel);
}

fn copied_fixture(label: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let root = development_workspace::path("open-project-protocol-tests").join(format!(
        "{}-{}-{}",
        std::process::id(),
        label,
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    for relative in [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        "rules/rules.json",
    ] {
        let destination = root.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::copy(fixture().join(relative), destination).unwrap();
    }
    root.canonicalize().unwrap()
}

#[test]
fn identical_copied_projects_are_distinct_origins_and_admission_holds_writer_exclusion() {
    let a = copied_fixture("origin-a");
    let b = copied_fixture("origin-b with space");
    let left = PreparedProjectOpen::prepare(&a, false).unwrap();
    let right = PreparedProjectOpen::prepare(&b, false).unwrap();
    assert_eq!(
        left.model.project.project_id,
        right.model.project.project_id
    );
    assert_eq!(left.model.model_revision, right.model.model_revision);
    let origin = ProjectOpenOrigin::capture(&a, 3).unwrap();
    assert!(!origin.matches(&b, 3));
    assert!(!origin.matches(&a, 4));
    let lease = right.admit().unwrap();
    assert!(eda_engine::revision::ProjectWriteLease::acquire(&b).is_err());
    drop(lease);
    assert!(eda_engine::revision::ProjectWriteLease::acquire(&b).is_ok());
}

#[test]
fn journal_bearing_destination_rejects_changed_source_without_history_file_changes() {
    use eda_engine::substrate::{CommitProvenance, CommitSource, Operation, OperationBatch};
    use serde_json::json;
    use uuid::Uuid;
    let root = copied_fixture("journal-source-change");
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let text_id = Uuid::new_v4();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "gui-protocol-test".into(),
                    source: CommitSource::Test,
                    reason: "qualify Open source admission on a journal-bearing native project"
                        .into(),
                },
                operations: vec![Operation::CreateBoardText {
                    text_id,
                    text: json!({"uuid": text_id,
            "text": "Admission", "position": {"x": 1_000_000, "y": 2_000_000},
            "rotation": 0, "layer": 37, "height_nm": 1_000_000}),
                }],
            },
        )
        .unwrap();
    let prepared = PreparedProjectOpen::prepare(&root, false).unwrap();
    assert!(!prepared.model.journal.is_empty());
    let journal_path = eda_engine::substrate::transaction_journal_path(&root);
    let journal_before = std::fs::read(&journal_path).unwrap();
    let cursor_path = root.join(".datum/journal/cursor.json");
    let cursor_before = std::fs::read(&cursor_path).ok();
    let before_hash = prepared
        .model
        .source_shards
        .iter()
        .find(|shard| shard.relative_path == "rules/rules.json")
        .unwrap()
        .content_hash
        .clone();
    let path = root.join("rules/rules.json");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.push(b'\n');
    // Deliberately change a source shard outside the cooperating writer. The
    // test proves conservative admission, not exclusion of arbitrary FS writes.
    std::fs::write(&path, bytes).unwrap();
    let changed = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(std::fs::read(journal_path).unwrap(), journal_before);
    assert_eq!(std::fs::read(cursor_path).ok(), cursor_before);
    let after_hash = &changed
        .source_shards
        .iter()
        .find(|shard| shard.relative_path == "rules/rules.json")
        .unwrap()
        .content_hash;
    assert_ne!(*after_hash, before_hash);
    assert!(changed.journal.is_empty());
    assert!(
        changed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "journal_after_revision_mismatch")
    );
    assert_ne!(changed, *prepared.model);
    assert!(prepared.revalidate().is_err());
    assert!(prepared.admit().is_err());
}

#[test]
fn open_accessibility_preserves_attempted_path_refusal_and_cancel_during_preparation() {
    let mut dialog = ProjectOpenDialogState {
        open: true,
        directory: "/owned/attempted-project".into(),
        ..Default::default()
    };
    dialog.focus = ProjectOpenFocus::Navigation(FolderBrowserFocus::Path);
    let mut navigation = FolderBrowserState::new(FolderBrowserMode::Open, dialog.directory.clone());
    navigation.directory = dialog.directory.clone();
    navigation.loading = false;
    let combined = |dialog: &ProjectOpenDialogState| {
        let mut nodes = dialog.accessibility_nodes();
        let focus = if let ProjectOpenFocus::Navigation(focus) = dialog.focus {
            Some(focus)
        } else {
            None
        };
        nodes.extend(navigation.accessibility_nodes(focus, None, !dialog.busy));
        nodes
    };
    let nodes = combined(&dialog);
    assert_eq!(
        nodes
            .iter()
            .filter(|n| n.role == crate::GlobalPreferencesAccessibleRole::Dialog)
            .count(),
        1
    );
    let path = nodes
        .iter()
        .find(|n| n.id == "folder-navigation-path")
        .unwrap();
    assert_eq!(path.value.as_deref(), Some("/owned/attempted-project"));
    assert!(path.focused && path.available);
    dialog.busy = true;
    let nodes = combined(&dialog);
    assert!(
        nodes
            .iter()
            .any(|n| n.id == "open-project-cancel" && n.available)
    );
    assert!(
        nodes
            .iter()
            .any(|n| n.id == "open-project-submit" && !n.available)
    );
    assert!(nodes.iter().any(|n| n.id == "open-project-busy"));
    dialog.busy = false;
    dialog.refusal = Some("Owned writer is busy; source unchanged".into());
    let nodes = combined(&dialog);
    assert_eq!(
        nodes
            .iter()
            .find(|n| n.id == "open-project-refusal")
            .unwrap()
            .value
            .as_deref(),
        dialog.refusal.as_deref()
    );
    assert_eq!(
        nodes
            .iter()
            .find(|n| n.id == "folder-navigation-path")
            .unwrap()
            .value
            .as_deref(),
        Some("/owned/attempted-project")
    );
    dialog.open = false;
    assert!(dialog.accessibility_nodes().is_empty());
}

#[test]
fn missing_or_corrupt_declared_rules_refuses_open_without_repair_or_publication() {
    let root = copied_fixture("missing-required-rules");
    let rules = std::fs::read(root.join("rules/rules.json")).unwrap();
    let retained: Vec<_> = [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
    ]
    .iter()
    .map(|path| (*path, std::fs::read(root.join(path)).unwrap()))
    .collect();
    std::fs::remove_file(root.join("rules/rules.json")).unwrap();
    for corrupt in [false, true] {
        if corrupt {
            std::fs::write(root.join("rules/rules.json"), b"{").unwrap();
        }
        let error = PreparedProjectOpen::prepare(&root, false)
            .err()
            .expect("incomplete declared root must refuse");
        assert!(format!("{error:#}").contains("missing_required_shard"));
        for (path, bytes) in &retained {
            assert_eq!(std::fs::read(root.join(path)).unwrap(), *bytes);
        }
        if corrupt {
            assert_eq!(std::fs::read(root.join("rules/rules.json")).unwrap(), b"{");
        } else {
            assert!(!root.join("rules/rules.json").exists());
        }
    }
    // Explicit fixture repair restores valid Open; the application never repairs it.
    std::fs::write(root.join("rules/rules.json"), rules).unwrap();
    PreparedProjectOpen::prepare(&root, false).unwrap();
}
