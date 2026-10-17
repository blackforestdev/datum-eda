//! Package creation takes the same guarded daemon transport as the ordinary App.
use super::*;

#[test]
fn native_package_creation_guards_refuse_without_partial_writes() {
    let mut engine = Engine::new().unwrap();
    let root = temp_native_project("package_creation_transport");
    let before = describe(&mut engine, 1, &root);
    let package = uuid::Uuid::new_v4();
    let base = json!({"project_root":root,"verb":"datum.library.create","params":{"kind":"packages","object_id":package,"pool_path":"pool","fields":{"name":"GUI physical body","body_dimensions":{"x_nm":5000000,"y_nm":3000000}}},"actor":"datum-gui","source":"manual","reason":"Create Package","expected_model_revision":before["model_revision"],"expected_transaction_tip":before["transaction_tip"]});
    let path = root.join(format!("pool/packages/{package}.json"));
    let terminal = uuid::Uuid::new_v4();
    for variant in 0..9 {
        let mut request = base.clone();
        match variant {
            0 => {
                request
                    .as_object_mut()
                    .unwrap()
                    .remove("expected_model_revision");
            }
            1 => {
                request
                    .as_object_mut()
                    .unwrap()
                    .remove("expected_transaction_tip");
            }
            2 => request["params"]["pool_path"] = json!("foreign-pool"),
            3 => request["source"] = json!("tool"),
            4 => request["source"] = json!("assistant"),
            5 => request["params"]["fields"]["body_dimensions"] = json!({"x_nm":"invalid"}),
            6 => request["verb"] = json!("datum.library.edit"),
            7 => {
                request["params"]["fields"]["terminals"] = json!({
                    terminal.to_string():{"uuid":terminal,"name":"valid"},
                    terminal.simple().to_string():{"uuid":package,"name":"mismatching alias"}
                })
            }
            _ => {
                request["params"]["fields"]["terminals"] = json!({
                    terminal.to_string():{"uuid":terminal,"name":"first"},
                    terminal.simple().to_string():{"uuid":terminal,"name":"second"}
                })
            }
        }
        assert!(
            dispatch_request(&mut engine, native_request(2, "native.write", request))
                .error
                .is_some(),
            "variant {variant}"
        );
        assert!(!path.exists());
        assert_eq!(describe(&mut engine, 3, &root)["journal_len"], 0);
    }
    let response = dispatch_request(&mut engine, native_request(4, "native.write", base.clone()));
    assert!(response.error.is_none(), "{response:?}");
    let source = std::fs::read(&path).unwrap();
    let journal_path = eda_engine::substrate::transaction_journal_path(&root);
    let journal = std::fs::read(&journal_path).unwrap();
    assert_eq!(describe(&mut engine, 5, &root)["journal_len"], 1);
    let mut stale = base;
    stale["params"]["object_id"] = json!(uuid::Uuid::new_v4());
    let blocked_path = root.join(format!(
        "pool/packages/{}.json",
        stale["params"]["object_id"].as_str().unwrap()
    ));
    assert!(
        dispatch_request(&mut engine, native_request(6, "native.write", stale))
            .error
            .is_some()
    );
    assert!(!blocked_path.exists());
    assert_eq!(std::fs::read(path).unwrap(), source);
    assert_eq!(std::fs::read(journal_path).unwrap(), journal);
}
