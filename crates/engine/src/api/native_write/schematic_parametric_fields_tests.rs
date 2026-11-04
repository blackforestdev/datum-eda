//! Shared field selectors preserve the same payload and lifecycle as GUI edits.
use super::*;

#[test]
fn native_parametric_fields_raw_metadata_and_cold_history() {
    let (mut f, _, _) = setup("parametric-fields-history");
    let mut expected = raw(&f.model, f.symbol.uuid);
    for patch in [
        json!({"field":"reference","value":"U901"}),
        json!({"field":"value","value":"literal value"}),
        json!({"field":"value","clear":true}),
    ] {
        let prepared = build(&f, patch.clone()).unwrap();
        expected[patch["field"].as_str().unwrap()] =
            patch.get("value").cloned().unwrap_or(json!(""));
        roundtrip(&mut f, prepared, &expected);
    }
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn native_parametric_fields_invalid_and_stale_edits_leave_files_unchanged() {
    let (mut f, first, _) = setup("parametric-fields-refusals");
    let before = bytes(&f.root);
    for patch in [
        json!({"field":"reference","clear":true}),
        json!({"field":"reference","value":""}),
        json!({"field":"reference","value":"U 9"}),
        json!({"field":"reference","value":f.symbol.reference}),
        json!({"field":"value","value":"two\nlines"}),
        json!({"field":"value"}),
        json!({"field":"value","clear":true,"value":""}),
        json!({"field":"value","value":"new","visible":false}),
        json!({"field":"value","value":"new","position":{"x":0,"y":0}}),
        json!({"field":"value","value":"new","field_id":first}),
        json!({"field":"reference","value":"U9","key":"Reference"}),
        json!({"field":"value","value":"new","symbol":Uuid::new_v4()}),
        json!({"field":"value","value":"new","instance_path":[Uuid::new_v4()]}),
        json!({"field":"part","value":"unsupported"}),
    ] {
        assert!(build(&f, patch.clone()).is_err(), "accepted {patch}");
        assert_eq!(bytes(&f.root), before);
    }
    let stale = build(&f, json!({"field":"reference","value":"U902"})).unwrap();
    let newer = build(
        &f,
        json!({"field":"custom","field_id":first,"value":"newer"}),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, newer).unwrap();
    let before = bytes(&f.root);
    assert!(commit_prepared(&mut f.model, &f.root, stale).is_err());
    assert_eq!(bytes(&f.root), before);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn native_parametric_fields_single_and_multi_gate_binding_history() {
    use crate::api::native_write::{
        schematic_delete_proposal::NativeProposalAction as Action,
        schematic_gate_placement::{self as gates, tests as fixture},
    };
    let (root, mut model, first, req) = fixture::setup("parametric-fields-gates");
    for multi in [false, true] {
        let target = if multi {
            let second = gates::preview(&model, &req).unwrap().symbol.uuid;
            fixture::action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
            fixture::action(
                &mut model,
                &root,
                Action::AcceptApply {
                    proposal_id: req.proposal_id,
                },
            );
            second
        } else {
            first
        };
        if multi {
            let before = bytes(&root);
            let duplicate = (find_native_write_verb("datum.schematic.set_symbol_field")
                .unwrap()
                .build)(
                &NativeWriteContext {
                    model: &model,
                    project_root: &root,
                },
                fixture::provenance(),
                json!({"symbol":target,"instance_path":[],"field":"reference","value":"U902"}),
            );
            assert!(
                duplicate
                    .unwrap_err()
                    .to_string()
                    .contains("already in use")
            );
            assert_eq!(bytes(&root), before);
        }
        for (field, value) in [
            ("reference", if multi { "U903" } else { "U902" }),
            ("value", "updated"),
            ("value", ""),
        ] {
            let before = fixture::raw(&model, target);
            let binding = fixture::binding(&model, req.component_instance);
            let identities = model.electrical_identities.clone();
            let partitions = crate::substrate::electrical_test_partitions(&model, &[]).unwrap();
            let mut params =
                json!({"symbol":target,"instance_path":[],"field":field,"value":value});
            if value.is_empty() {
                params.as_object_mut().unwrap().remove("value");
                params["clear"] = json!(true);
            }
            let write = (find_native_write_verb("datum.schematic.set_symbol_field")
                .unwrap()
                .build)(
                &NativeWriteContext {
                    model: &model,
                    project_root: &root,
                },
                fixture::provenance(),
                params,
            )
            .unwrap();
            let mut expected = before.clone();
            expected["symbols"][target.to_string()][field] = json!(value);
            commit_prepared(&mut model, &root, write).unwrap();
            model = ProjectResolver::new(&root).resolve().unwrap();
            assert_eq!(fixture::raw(&model, target), expected);
            assert_eq!(fixture::binding(&model, req.component_instance), binding);
            model
                .commit_journal_undo(&root, fixture::provenance().into())
                .unwrap();
            model = ProjectResolver::new(&root).resolve().unwrap();
            assert_eq!(fixture::raw(&model, target), before);
            assert_eq!(fixture::binding(&model, req.component_instance), binding);
            model
                .commit_journal_redo(&root, fixture::provenance().into())
                .unwrap();
            model = ProjectResolver::new(&root).resolve().unwrap();
            assert_eq!(fixture::raw(&model, target), expected);
            assert_eq!(fixture::binding(&model, req.component_instance), binding);
            assert_eq!(model.electrical_identities, identities);
            assert_eq!(
                crate::substrate::electrical_test_partitions(&model, &[]).unwrap(),
                partitions
            );
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
