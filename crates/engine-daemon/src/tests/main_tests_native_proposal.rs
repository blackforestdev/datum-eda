//! Typed transport reaches the same persisted Proposal lifecycle as the engine.
use super::*;
use eda_engine::{
    api::native_write::{
        WriteProvenance, commit_prepared,
        genesis::{GenesisSpec, bootstrap_native_project},
        registry::{NativeWriteContext, find_native_write_verb},
        schematic_delete_proposal::{
            DeleteClass, DeleteTarget, NativeProposalAction, NativeProposalResponse,
        },
        schematic_sheets::build_create_schematic_sheet,
    },
    substrate::{CommitSource, ElectricalSelectionSnapshot, ProjectResolver, ProposalStatus},
};
use uuid::Uuid;
fn call(engine: &mut Engine, request: &NativeProposalRequest) -> NativeProposalResponse {
    let reply = dispatch_request(
        engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(1),
            method: "native.proposal".into(),
            params: serde_json::to_value(request).unwrap(),
        },
    );
    assert!(reply.error.is_none(), "{reply:?}");
    serde_json::from_value(reply.result.unwrap()).unwrap()
}
#[test]
fn native_batch_delete_transport_requires_complete_typed_guards_and_persists_review() {
    let root = std::env::temp_dir().join(format!("datum_native_proposal_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let ids = bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Proposal transport".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let provenance =
        || WriteProvenance::new("test", CommitSource::Test, "proposal transport fixture");
    let sheet = Uuid::new_v4();
    let value = json!({"schema_version":1,"uuid":sheet,"name":"Main","symbols":{},"wires":{},"labels":{},
        "ports":{},"junctions":{},"buses":{},"bus_entries":{},"noconnects":{},"texts":{},"drawings":{}});
    let write = build_create_schematic_sheet(
        &model,
        provenance(),
        ids.schematic_uuid,
        sheet,
        "sheets/main.json",
        value,
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    for (verb, params) in [
        (
            "datum.schematic.draw_wire",
            json!({"sheet":sheet,"points":[{"x":0,"y":0},{"x":100,"y":0}]}),
        ),
        (
            "datum.schematic.place_label",
            json!({"sheet":sheet,"name":"D0","kind":"local","x_nm":0,"y_nm":0}),
        ),
    ] {
        let write = (find_native_write_verb(verb).unwrap().build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            provenance(),
            params,
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }
    let before = ElectricalSelectionSnapshot::capture(&model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone();
    let wire = *before.sheets[&sheet].wires.keys().next().unwrap();
    let label = *before.sheets[&sheet].labels.keys().next().unwrap();
    let id = Uuid::new_v4();
    let mut request = NativeProposalRequest {
        project_root: root.clone(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: eda_engine::revision::transaction_tip(&model.journal),
        action: NativeProposalAction::CreateDelete {
            proposal_id: id,
            sheet,
            instance_path: vec![],
            targets: vec![
                DeleteTarget {
                    class: DeleteClass::Wire,
                    id: wire,
                },
                DeleteTarget {
                    class: DeleteClass::Label,
                    id: label,
                },
            ],
        },
    };
    let mut engine = Engine::new().unwrap();
    let mut missing = serde_json::to_value(&request).unwrap();
    missing
        .as_object_mut()
        .unwrap()
        .remove("expected_transaction_tip");
    assert_eq!(
        response(&mut engine, json!(0), missing).error.unwrap().code,
        -32602
    );
    let mut invalid = serde_json::to_value(&request).unwrap();
    invalid["action"]["targets"][0]["class"] = json!("bus_entry");
    assert_eq!(
        response(&mut engine, json!(0), invalid).error.unwrap().code,
        -32602
    );
    let draft = call(&mut engine, &request);
    assert_eq!(draft.status, ProposalStatus::Draft);
    request.expected_model_revision = draft.current_model_revision;
    request.expected_transaction_tip = draft.transaction_tip;
    request.action = NativeProposalAction::Preview { proposal_id: id };
    let preview = call(&mut engine, &request);
    assert_eq!(preview.status, ProposalStatus::Draft);
    assert_eq!(preview.transaction_tip, request.expected_transaction_tip);
    request.action = NativeProposalAction::Reject { proposal_id: id };
    let rejected = call(&mut engine, &request);
    assert_eq!(rejected.status, ProposalStatus::Rejected);
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        ElectricalSelectionSnapshot::capture(&reopened)
            .unwrap()
            .schematic_source()
            .unwrap(),
        &before
    );
    assert_eq!(reopened.proposals[&id].status, ProposalStatus::Rejected);
    std::fs::remove_dir_all(root).unwrap();
}
