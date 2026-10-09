//! Required P0 typed transport rehearsal over the same owned writer companion.
use super::*;

#[test]
#[ignore = "requires owned DATUM_FIRST_EDIT_PROJECT; mutates and restores its name"]
fn first_edit_p0_transport_and_stale_refusal() {
    let root = PathBuf::from(std::env::var_os("DATUM_FIRST_EDIT_PROJECT").unwrap());
    let client = NativeProjectClient::spawn().unwrap();
    let initial = client.describe(&root).unwrap();
    let preimage = std::fs::read(root.join("project.json")).unwrap();
    let id = Uuid::new_v4();
    let result=client.call("native.write",json!({
        "project_root":root,"verb":"datum.project.set_name","params":{"name":"First edit transport rehearsal"},
        "actor":"datum-gui","source":"tool","reason":format!("P0 typed transport {id}"),
        "expected_model_revision":initial["model_revision"],"batch_id":id
    }),id).unwrap();
    assert_eq!(result["status"], "applied");
    let next = client.describe(&root).unwrap();
    assert_eq!(next["project_name"], "First edit transport rehearsal");
    let stale = NativeEditIntent::new(
        root.clone(),
        serde_json::from_value(initial["project_id"].clone()).unwrap(),
        initial["model_revision"].as_str().unwrap().into(),
        serde_json::from_value(initial["transaction_tip"].clone()).unwrap(),
        NativeEditAction::Save,
    );
    assert!(
        client.execute(&stale).is_err(),
        "stale prepared intent cannot acknowledge current Project"
    );
    let tip = serde_json::from_value(next["transaction_tip"].clone()).unwrap();
    let undo = NativeEditIntent::new(
        root.clone(),
        stale.project_id,
        next["model_revision"].as_str().unwrap().into(),
        Some(tip),
        NativeEditAction::Undo { expected_tip: tip },
    );
    assert!(client.execute(&undo).unwrap().applied);
    assert_eq!(
        std::fs::read(root.join("project.json")).unwrap(),
        preimage,
        "P0 undo byte-exact"
    );
    eprintln!(
        "P0 typed native rename; independent journal describe; stale intent refused; canonical undo byte-exact"
    );
}

#[test]
fn complete_model_binding_refuses_stale_mismatched_and_wrong_root_snapshots() {
    use eda_engine::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};
    let root = std::env::temp_dir().join(format!("datum-snapshot-binding-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Snapshot".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let description = json!({"project_id":model.project.project_id,"model_revision":model.model_revision.0,"transaction_tip":eda_engine::revision::transaction_tip(&model.journal),"journal_len":model.journal.len()});
    validate_read_model(&root, &description, &description, &model).unwrap();
    let mut stale = model.clone();
    stale.model_revision.0 = "stale".into();
    assert!(validate_read_model(&root, &description, &description, &stale).is_err());
    let mut other = model.clone();
    other.project.project_id = Uuid::new_v4();
    assert!(validate_read_model(&root, &description, &description, &other).is_err());
    let mut changed = description.clone();
    changed["transaction_tip"] = json!(Uuid::new_v4());
    assert!(validate_read_model(&root, &description, &changed, &model).is_err());
    let foreign = root.join("foreign");
    std::fs::create_dir(&foreign).unwrap();
    std::fs::copy(root.join("project.json"), foreign.join("project.json")).unwrap();
    assert!(validate_read_model(&foreign, &description, &description, &model).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "requires owned DATUM_FIRST_EDIT_PROJECT; commits one real-footprint move"]
fn first_edit_committed_model_matches_fresh_projection() {
    use eda_engine::api::native_write::{
        WriteProvenance,
        registry::{NativeWriteContext, find_native_write_verb},
    };
    let root = PathBuf::from(std::env::var_os("DATUM_FIRST_EDIT_PROJECT").unwrap());
    let mut engine = eda_engine::api::Engine::new().unwrap();
    let mut model = std::sync::Arc::unwrap_or_clone(engine.read_native_project(&root).unwrap());
    let component = "ca340e78-04a5-5307-a024-9c5720a807d2";
    let board: Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    let pos = &board["packages"][component]["position"];
    let prepared = (find_native_write_verb("datum.pcb.move_component")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        WriteProvenance::new(
            "native-fixture",
            eda_engine::substrate::CommitSource::Test,
            "committed model/fresh projection correspondence",
        ),
        json!({"component":component,"x_nm":pos["x"].as_i64().unwrap()+250_000,"y_nm":pos["y"]}),
    )
    .unwrap();
    engine
        .commit_native_prepared(&mut model, &root, prepared)
        .unwrap();
    let fresh = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let a = serde_json::to_value(&model).unwrap();
    let b = serde_json::to_value(&fresh).unwrap();
    let changed: Vec<_> = a
        .as_object()
        .unwrap()
        .iter()
        .filter(|(k, v)| b.get(k.as_str()) != Some(v))
        .map(|(k, _)| k.clone())
        .collect();
    eprintln!("committed/fresh model differing fields={changed:?}");
    assert!(
        changed.is_empty(),
        "Committed result cannot replace fresh projection: {changed:?}"
    );
}
