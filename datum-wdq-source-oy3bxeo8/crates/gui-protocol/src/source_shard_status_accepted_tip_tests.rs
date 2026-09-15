use std::path::{Path, PathBuf};

use eda_engine::substrate::{CommitProvenance, CommitSource, Operation, OperationBatch};
use uuid::Uuid;

use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_json(path: &Path, value: serde_json::Value) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("fixture directory should create");
    }
    std::fs::write(path, format!("{value}\n")).expect("fixture JSON should write");
}

fn write_minimal_native_project(root: &Path) {
    write_json(
        &root.join("project.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v4(),
            "name": "GUI Accepted Transaction Tip Demo",
            "pools": [],
            "schematic": "schematic/schematic.json",
            "board": "board/board.json",
            "rules": "rules/rules.json",
            "forward_annotation_review": {}
        }),
    );
    write_json(
        &root.join("schematic/schematic.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v4(),
            "sheets": {},
            "definitions": {},
            "instances": [],
            "variants": {},
            "waivers": [],
            "deviations": []
        }),
    );
    write_json(
        &root.join("board/board.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v4(),
            "name": "GUI Accepted Transaction Tip Demo Board",
            "stackup": { "layers": [] },
            "outline": { "vertices": [], "closed": true },
            "packages": {},
            "component_silkscreen": {},
            "component_pads": {},
            "pads": {},
            "tracks": {},
            "vias": {},
            "zones": {},
            "nets": {},
            "net_classes": {},
            "dimensions": {},
            "texts": {},
            "keepouts": {}
        }),
    );
    write_json(
        &root.join("rules/rules.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v4(),
            "object_revision": 0,
            "rules": []
        }),
    );
}

#[test]
fn accepted_transaction_tip_reports_resolver_journal_tip() {
    let root = unique_project_root("datum-gui-accepted-transaction-tip");
    write_minimal_native_project(&root);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve before commit");
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "gui-protocol-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record review sidecar".to_string(),
                },
                operations: vec![Operation::SetForwardAnnotationReview {
                    relative_path: ".datum/forward_annotation_review/review.json".to_string(),
                    previous_review: None,
                    review: serde_json::json!({
                        "schema_version": 1,
                        "reviews": {}
                    }),
                }],
            },
        )
        .expect("review sidecar should commit");

    let tip = load_accepted_transaction_tip(&LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .expect("accepted transaction tip should load");

    assert_eq!(tip, Some(report.transaction.transaction_id.to_string()));
    let _ = std::fs::remove_dir_all(&root);
}
