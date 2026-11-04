//! Existing native controls preserve raw payload through canonical edits and history.
use super::*;
use crate::api::native_write::{
    commit_prepared, electrical::topology_tests::fixture, registry::find_native_write_verb,
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver, electrical_test_partitions};
use serde_json::{Value, json};
use std::path::Path;
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "label/port raw payload")
}
fn build(
    model: &DesignModel,
    root: &Path,
    verb: &str,
    params: Value,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
}
fn source(model: &DesignModel, sheet: Uuid, id: Uuid) -> Value {
    source_sheet_payload(model, sheet, id).unwrap()
}
fn params(port: bool, sheet: Uuid, id: Option<Uuid>, name: &str) -> Value {
    if port {
        json!({"sheet":sheet,"port":id,"instance_path":[],"name":name,"direction":"input","x_nm":100,"y_nm":200})
    } else if let Some(id) = id {
        json!({"label":id,"instance_path":[],"name":name})
    } else {
        json!({"sheet":sheet,"name":name,"kind":"local","x_nm":100,"y_nm":200})
    }
}
fn case(port: bool) {
    let (root, mut model, _, sheet) = fixture("label_port_payload_history");
    let create = if port {
        "datum.schematic.place_port"
    } else {
        "datum.schematic.place_label"
    };
    let edit = if port {
        create
    } else {
        "datum.schematic.rename_label"
    };
    let class = if port { "ports" } else { "labels" };
    let write = build(&model, &root, create, params(port, sheet, None, "BEFORE")).unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut sibling = params(port, sheet, None, "OTHER");
    sibling["x_nm"] = json!(5000);
    let write = build(&model, &root, create, sibling).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let key = id.to_string();
    let mut raw = source(&model, sheet, id)[class][&key].clone();
    raw["extension"] = json!({"future":[false,9,"verbatim"],"reserved":null});
    raw["position"]["extension"] = json!({"units":"nm","opaque":[1,2]});
    let op = if port {
        Operation::SetSchematicPort {
            sheet_id: sheet,
            port_id: id,
            port: raw,
        }
    } else {
        Operation::SetSchematicLabel {
            sheet_id: sheet,
            label_id: id,
            label: raw,
        }
    };
    let write = BatchComposer::compose(&model, provenance())
        .push_op(op)
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let before = source(&model, sheet, id);
    let partitions = electrical_test_partitions(&model, &[]).unwrap();
    let identities: Vec<_> = model.electrical_identities.keys().copied().collect();
    let count = model.journal.len();
    let mut change = params(port, sheet, Some(id), "AFTER");
    change[if port { "direction" } else { "kind" }] =
        json!(if port { "bidirectional" } else { "power" });
    let write = build(&model, &root, edit, change.clone()).unwrap();
    assert_eq!(write.primary_object_id, Some(id));
    assert_eq!(
        source(&ProjectResolver::new(&root).resolve().unwrap(), sheet, id),
        before,
        "Prepared/cancelled edit has no writes"
    );
    assert_eq!(model.journal.len(), count);
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let mut expected = before.clone();
    expected[class][&key]["name"] = json!("AFTER");
    expected[class][&key][if port { "direction" } else { "kind" }] =
        json!(if port { "Bidirectional" } else { "Power" });
    assert_eq!(source(&model, sheet, id), expected);
    assert_eq!(model.journal.len(), count + 1);
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
    assert_eq!(
        model
            .electrical_identities
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        identities
    );
    let count = model.journal.len();
    assert!(
        build(&model, &root, edit, change.clone()).is_err(),
        "No-op refuses before commit"
    );
    for bad in [
        "name",
        "instance_path",
        if port { "sheet" } else { "label" },
        if port { "direction" } else { "kind" },
    ] {
        let mut request = change.clone();
        request[bad] = match bad {
            "name" => json!("bad\nname"),
            "instance_path" => json!([Uuid::new_v4()]),
            "sheet" | "label" => json!(Uuid::new_v4()),
            _ => json!("invalid"),
        };
        assert!(build(&model, &root, edit, request).is_err(), "{bad}");
    }
    assert_eq!(model.journal.len(), count);
    assert_eq!(source(&model, sheet, id), expected);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model, sheet, id), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model, sheet, id), expected);
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
    if port {
        // Existing native x/y parameters retain nested metadata when explicitly changed.
        change["x_nm"] = json!(300);
        change["y_nm"] = json!(-400);
    } else {
        // Omitted kind preserves the source's current Power kind exactly.
        change.as_object_mut().unwrap().remove("kind");
        change["name"] = json!("AFTER_AGAIN");
    }
    let prepared = build(&model, &root, edit, change).unwrap();
    let previous = expected.clone();
    if port {
        expected[class][&key]["position"]["x"] = json!(300);
        expected[class][&key]["position"]["y"] = json!(-400);
    } else {
        expected[class][&key]["name"] = json!("AFTER_AGAIN");
    }
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model, sheet, id), expected);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model, sheet, id), previous);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(source(&model, sheet, id), expected);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_named_edit_label_raw_payload_and_history() {
    case(false);
}
#[test]
fn native_named_edit_port_raw_payload_and_history() {
    case(true);
}
