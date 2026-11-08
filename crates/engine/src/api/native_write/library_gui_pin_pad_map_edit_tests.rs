//! Saved assignment edits preserve canonical raw cells and refusal boundaries.
use super::*;
fn edit(f: &F, id: Uuid, fields: Value) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        json!({"kind":"pin_pad_maps","object_id":id,"pool_path":"pool","fields":fields}),
    )
}

#[test]
fn saved_assignments_preserve_raw_cells_identity_and_history() {
    let mut f = fixture();
    let id = Uuid::new_v4();
    let mut original = maps::pin_pad_map_payload(
        id,
        f.part,
        Some(f.foot),
        &[
            PinPadMapMapping {
                pad: f.pad,
                gate: f.gate,
                pin: f.pin,
            },
            PinPadMapMapping {
                pad: f.pad2,
                gate: f.gate2,
                pin: f.pin,
            },
        ],
    );
    original["x_vendor"] = json!({"ordered":[3,1,3]});
    original["mappings"][f.pad.to_string()]["x_cell"] = json!(["keep", 17]);
    original["mappings"][f.pad2.to_string()]["x_cell"] = json!({"retain":true});
    original.as_object_mut().unwrap().remove("tags");
    let prepared = library::build_pool_library_write(
        &f.model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "pin_pad_maps", id),
            object: original.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let before_part = raw_part(&f);
    let before = journal(&f);
    let revision = f.model.model_revision.clone();
    // Explicitly unmap the second cell and reassign the first. Unknown data in
    // the retained cell is not resubmitted by the form and must survive.
    let fields = json!({"mappings":{f.pad.to_string():{"gate":f.gate2,"pin":f.pin}}});
    let prepared = edit(&f, id, fields).unwrap();
    assert_eq!(journal(&f), before);
    assert_eq!(f.model.model_revision, revision);
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let mut expected = original.clone();
    expected["mappings"]
        .as_object_mut()
        .unwrap()
        .remove(&f.pad2.to_string());
    expected["mappings"][f.pad.to_string()]["gate"] = json!(f.gate2);
    let path = format!("pool/pin_pad_maps/{id}.json");
    assert_eq!(
        library::pool_library_object_payload(&f.model, &path, id).unwrap(),
        expected
    );
    assert_eq!(
        f.model
            .materialized_pool_object_revision(
                id,
                "pin_pad_maps",
                crate::substrate::ObjectRevision(0)
            )
            .unwrap(),
        original
    );
    assert_eq!(raw_part(&f), before_part);
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(
        library::pool_library_object_payload(&f.model, &path, id).unwrap(),
        original
    );
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        library::pool_library_object_payload(&f.model, &path, id).unwrap(),
        expected
    );
    assert_eq!(raw_part(&f), before_part);
    std::fs::remove_dir_all(&f.root).unwrap();
}

#[test]
fn saved_mapping_forbidden_fields_and_invalid_assignments_refuse_without_writes() {
    let mut f = fixture();
    let id = Uuid::new_v4();
    let prepared = prepare(&f, id, fields(&f, false)).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let before = journal(&f);
    let path = format!("pool/pin_pad_maps/{id}.json");
    let raw = library::pool_library_object_payload(&f.model, &path, id).unwrap();
    for forbidden in [
        "uuid",
        "part",
        "footprint",
        "assign_default",
        "tags",
        "name",
        "lifecycle",
    ] {
        assert!(
            edit(&f, id, json!({forbidden:Value::Null})).is_err(),
            "{forbidden}"
        );
    }
    for rows in [
        json!({}),
        json!({f.pad.to_string():{"gate":f.gate,"pin":f.pin},f.pad2.to_string():{"gate":f.gate,"pin":f.pin}}),
        json!({Uuid::new_v4().to_string():{"gate":f.gate,"pin":f.pin}}),
    ] {
        assert!(edit(&f, id, json!({"mappings":rows})).is_err());
    }
    assert_eq!(journal(&f), before);
    assert_eq!(
        library::pool_library_object_payload(&f.model, &path, id).unwrap(),
        raw
    );
    std::fs::remove_dir_all(&f.root).unwrap();
}

