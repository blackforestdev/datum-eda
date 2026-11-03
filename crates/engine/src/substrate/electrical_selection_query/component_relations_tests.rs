//! Actual native assignment authority; ownership and cross-domain relations differ.
use super::*;
use crate::api::native_write::{
    BatchComposer, WriteProvenance, commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_assignment::tests::fixture,
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
};
use crate::substrate::{CommitSource, Operation, ProjectResolver};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "binding Inspector")
}
fn origin(class: &str, id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id: id,
        instance_path: vec![],
    }
}
fn context(model: &DesignModel, at: ElectricalOccurrence) -> ComponentSelectionContext {
    let snapshot = ElectricalSelectionSnapshot::capture(model).unwrap();
    snapshot
        .component_selection_context(snapshot.revision(), &BTreeSet::from([at]))
        .unwrap()
}
fn raw(model: &DesignModel, id: Uuid) -> Value {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&id].source_shard_id)
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()
}
fn setup(name: &str) -> (PathBuf, DesignModel, Uuid, Uuid, Uuid) {
    let (root, mut model, symbol, part) = fixture(name);
    let unbound = context(&model, origin("symbols", symbol));
    assert!(unbound.bindings.is_empty());
    assert!(unbound.relations.is_empty());
    assert_eq!(unbound.unavailable.len(), 1);
    let proposal_id = Uuid::new_v4();
    let mut ci = None;
    for action in [
        Action::CreateAssignment {
            proposal_id,
            symbol,
            instance_path: vec![],
            part,
            gate: "40000000-0000-4000-8000-000000000101".parse().unwrap(),
        },
        Action::AcceptApply { proposal_id },
    ] {
        let request = NativeProposalRequest {
            project_root: root.clone(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        };
        ci = proposal::execute(&mut model, &request)
            .unwrap()
            .assignment
            .map(|a| a.component_instance);
    }
    (root, model, symbol, ci.unwrap(), part)
}
fn schematic_only(model: &DesignModel, symbol: Uuid, ci: Uuid) {
    let at = origin("symbols", symbol);
    let result = context(model, at.clone());
    assert_eq!(
        result.bindings[&at],
        ComponentSelectionBinding {
            component_instance: ci,
            counterpart_status: ComponentCounterpartStatus::NotDeclared
        }
    );
    assert!(result.relations.is_empty());
    assert!(result.unavailable.is_empty());
}
#[test]
fn component_binding_context_schematic_only_survives_edits_history_and_reopen() {
    let (root, mut model, symbol, ci, _) = setup("binding-context-schematic");
    schematic_only(&model, symbol, ci);
    let write = (find_native_write_verb("datum.schematic.set_symbol_value")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"symbol":symbol,"instance_path":[],"value":"updated"}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    schematic_only(&model, symbol, ci);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    schematic_only(&model, symbol, ci);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    schematic_only(&model, symbol, ci);
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert!(
        snapshot
            .component_selection_context(
                &ModelRevision("stale".into()),
                &BTreeSet::from([origin("symbols", symbol)])
            )
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}
fn physical(model: &mut DesignModel, root: &Path, symbol: Uuid, ci: Uuid, part: Uuid) -> Uuid {
    use crate::api::native_write::board_components::{
        BoardPackagePlacement, build_place_board_package,
    };
    let package = crate::board::PlacedPackage {
        uuid: Uuid::new_v4(),
        part,
        package: "50000000-0000-4000-8000-000000000001".parse().unwrap(),
        reference: "U1".into(),
        value: "".into(),
        position: crate::ir::geometry::Point::zero(),
        rotation: 0,
        layer: 1,
        locked: false,
    };
    let write = build_place_board_package(
        model,
        provenance(),
        &BoardPackagePlacement {
            package: package.clone(),
            materialized: json!({}),
        },
    )
    .unwrap();
    commit_prepared(model, root, write).unwrap();
    let previous = raw(model, ci)["component_instance"].clone();
    let mut next = previous.clone();
    next["object_revision"] = json!(model.objects[&ci].object_revision.0 + 1);
    next["placed_package_refs"] = json!([{"object_id":package.uuid,"object_revision":model.objects[&package.uuid].object_revision.0}]);
    let write = BatchComposer::compose(model, provenance())
        .push_op(Operation::SetComponentInstance {
            component_instance_id: ci,
            previous_component_instance: previous,
            component_instance: next,
        })
        .finish()
        .unwrap();
    commit_prepared(model, root, write).unwrap();
    assert_eq!(
        model.component_instances[&ci].placed_symbol_refs,
        vec![symbol]
    );
    package.uuid
}
#[test]
fn component_binding_context_keeps_physical_bidirectional_and_declared_stale_unavailable() {
    let (root, mut model, symbol, ci, part) = setup("binding-context-physical");
    let package = physical(&mut model, &root, symbol, ci, part);
    model = ProjectResolver::new(&root).resolve().unwrap();
    for (class, id, other) in [("symbols", symbol, package), ("packages", package, symbol)] {
        let at = origin(class, id);
        let result = context(&model, at.clone());
        assert_eq!(
            result.bindings[&at].counterpart_status,
            ComponentCounterpartStatus::Resolved
        );
        assert_eq!(result.bindings[&at].component_instance, ci);
        assert!(result.unavailable.is_empty());
        assert_eq!(result.relations.len(), 1);
        assert_eq!(
            result.relations.first().unwrap().counterpart.source_id,
            other
        );
    }
    // A declared package with an advanced source epoch is not "not declared".
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetBoardPackageValue {
            package_id: package,
            value: "external edit".into(),
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    for reopened in [false, true] {
        if reopened {
            model = ProjectResolver::new(&root).resolve().unwrap();
        }
        let result = context(&model, origin("symbols", symbol));
        assert!(result.bindings.is_empty());
        assert!(result.relations.is_empty());
        assert_eq!(result.unavailable.len(), 1);
    }
    let package_raw = raw(&model, package)["packages"][package.to_string()].clone();
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::DeleteBoardPackage {
            package_id: package,
            package: package_raw,
            materialized: json!({}),
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let result = context(&model, origin("symbols", symbol));
    assert!(result.bindings.is_empty());
    assert!(result.relations.is_empty());
    assert_eq!(result.unavailable.len(), 1);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn component_binding_context_ambiguous_owners_and_repeated_occurrences_remain_unavailable() {
    let (root, mut model, symbol, ci, _) = setup("binding-context-ambiguous");
    let mut other = raw(&model, ci)["component_instance"].clone();
    let id = Uuid::new_v4();
    other["uuid"] = json!(id);
    other["object_revision"] = json!(0);
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::CreateComponentInstance {
            component_instance_id: id,
            component_instance: other.clone(),
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let result = context(&model, origin("symbols", symbol));
    assert!(result.bindings.is_empty());
    assert!(result.relations.is_empty());
    assert!(
        result
            .unavailable
            .values()
            .next()
            .unwrap()
            .contains("Multiple")
    );
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::DeleteComponentInstance {
            component_instance_id: id,
            component_instance: other,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    use crate::api::native_write::schematic_sheets::{
        build_create_schematic_definition, build_create_schematic_sheet_instance,
    };
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let source = snapshot.schematic_source().unwrap();
    let sheet = source
        .sheets
        .values()
        .find(|s| s.symbols.contains_key(&symbol))
        .unwrap()
        .uuid;
    let schematic = source.uuid;
    let definition = crate::schematic::SheetDefinition {
        uuid: Uuid::new_v4(),
        name: "Repeated".into(),
        root_sheet: sheet,
    };
    let write = build_create_schematic_definition(
        &model,
        provenance(),
        schematic,
        definition.uuid,
        "definitions/repeated.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut origins = BTreeSet::new();
    for index in 0..2 {
        let instance = crate::schematic::SheetInstance {
            uuid: Uuid::new_v4(),
            definition: definition.uuid,
            parent_sheet: None,
            position: crate::ir::geometry::Point::new(index, 0),
            name: format!("I{index}"),
            ports: vec![],
        };
        origins.insert(ElectricalOccurrence {
            class: "symbols".into(),
            source_id: symbol,
            instance_path: vec![instance.uuid],
        });
        let write = build_create_schematic_sheet_instance(
            &model,
            provenance(),
            schematic,
            instance.uuid,
            serde_json::to_value(instance).unwrap(),
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let result = snapshot
        .component_selection_context(snapshot.revision(), &origins)
        .unwrap();
    assert!(result.bindings.is_empty());
    assert!(result.relations.is_empty());
    assert_eq!(result.unavailable.len(), 2);
    std::fs::remove_dir_all(root).unwrap();
}
