//! Canonical Unit pin metadata preserves identities and raw alternate rows.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};
use std::path::Path;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Edit Unit pin metadata")
}

fn edit(
    model: &DesignModel,
    root: &Path,
    unit: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    super::super::build(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        LibraryDefinitionRequest {
            kind: "units".into(),
            object_id: unit,
            pool_path: "pool".into(),
            fields: fields.as_object().unwrap().clone(),
        },
        false,
    )
}

#[test]
fn saved_unit_pin_metadata_exact_preimages_compose_and_history() {
    let root = crate::api::native_write::test_support::temp_project_root("pin_metadata");
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Pin metadata".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, sibling) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let key = pin.simple().to_string().to_uppercase();
    let row = json!({"name":" SAME ","kind":"alias","vendor":{"value":[1,2]}});
    let mut raw = library::pool_unit_payload(unit, "Unit", "Maker");
    raw["pins"] = json!({
        key.clone():{"uuid":pin,"name":"In","direction":"Input","swap_group":7,"alternates":[row,row],"vendor":true},
        sibling.to_string():{"uuid":sibling,"name":"Out","direction":"Output","swap_group":0,"alternates":[]}
    });
    let path = format!("pool/units/{unit}.json");
    let prepared = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "units", unit),
            object: raw,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let read =
        |model: &DesignModel| library::pool_library_object_payload(model, &path, unit).unwrap();
    let before = read(&model);
    let pin_before = &before["pins"][&key];
    let mut pins = before["pins"].clone();
    pins[&key]["name"] = json!("Renamed");
    let addition = json!({"name":"","kind":" Ω "});
    let good = json!({"previous":pin_before,"swap_group":u32::MAX,
        "alternate_edits":[{"index":1,"previous":row,"name":" revised ","kind":""}],
        "alternate_removals":[{"index":0,"previous":row}],"alternate_additions":[addition]});
    for change in [
        json!({"previous":{},"swap_group":2}),
        json!({"previous":pin_before,"swap_group":-1}),
        json!({"previous":pin_before,"alternate_removals":[{"index":0,"previous":{} }]}),
        json!({"previous":pin_before,"alternate_removals":[{"index":0,"previous":row},{"index":0,"previous":row}]}),
    ] {
        assert!(
            edit(
                &model,
                &root,
                unit,
                json!({"pin_metadata":{key.clone():change}})
            )
            .is_err()
        );
        assert_eq!(read(&model), before);
    }
    let context = NativeWriteContext {
        model: &model,
        project_root: &root,
    };
    let mut malformed = before.clone();
    malformed["pins"][&key]["alternates"][0] = json!("malformed");
    let mut candidate = malformed.clone();
    let request = LibraryDefinitionRequest {
        kind: "units".into(),
        object_id: unit,
        pool_path: "pool".into(),
        fields: json!({"pin_metadata":{key.clone():{"previous":malformed["pins"][&key],
            "alternate_edits":[{"index":0,"previous":"malformed","name":"Edit","kind":"alias"}]}}})
        .as_object()
        .unwrap()
        .clone(),
    };
    assert!(
        pin_metadata::apply(
            &context,
            &request,
            &malformed,
            &mut candidate,
            &HashSet::new()
        )
        .is_err()
    );
    assert_eq!(candidate, malformed);

    let request = json!({"pins":pins,"pin_metadata":{key.clone():good}});
    let stale = edit(&model, &root, unit, request.clone()).unwrap();
    let prepared = edit(&model, &root, unit, request).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    let mut expected = before.clone();
    expected["pins"] = pins;
    expected["pins"][&key]["swap_group"] = json!(u32::MAX);
    expected["pins"][&key]["alternates"] = json!([
        {"name":" revised ","kind":"","vendor":{"value":[1,2]}},addition
    ]);
    assert_eq!(read(&model), expected);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(read(&model), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        read(&ProjectResolver::new(&root).resolve().unwrap()),
        expected
    );
}