#[test]
fn restored_map_counter_preserves_exact_pinned_revision_and_refuses_collision() {
    let mut f = fixture();
    let map = Uuid::new_v4();
    let prepared = prepare(&f, map, fields(&f, false)).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let history = json!({"mappings":{f.pad.to_string():{"gate":f.gate2,"pin":f.pin}}});
    let prepared = edit(&f, map, history.clone()).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    assert_eq!(f.model.objects[&map].object_revision.0, 0);
    assert_eq!(
        f.model
            .materialized_pool_object_revision(
                map,
                "pin_pad_maps",
                crate::substrate::ObjectRevision(1)
            )
            .unwrap()["mappings"],
        history["mappings"]
    );
    let mut op = native::component_instances::build_placed_symbol_component_instance_op(
        &f.model, f.placed, f.symbol, f.part,
    )
    .unwrap();
    let ci = if let Operation::CreateComponentInstance {
        component_instance_id,
        component_instance,
    } = &mut op
    {
        component_instance["library_bindings"][Uuid::new_v4().to_string()] =
            json!({"target_object_id":map,"pinned_object_revision":1,"binding_role":"pin_pad_map"});
        *component_instance_id
    } else {
        unreachable!()
    };
    let batch = native::build_batch(&f.model, provenance(), vec![op]).unwrap();
    f.model.commit_journaled(&f.root, batch).unwrap();
    let declared = f.model.component_instances[&ci].clone();
    let bytes = std::fs::read(f.root.join(format!("pool/pin_pad_maps/{map}.json"))).unwrap();
    let before = journal(&f);
    let conflict = json!({"mappings":{f.pad.to_string():{"gate":f.gate,"pin":f.pin}}});
    assert!(
        edit(&f, map, conflict)
            .unwrap_err()
            .to_string()
            .contains("ambiguous")
    );
    assert_eq!(journal(&f), before);
    assert_eq!(
        std::fs::read(f.root.join(format!("pool/pin_pad_maps/{map}.json"))).unwrap(),
        bytes
    );
    assert_eq!(f.model.component_instances[&ci], declared);
    let prepared = edit(&f, map, history).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.model.component_instances[&ci], declared);
    assert!(
        !f.model
            .diagnostics
            .iter()
            .any(|d| d.code == "invalid_component_instance_shard")
    );
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn saved_map_sparse_change_keeps_unchanged_raw_uuid_spellings_and_cell_metadata() {
    let mut f = fixture();
    let map = Uuid::new_v4();
    let mut original = maps::pin_pad_map_payload(
        map,
        f.part,
        Some(f.foot),
        &[
            PinPadMapMapping {
                pad: f.pad,
                gate: f.gate,
                pin: f.pin,
            },
            PinPadMapMapping {
                pad: f.pad2,
                gate: f.gate2,
                pin: f.pin,
            },
        ],
    );
    // These accepted UUID representations must not be rewritten merely because
    // another selected pad is unmapped or a different assignment member changes.
    original["mappings"][f.pad.to_string()]["gate"] = json!(f.gate.simple().to_string());
    original["mappings"][f.pad.to_string()]["pin"] = json!(f.pin.simple().to_string());
    original["mappings"][f.pad.to_string()]["x_cell"] = json!({"ordered":[3,1,3]});
    let p = library::build_pool_library_write(
        &f.model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "pin_pad_maps", map),
            object: original.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    let p = edit(
        &f,
        map,
        json!({"mappings":{f.pad.to_string():{"gate":f.gate,"pin":f.pin}}}),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    let path = format!("pool/pin_pad_maps/{map}.json");
    let mut expected = original.clone();
    expected["mappings"]
        .as_object_mut()
        .unwrap()
        .remove(&f.pad2.to_string());
    assert_eq!(
        library::pool_library_object_payload(&f.model, &path, map).unwrap(),
        expected
    );
    let p = edit(
        &f,
        map,
        json!({"mappings":{f.pad.to_string():{"gate":f.gate2,"pin":f.pin}}}),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, p).unwrap();
    expected["mappings"][f.pad.to_string()]["gate"] = json!(f.gate2);
    assert_eq!(
        library::pool_library_object_payload(&f.model, &path, map).unwrap(),
        expected
    );
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        library::pool_library_object_payload(&f.model, &path, map).unwrap(),
        expected
    );
    std::fs::remove_dir_all(f.root).unwrap();
}
