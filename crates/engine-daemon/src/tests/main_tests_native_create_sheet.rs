//! Canonical first-sheet creation, snapshot guards and journal history.
use super::*;

#[test]
fn native_create_sheet_requires_guards_and_survives_undo_redo_reopen() {
    let mut engine = Engine::new().unwrap();
    let root = temp_native_project("create_first_sheet");
    let before = describe(&mut engine, 1, &root);
    assert_eq!(before["journal_len"], 0);
    assert!(before["transaction_tip"].is_null());
    let sheet = uuid::Uuid::new_v4();
    let path = root.join(format!("schematic/sheets/{sheet}.json"));
    let base = json!({
        "project_root":root,"verb":"datum.schematic.create_sheet",
        "params":{"sheet":sheet,"name":"  Main  "},
        "actor":"datum-gui","source":"manual","reason":"Create first sheet"
    });
    for (revision, tip) in [(false, false), (true, false), (false, true)] {
        let mut params = base.clone();
        if revision {
            params["expected_model_revision"] = before["model_revision"].clone();
        }
        if tip {
            params["expected_transaction_tip"] = Value::Null;
        }
        let response = dispatch_request(&mut engine, native_request(2, "native.write", params));
        assert!(
            response.error.is_some(),
            "both explicit guards required: {response:?}"
        );
        assert_eq!(describe(&mut engine, 3, &root)["journal_len"], 0);
        assert!(!path.exists());
    }
    let mut guarded = base;
    guarded["expected_model_revision"] = before["model_revision"].clone();
    guarded["expected_transaction_tip"] = Value::Null;
    let created = dispatch_request(
        &mut engine,
        native_request(4, "native.write", guarded.clone()),
    );
    assert!(created.error.is_none(), "{created:?}");
    assert_eq!(
        created.result.unwrap()["primary_object_id"],
        sheet.to_string()
    );
    let bytes = std::fs::read(&path).unwrap();
    let created_description = describe(&mut engine, 5, &root);
    assert_eq!(created_description["journal_len"], 1);
    // Test the revision and history guards independently with a different identity.
    let other = uuid::Uuid::new_v4();
    guarded["params"]["sheet"] = json!(other);
    for revision in [
        before["model_revision"].clone(),
        created_description["model_revision"].clone(),
    ] {
        guarded["expected_model_revision"] = revision;
        assert!(
            dispatch_request(
                &mut engine,
                native_request(6, "native.write", guarded.clone())
            )
            .error
            .is_some()
        );
        assert!(!root.join(format!("schematic/sheets/{other}.json")).exists());
        assert_eq!(describe(&mut engine, 7, &root)["journal_len"], 1);
    }
    for (verb, exists) in [("datum.journal.undo", false), ("datum.journal.redo", true)] {
        let current = describe(&mut engine, 8, &root);
        let response = dispatch_request(
            &mut engine,
            native_request(
                9,
                "native.write",
                json!({
                    "project_root":root,"verb":verb,"params":{"expected_tip":current["transaction_tip"]},
                    "actor":"datum-gui","source":"manual","reason":"Sheet history proof",
                    "expected_model_revision":current["model_revision"],
                    "expected_transaction_tip":current["transaction_tip"]
                }),
            ),
        );
        assert!(response.error.is_none(), "{response:?}");
        assert_eq!(path.exists(), exists);
        let reopened = eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap();
        let source =
            eda_engine::substrate::ElectricalSelectionSnapshot::capture(&reopened).unwrap();
        assert_eq!(
            source
                .schematic_source()
                .unwrap()
                .sheets
                .contains_key(&sheet),
            exists
        );
    }
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(describe(&mut engine, 10, &root)["journal_len"], 3);
    std::fs::remove_dir_all(root).unwrap();
}
