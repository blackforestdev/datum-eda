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
fn physical(
    model: &mut DesignModel,
    root: &Path,
    symbol: Uuid,
    ci: Uuid,
    part: Uuid,
    position: crate::ir::geometry::Point,
) -> Uuid {
    use crate::api::native_write::board_components::{
        BoardPackagePlacement, build_place_board_package,
    };
    let package = crate::board::PlacedPackage {
        uuid: Uuid::new_v4(),
        part,
        package: "50000000-0000-4000-8000-000000000001".parse().unwrap(),
        reference: "U1".into(),
        value: "".into(),
        position,
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
    let package = physical(
        &mut model,
        &root,
        symbol,
        ci,
        part,
        crate::ir::geometry::Point::zero(),
    );
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
fn saved_part_metadata_retains_resolved_physical_binding_history_and_exact_source_pins() {
    use crate::api::native_write::board_components::pad_materialization::build_materialize_component_pad_sources;
    use crate::substrate::component_instance::pool_history::PoolHistoryBasis;
    use crate::substrate::{ObjectRevision, SourceShardTaxon, component_pad_source as pins};
    let (root, mut model, symbol, ci, part, package, _map, request) =
        physical_fixture::eight_pad_fixture("saved-part-history-physical");
    let board = model
        .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    let projected = |model: &DesignModel| {
        let source = model
            .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
            .unwrap();
        let projection = pins::component_pad_projection(model, &source).unwrap();
        assert!(
            projection.unavailable.is_empty(),
            "{:?}",
            projection.unavailable
        );
        assert_eq!(projection.query_pads.len(), 8);
        json!({"pads":projection.pads,"queries":projection.query_pads.iter().map(|(id,p)|(*id,serde_json::to_value(p.terminal()).unwrap())).collect::<BTreeMap<_,_>>()})
    };
    let pad_projection = projected(&model);
    let before = model.clone();
    let original = raw(&model, part);
    let ci_raw = raw(&model, ci);
    let declared = model.component_instances[&ci].clone();
    let pin = pins::capture_root(&model, part, SourceShardTaxon::PoolPart).unwrap();
    let revision = model.objects[&part].object_revision;
    let prepared = (find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"kind":"parts","object_id":part,"pool_path":"pool",
            "fields":{"mpn":"REVISED-PART","value":"Revised source value"}}),
    )
    .unwrap();
    let next = ObjectRevision(revision.0 + 1);
    // A pending later Set is not accepted evidence for an earlier CI operation.
    let pending = PoolHistoryBasis {
        model: &before,
        operations: &prepared.batch.operations,
        applied_operations: &[],
    };
    assert!(!pending.definition_ref_matches(&before.objects, part, next, "parts"));
    let mut staged = before.objects.clone();
    staged.get_mut(&part).unwrap().object_revision = next;
    let applied = PoolHistoryBasis {
        model: &before,
        operations: &prepared.batch.operations,
        applied_operations: &prepared.batch.operations,
    };
    assert!(applied.definition_ref_matches(&staged, part, revision, "parts"));
    assert!(applied.definition_ref_matches(&staged, part, next, "parts"));
    assert!(!applied.definition_ref_matches(&staged, part, ObjectRevision(next.0 + 1), "parts"));
    let stale = prepared.clone();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let corrected = raw(&model, part);
    let mut expected = original.clone();
    expected["mpn"] = json!("REVISED-PART");
    expected["value"] = json!("Revised source value");
    assert_eq!(corrected, expected);
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    let verify = |model: &DesignModel| {
        assert_eq!(
            model
                .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
                .unwrap(),
            board
        );
        assert_eq!(
            projected(model),
            pad_projection,
            "all8 frozen Pad identities/geometry/mapping query projections remain exact"
        );
        assert_eq!(model.component_instances[&ci], declared);
        assert_eq!(raw(model, ci), ci_raw);
        assert!(
            !model
                .diagnostics
                .iter()
                .any(|d| d.code.starts_with("component_instance_")),
            "{:?}",
            model.diagnostics
        );
        assert_eq!(
            model
                .materialized_pool_object_revision(part, "parts", revision)
                .unwrap(),
            original
        );
        assert_eq!(pins::source_value(model, &pin).unwrap(), original);
        pins::check_pin(model, &pin).unwrap();
        for (class, id, other) in [("symbols", symbol, package), ("packages", package, symbol)] {
            let at = origin(class, id);
            let result = context(model, at.clone());
            assert!(result.unavailable.is_empty());
            assert_eq!(result.bindings[&at].component_instance, ci);
            assert_eq!(
                result.bindings[&at].counterpart_status,
                ComponentCounterpartStatus::Resolved
            );
            assert_eq!(result.relations.len(), 1);
            assert_eq!(
                result.relations.first().unwrap().counterpart.source_id,
                other
            );
        }
    };
    verify(&model);
    for tamper in 0..4 {
        let mut bad = pin.clone();
        match tamper {
            0 => bad.owner_value_hash = "altered".into(),
            1 => bad.record_hash = "altered".into(),
            2 => bad.incarnation = Some(Uuid::new_v4()),
            _ => bad.source_shard_id = Uuid::new_v4(),
        }
        assert!(pins::check_pin(&model, &bad).is_err());
        assert!(pins::source_value(&model, &bad).is_err());
    }
    let mut wrong_kind = model.clone();
    wrong_kind.objects.get_mut(&part).unwrap().kind = "entities".into();
    assert!(pins::check_pin(&wrong_kind, &pin).is_err());
    assert!(pins::source_value(&wrong_kind, &pin).is_err());
    assert!(
        model
            .materialized_pool_object_revision(part, "parts", ObjectRevision(next.0 + 100))
            .is_err()
    );
    // Counterfactual source-evidence controls: deletion/reuse cannot qualify a pin.
    let mut reused = model.clone();
    reused
        .journal
        .iter_mut()
        .find(|t| t.transaction_kind == crate::substrate::TransactionKind::Normal)
        .unwrap()
        .operations
        .push(Operation::DeletePoolLibraryObject {
            object_id: part,
            relative_path: pin.relative_path.clone(),
            object_kind: "parts".into(),
            object: corrected.clone(),
        });
    assert!(pins::check_pin(&reused, &pin).is_err());
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, part), original);
    verify(&model);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, part), corrected);
    verify(&model);
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&cold, part), corrected);
    verify(&cold);
    // Separately triggered PM057 rematerialization must remain available. This
    // is not part of the metadata transaction or automatic reference updating.
    model = cold;
    let mut refresh = request.clone();
    let target_owner = pins::capture_root(
        &model,
        "60000000-0000-4000-8000-000000000001".parse().unwrap(),
        SourceShardTaxon::PoolFootprint,
    )
    .unwrap();
    for value in board["component_pads"][package.to_string()]
        .as_array()
        .unwrap()
    {
        let pad: pins::ComponentPadSource = serde_json::from_value(value.clone()).unwrap();
        refresh.correspondences.push(crate::api::native_write::board_components::pad_materialization::PlacedPadCorrespondence {
            placed_pad_id:pad.uuid,target_library_pad:pins::capture_pad(&model,&target_owner,pad.source_binding.as_ref().unwrap().library_pad.object_id).unwrap()});
    }
    let refresh_tip = model.journal.last().unwrap().transaction_id;
    let refresh_revision = model.objects[&part].object_revision;
    let refresh_hash = crate::substrate::sha256_hex(
        crate::ir::serialization::to_json_deterministic(&corrected)
            .unwrap()
            .as_bytes(),
    );
    let check_refresh = |model: &DesignModel, refreshed: bool| {
        assert_eq!(raw(model, part), corrected);
        assert_eq!(
            projected(model),
            pad_projection,
            "all8 Pad IDs and query geometry remain unchanged across explicit refresh/history"
        );
        let owner = &model.component_instances[&ci];
        assert_eq!(owner.part_ref, Some(part));
        assert!(owner.library_bindings.values().any(|b| b.binding_role
            == crate::substrate::LibraryBindingRole::Part
            && b.target_object_id == part
            && b.pinned_object_revision == revision));
        assert!(
            !model
                .diagnostics
                .iter()
                .any(|d| d.code.starts_with("component_instance_"))
        );
        let current = model
            .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
            .unwrap();
        for value in current["component_pads"][package.to_string()]
            .as_array()
            .unwrap()
        {
            let old = board["component_pads"][package.to_string()]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["uuid"] == value["uuid"])
                .unwrap();
            assert_eq!(
                value["source_binding"]["frozen_geometry"],
                old["source_binding"]["frozen_geometry"]
            );
            for key in [
                "part",
                "package",
                "land_pattern",
                "library_pad",
                "padstack",
                "pin_pad_map",
            ] {
                let mut expected = old["source_binding"][key].clone();
                if refreshed {
                    expected["journal_tip"] = json!(refresh_tip);
                    if key == "part" {
                        expected["object_revision"] = json!(refresh_revision.0);
                        expected["owner_value_hash"] = json!(refresh_hash);
                        expected["record_hash"] = json!(refresh_hash);
                    }
                }
                assert_eq!(
                    value["source_binding"][key], expected,
                    "exact declared source pin {key}, including the known capture tip"
                );
            }
            assert_eq!(
                value["source_binding"]["component_instance"]["object_revision"],
                json!(owner.object_revision.0)
            );
        }
        for (class, id, other) in [("symbols", symbol, package), ("packages", package, symbol)] {
            let at = origin(class, id);
            let result = context(model, at.clone());
            assert!(result.unavailable.is_empty());
            assert_eq!(
                result.bindings[&at].counterpart_status,
                ComponentCounterpartStatus::Resolved
            );
            assert_eq!(
                result.relations.first().unwrap().counterpart.source_id,
                other
            );
        }
    };
    let p = build_materialize_component_pad_sources(&model, provenance(), &refresh).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    check_refresh(&model, true);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    check_refresh(&model, false);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    check_refresh(&model, true);
    check_refresh(&ProjectResolver::new(&root).resolve().unwrap(), true);
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

#[path = "component_relations_initial_materialization_tests.rs"]
mod initial_materialization_tests;
#[path = "component_relations_map_tests.rs"]
mod map_tests;
#[path = "component_relations_physical_fixture.rs"]
mod physical_fixture;
