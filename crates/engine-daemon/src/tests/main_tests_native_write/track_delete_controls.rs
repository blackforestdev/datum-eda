//! Real daemon dispatch proves a tip guard adds protection beyond model revision.
use super::*;
use eda_engine::{
    api::native_write::{self as native, WriteProvenance, commit_prepared},
    board::{Net, NetClass, Track},
    ir::geometry::Point,
    substrate::{
        CommitSource, ProjectResolver, ProposalCreateRequest, ProposalSource,
        create_draft_proposal_from_batch,
    },
};
use std::collections::BTreeMap;
use uuid::Uuid;

fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(path: &Path, output: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            assert!(!entry.file_type().unwrap().is_symlink());
            let path = entry.path();
            if path.is_dir() {
                visit(&path, output);
            } else {
                output.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut output = BTreeMap::new();
    visit(root, &mut output);
    output
}

#[test]
fn native_track_delete_requires_revision_and_explicit_current_tip_without_writes() {
    let root = temp_native_project("track_delete_guards");
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let proof = || {
        WriteProvenance::new(
            "track-delete-test",
            CommitSource::Manual,
            "Prepare transport fixture",
        )
    };
    let class = NetClass {
        uuid: Uuid::new_v4(),
        name: "Signal".into(),
        clearance: 100_000,
        track_width: 200_000,
        via_drill: 300_000,
        via_diameter: 600_000,
        diffpair_width: 200_000,
        diffpair_gap: 200_000,
    };
    let prepared =
        native::board_routing::build_place_board_net_class(&model, proof(), &class).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let net = Net::new(Uuid::new_v4(), "Signal", class.uuid);
    let prepared = native::board_routing::build_place_board_net(&model, proof(), &net).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let track = Track::straight(
        Uuid::new_v4(),
        net.uuid,
        Point::new(0, 0),
        Point::new(2_000_000, 0),
        200_000,
        1,
    );
    let prepared = native::board_routing::build_place_board_track(&model, proof(), &track).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut engine = Engine::new().unwrap();
    let before = describe(&mut engine, 1, &root);
    let base = json!({"project_root":root,"verb":"datum.pcb.delete_track","params":{"track":track.uuid},
        "source":"manual","reason":"Delete obsolete copper","expected_model_revision":before["model_revision"],
        "expected_transaction_tip":before["transaction_tip"]});
    let original = files(&root);
    for (field, value, code, message) in [
        (
            "expected_model_revision",
            None,
            -32602,
            "requires expected_model_revision",
        ),
        (
            "expected_transaction_tip",
            None,
            -32602,
            "requires explicit expected_transaction_tip",
        ),
        (
            "expected_transaction_tip",
            Some(Value::Null),
            -32061,
            "stale expected transaction tip",
        ),
        (
            "expected_transaction_tip",
            Some(json!(Uuid::new_v4())),
            -32061,
            "stale expected transaction tip",
        ),
    ] {
        let mut params = base.clone();
        if let Some(value) = value {
            params[field] = value;
        } else {
            params.as_object_mut().unwrap().remove(field);
        }
        let response = dispatch_request(&mut engine, native_request(2, "native.write", params));
        let error = response
            .error
            .as_ref()
            .expect("guard must refuse before mutation");
        assert_eq!(error.code, code, "{response:?}");
        assert!(error.message.contains(message), "{response:?}");
        assert_eq!(files(&root), original);
    }
    // A real proposal metadata transaction changes the tip while preserving the
    // model revision. This catches enforcement accidentally reduced to revision.
    let prepared =
        native::project::build_set_project_name(&model, proof(), "Unapplied proposal").unwrap();
    create_draft_proposal_from_batch(
        &mut model,
        &root,
        ProposalCreateRequest {
            proposal_id: Some(Uuid::new_v4()),
            batch: prepared.batch,
            rationale: "Tip guard control".into(),
            source: ProposalSource::Manual,
            checks_run: vec![],
            finding_fingerprints: vec![],
        },
    )
    .unwrap();
    let current = describe(&mut engine, 3, &root);
    assert_eq!(current["model_revision"], before["model_revision"]);
    assert_ne!(current["transaction_tip"], before["transaction_tip"]);
    let original = files(&root);
    let response = dispatch_request(&mut engine, native_request(4, "native.write", base.clone()));
    assert_eq!(response.error.as_ref().unwrap().code, -32061);
    assert!(
        response
            .error
            .unwrap()
            .message
            .contains("stale expected transaction tip")
    );
    assert_eq!(files(&root), original);
    let mut valid = base;
    valid["expected_transaction_tip"] = current["transaction_tip"].clone();
    let response = dispatch_request(&mut engine, native_request(5, "native.write", valid));
    assert!(response.error.is_none(), "{response:?}");
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    let board = cold
        .materialized_source_shard_value(eda_engine::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    assert!(board["tracks"].as_object().unwrap().is_empty());
    assert_eq!(
        cold.journal.len() as u64,
        current["journal_len"].as_u64().unwrap() + 1
    );
    std::fs::remove_dir_all(root).unwrap();
}
