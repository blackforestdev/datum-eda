//! Real governed native pool data, canonical authoring, qualified Proposal and persisted history.
use super::super::{
    commit_prepared,
    electrical::topology_tests,
    library::{self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec},
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
};
use super::*;
use crate::substrate::{CommitSource, ProjectResolver, ProposalStatus, SourceShardKind};
use std::path::{Path, PathBuf};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "assignment fixture")
}
fn ids() -> (Uuid, Uuid, Uuid, Uuid) {
    (
        "30000000-0000-4000-8000-000000000001".parse().unwrap(),
        "90000000-0000-4000-8000-000000000001".parse().unwrap(),
        "40000000-0000-4000-8000-000000000001".parse().unwrap(),
        "40000000-0000-4000-8000-000000000101".parse().unwrap(),
    )
}
pub(crate) fn fixture(name: &str) -> (PathBuf, DesignModel, Uuid, Uuid) {
    let (root, mut model, _, sheet) = topology_tests::fixture(name);
    let baseline = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-harness/testdata/library/native_authored_baseline_v1/pool");
    let mut objects = vec![];
    for kind in [
        "units",
        "symbols",
        "entities",
        "packages",
        "padstacks",
        "footprints",
        "parts",
        "pin_pad_maps",
    ] {
        for entry in std::fs::read_dir(baseline.join(kind)).unwrap() {
            let object: Value =
                serde_json::from_slice(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap();
            objects.push(Spec::Create {
                target: Target::new("pool", kind, id(&object, "uuid").unwrap()),
                object,
            });
        }
    }
    // A second legitimate native Part uses the ordinary CLI's canonical payload builder.
    let second = Uuid::new_v4();
    let package = "50000000-0000-4000-8000-000000000001".parse().unwrap();
    objects.push(Spec::Create {
        target: Target::new("pool", "parts", second),
        object: library::pool_part_payload(
            second,
            ids().2,
            package,
            "Alternate native op amp",
            "Datum",
            "OPAMP",
            "Assignment choice",
            "",
            "Active",
        ),
    });
    let write =
        library::build_pool_library_write(&model, provenance(), Some("pool"), objects).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let write=(find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(&NativeWriteContext{model:&model,project_root:&root},provenance(),json!({"sheet":sheet,"library_symbol":ids().0,"reference":"U1","value":"OPAMP","x_nm":10_000_000,"y_nm":20_000_000})).unwrap();
    let symbol = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(
        model
            .component_instances
            .values()
            .all(|c| !c.placed_symbol_refs.contains(&symbol))
    );
    let selected = options(&model, symbol, &[]).unwrap();
    let old = super::super::schematic_symbol_payload::source_symbol_payload(
        &model,
        sheet,
        &selected.original,
    )
    .unwrap();
    let mut raw = old.clone();
    raw["extension"] = json!({"retain":"symbol"});
    raw["pins"][0]["extension"] = json!([1, 2, 3]);
    raw["fields"] = json!([{"uuid":Uuid::new_v4(),"key":"User note","value":"keep","visible":false,"position":{"x":77,"y":88,"extension":true},"extension":"field"}]);
    let write = PreparedWrite {
        primary_object_id: Some(symbol),
        batch: super::super::context::build_batch(
            &model,
            provenance(),
            vec![Operation::SetSchematicSymbol {
                sheet_id: sheet,
                symbol_id: symbol,
                symbol: raw,
            }],
        )
        .unwrap(),
    };
    commit_prepared(&mut model, &root, write).unwrap();
    (root, model, symbol, second)
}
fn request(model: &DesignModel, root: &Path, action: Action) -> NativeProposalRequest {
    NativeProposalRequest {
        project_root: root.into(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
        action,
    }
}
fn act(model: &mut DesignModel, root: &Path, action: Action) -> proposal::NativeProposalResponse {
    let req = request(model, root, action);
    proposal::execute(model, &req).unwrap()
}
fn create(symbol: Uuid, part: Uuid) -> Action {
    Action::CreateAssignment {
        proposal_id: Uuid::new_v4(),
        symbol,
        instance_path: vec![],
        part,
        gate: ids().3,
    }
}
fn raw_symbol(model: &DesignModel, symbol: Uuid) -> Value {
    let o = options(model, symbol, &[]).unwrap();
    super::super::schematic_symbol_payload::source_symbol_payload(model, o.sheet, &o.original)
        .unwrap()
}
fn assert_binding(model: &DesignModel, symbol: Uuid, ci: Uuid, part: Uuid) {
    let current = &model.component_instances[&ci];
    assert_eq!(current.part_ref, Some(part));
    assert_eq!(current.placed_symbol_refs, vec![symbol]);
    assert!(current.placed_package_refs.is_empty());
    let shard = model
        .source_shards
        .iter()
        .find(|s| {
            s.kind == SourceShardKind::ComponentInstance
                && s.shard_id == model.objects[&ci].source_shard_id
        })
        .unwrap();
    let value = model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap();
    assert_eq!(
        value["component_instance"]["placed_symbol_refs"][0]["object_revision"],
        json!(model.objects[&symbol].object_revision.0)
    );
    assert!(
        !model
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("component_instance_")),
        "{:?}",
        model.diagnostics
    );
}
#[test]
fn native_assignment_initial_reassignment_raw_identity_and_one_step_history() {
    let (root, mut model, symbol, second) = fixture("assignment-history");
    let original = raw_symbol(&model, symbol);
    let nets = model.electrical_identities.clone();
    let draft = act(&mut model, &root, create(symbol, ids().1));
    let id = draft.proposal_id;
    assert_eq!(draft.status, ProposalStatus::Draft);
    assert_eq!(raw_symbol(&model, symbol), original);
    let summary = draft.assignment.unwrap();
    assert_eq!(summary.previous_part, None);
    let ci = summary.component_instance;
    let applied = act(&mut model, &root, Action::AcceptApply { proposal_id: id });
    assert_eq!(applied.assignment.unwrap(), summary);
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_binding(&model, symbol, ci, ids().1);
    let assigned = raw_symbol(&model, symbol);
    let mut expect = original.clone();
    expect["part"] = json!(ids().1);
    expect["entity"] = json!(ids().2);
    expect["gate"] = json!(ids().3);
    assert_eq!(assigned, expect);
    assert_eq!(model.electrical_identities, nets);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw_symbol(&model, symbol), original);
    assert!(!model.component_instances.contains_key(&ci));
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_binding(&model, symbol, ci, ids().1);
    assert_eq!(raw_symbol(&model, symbol), assigned);
    // Canonical structured provenance/role extension payload survives reassignment and history.
    let (_, previous_ci) = component::current(&model, symbol).unwrap().unwrap();
    let mut extended_ci = previous_ci.clone();
    extended_ci["object_revision"] = json!(model.objects[&ci].object_revision.0 + 1);
    for binding in extended_ci["library_bindings"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        binding["provenance"] = json!({"extension":{"source":"retain verbatim"}});
    }
    extended_ci["placed_symbol_roles"][symbol.to_string()]["label"] = json!("Preserved role");
    let write = PreparedWrite {
        primary_object_id: Some(ci),
        batch: super::super::context::build_batch(
            &model,
            provenance(),
            vec![Operation::SetComponentInstance {
                component_instance_id: ci,
                previous_component_instance: previous_ci,
                component_instance: extended_ci.clone(),
            }],
        )
        .unwrap(),
    };
    commit_prepared(&mut model, &root, write).unwrap();
    let draft = act(&mut model, &root, create(symbol, second));
    let summary = draft.assignment.unwrap();
    assert_eq!(summary.previous_part, Some(ids().1));
    let applied = act(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    assert_eq!(applied.assignment.unwrap(), summary);
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_binding(&model, symbol, ci, second);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_binding(&model, symbol, ci, ids().1);
    assert_eq!(raw_symbol(&model, symbol), assigned);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_binding(&model, symbol, ci, second);
    assert_eq!(model.electrical_identities, nets);
    let (_, preserved) = component::current(&model, symbol).unwrap().unwrap();
    assert_eq!(
        preserved["placed_symbol_roles"],
        extended_ci["placed_symbol_roles"]
    );
    for binding in preserved["library_bindings"].as_object().unwrap().values() {
        assert_eq!(
            binding["provenance"],
            json!({"extension":{"source":"retain verbatim"}})
        );
    }
    assert!(build(&model, provenance(), symbol, &[], second, ids().3).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_assignment_refusals_cancel_stale_and_incompatible_choice_do_not_mutate() {
    let (root, mut model, symbol, second) = fixture("assignment-refusals");
    let original = raw_symbol(&model, symbol);
    let len = model.journal.len();
    for (path, part, gate) in [
        (vec![Uuid::new_v4()], ids().1, ids().3),
        (vec![], Uuid::new_v4(), ids().3),
        (vec![], ids().1, Uuid::new_v4()),
    ] {
        assert!(build(&model, provenance(), symbol, &path, part, gate).is_err());
    }
    assert_eq!(model.journal.len(), len);
    let rejected = act(&mut model, &root, create(symbol, ids().1)).proposal_id;
    act(
        &mut model,
        &root,
        Action::Reject {
            proposal_id: rejected,
        },
    );
    assert_eq!(raw_symbol(&model, symbol), original);
    let stale = act(&mut model, &root, create(symbol, ids().1)).proposal_id;
    let winner = act(&mut model, &root, create(symbol, second)).proposal_id;
    act(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: winner,
        },
    );
    let before = raw_symbol(&model, symbol);
    let len = model.journal.len();
    let req = request(&model, &root, Action::AcceptApply { proposal_id: stale });
    assert!(proposal::execute(&mut model, &req).is_err());
    assert_eq!(model.journal.len(), len);
    assert_eq!(raw_symbol(&model, symbol), before);
    // Recorded semantic payload tampering is never admitted as this native operation.
    model
        .proposals
        .get_mut(&stale)
        .unwrap()
        .batch
        .operations
        .iter_mut()
        .for_each(|op| {
            if let Operation::SetSchematicSymbol { symbol, .. } = op {
                symbol["value"] = json!("unrelated");
            }
        });
    let req = request(&model, &root, Action::Preview { proposal_id: stale });
    assert!(proposal::execute(&mut model, &req).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[path = "schematic_assignment_admission_tests.rs"]
mod admission;
