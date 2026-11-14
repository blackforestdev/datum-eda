//! Preserve the established partial singleton lifecycle while keeping gate growth strict.
use super::tests::{action, binding, provenance, raw};
use super::*;
use crate::api::native_write::{
    commit_prepared,
    context::BatchComposer,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_delete_proposal::{DeleteClass, DeleteTarget, NativeProposalAction as Action},
};
use crate::substrate::ProjectResolver;
use std::path::Path;
fn native(model: &mut DesignModel, root: &Path, verb: &str, params: Value) {
    let write = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    commit_prepared(model, root, write).unwrap();
}
fn set_owner(model: &mut DesignModel, root: &Path, ci: Uuid, mut next: Value) {
    let old = raw(model, ci)["component_instance"].clone();
    next["object_revision"] = json!(model.objects[&ci].object_revision.0 + 1);
    let write = BatchComposer::compose(model, provenance())
        .push_op(Operation::SetComponentInstance {
            component_instance_id: ci,
            previous_component_instance: old,
            component_instance: next,
        })
        .finish()
        .unwrap();
    commit_prepared(model, root, write).unwrap();
}
#[test]
fn native_gate_preserves_partial_singleton_edit_delete_assignment_history() {
    let (root, mut model, first, part) =
        crate::api::native_write::schematic_assignment::tests::fixture(
            "partial-singleton-compatibility",
        );
    let original =
        crate::api::native_write::schematic_assignment::options(&model, first, &[]).unwrap();
    let sheet = original.sheet;
    // A display-only override is supported by existing singleton placement,
    // Move and assignment. It must not acquire the stricter multi-gate rule.
    let mut symbol = raw(&model, first)["symbols"][first.to_string()].clone();
    // The shared native fixture already resolves library identities. Author the
    // partial legacy state explicitly instead of assuming those fields are absent.
    for field in ["part", "entity", "gate"] {
        symbol.as_object_mut().unwrap().remove(field);
    }
    let display = crate::schematic::PinDisplayOverride {
        pin: original.original.pins[0].uuid,
        visible: false,
        position: None,
    };
    symbol["pin_overrides"] = serde_json::to_value(vec![display]).unwrap();
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetSchematicSymbol {
            sheet_id: sheet,
            symbol_id: first,
            symbol,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let partial = source(&model).unwrap().sheets[&sheet].symbols[&first].clone();
    assert!(partial.part.is_none() && partial.entity.is_none() && partial.gate.is_none());
    let ci = Uuid::new_v4();
    let payload = json!({"uuid":ci,"object_revision":0,"part_ref":null,"library_bindings":{},"placed_symbol_refs":[{"object_id":first,"object_revision":model.objects[&first].object_revision.0}],"placed_package_refs":[],"placed_symbol_roles":{first.to_string():{"role":"primary","label":"partial native"}},"placed_package_roles":{}});
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::CreateComponentInstance {
            component_instance_id: ci,
            component_instance: payload,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(model.component_instances[&ci].part_ref.is_none());
    assert!(options(&model, first, &[]).is_err()); // Cannot add another Gate yet.
    assert_eq!(
        crate::api::native_write::schematic_assignment::options(&model, first, &[])
            .unwrap()
            .component_instance,
        Some(ci)
    );
    for (verb, params) in [
        (
            "datum.schematic.move_symbol",
            json!({"symbol":first,"instance_path":[],"x_nm":30_000_000,"y_nm":40_000_000,"rotation_deg":90,"mirrored":true}),
        ),
        (
            "datum.schematic.set_symbol_reference",
            json!({"symbol":first,"instance_path":[],"reference":"U5","value":"partial owner"}),
        ),
        (
            "datum.schematic.set_symbol_field",
            json!({"symbol":first,"instance_path":[],"field":"custom","key":"note","value":"still partial"}),
        ),
    ] {
        let before = raw(&model, first);
        let old = binding(&model, ci);
        native(&mut model, &root, verb, params);
        let after = raw(&model, first);
        assert_eq!(binding(&model, ci), old);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, first), before);
        assert_eq!(binding(&model, ci), old);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, first), after);
        assert_eq!(binding(&model, ci), old);
    }
    let before = raw(&model, first);
    let old = binding(&model, ci);
    let draft = action(
        &mut model,
        &root,
        Action::CreateDelete {
            proposal_id: Uuid::new_v4(),
            sheet,
            instance_path: vec![],
            targets: vec![DeleteTarget {
                class: DeleteClass::Symbol,
                id: first,
            }],
        },
    );
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    assert!(!model.component_instances.contains_key(&ci));
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, first), before);
    assert_eq!(binding(&model, ci), old);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert!(!model.component_instances.contains_key(&ci));
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(binding(&model, ci), old);
    // A second incomplete member cannot inherit the legacy singleton exemption.
    let lib = original.original.lib_id.unwrap();
    native(
        &mut model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":sheet,"library_symbol":lib,"reference":"U6","value":"unbound sibling","x_nm":60_000_000,"y_nm":40_000_000}),
    );
    let source = source(&model).unwrap();
    let second = source.sheets[&sheet]
        .symbols
        .values()
        .find(|s| s.reference == "U6")
        .unwrap()
        .uuid;
    let singleton = raw(&model, ci)["component_instance"].clone();
    let mut multiple = singleton.clone();
    multiple["placed_symbol_refs"].as_array_mut().unwrap().push(
        json!({"object_id":second,"object_revision":model.objects[&second].object_revision.0}),
    );
    set_owner(&mut model, &root, ci, multiple);
    assert!(
        crate::api::native_write::schematic_assignment::repin_symbols(
            &model,
            &[source.sheets[&sheet].symbols[&first].clone()]
        )
        .is_err()
    );
    assert!(
        crate::api::native_write::schematic_delete_proposal::bound_symbol_owner(&model, first, &[])
            .is_err()
    );
    assert!(options(&model, first, &[]).is_err());
    set_owner(&mut model, &root, ci, singleton);
    let before = raw(&model, first);
    let old = binding(&model, ci);
    let draft = action(
        &mut model,
        &root,
        Action::CreateAssignment {
            component: None,
            proposal_id: Uuid::new_v4(),
            symbol: first,
            instance_path: vec![],
            part,
            gate: "40000000-0000-4000-8000-000000000101".parse().unwrap(),
        },
    );
    assert_eq!(draft.assignment.unwrap().component_instance, ci);
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    assert_eq!(model.component_instances[&ci].part_ref, Some(part));
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, first), before);
    assert_eq!(binding(&model, ci), old);
    assert!(model.component_instances[&ci].part_ref.is_none());
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(model.component_instances[&ci].part_ref, Some(part));
    binding(&model, ci);
    assert!(
        options(&model, first, &[])
            .unwrap_err()
            .to_string()
            .contains("pin overrides")
    );
    assert_eq!(
        raw(&model, first)["symbols"][first.to_string()]["pin_overrides"],
        before["symbols"][first.to_string()]["pin_overrides"]
    );
    std::fs::remove_dir_all(root).unwrap();
}
