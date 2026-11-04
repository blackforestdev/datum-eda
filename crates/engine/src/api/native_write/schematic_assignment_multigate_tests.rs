//! Whole-owner reassignment through the existing native Part Proposal lifecycle.
use super::*;
use crate::api::native_write::schematic_gate_placement::{self as gates, tests as fixture};
use crate::substrate::electrical_test_partitions;

pub(crate) fn setup(name: &str) -> (PathBuf, DesignModel, [Uuid; 2], Uuid, Uuid) {
    let (root, mut model, first, req) = fixture::setup(name);
    let second = gates::preview(&model, &req).unwrap().symbol.uuid;
    fixture::action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    fixture::action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    let current = model.component_instances[&req.component_instance]
        .part_ref
        .unwrap();
    let old = materialized_pool_object(&model, current, "parts").unwrap();
    let alternate = Uuid::new_v4();
    let payload = library::pool_part_payload(
        alternate,
        id(&old, "entity").unwrap(),
        id(&old, "package").unwrap(),
        "Compatible alternate",
        "Datum",
        "OPAMP",
        "shared reassignment",
        "",
        "Active",
    );
    let write = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "parts", alternate),
            object: payload,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    (
        root,
        model,
        [first, second],
        req.component_instance,
        alternate,
    )
}
fn draft(
    model: &mut DesignModel,
    root: &Path,
    symbol: Uuid,
    part: Uuid,
) -> proposal::NativeProposalResponse {
    let gate = options(model, symbol, &[]).unwrap().original.gate.unwrap();
    act(
        model,
        root,
        Action::CreateAssignment {
            proposal_id: Uuid::new_v4(),
            symbol,
            instance_path: vec![],
            part,
            gate,
        },
    )
}
fn owner_metadata(model: &DesignModel, ci: Uuid) -> Value {
    let mut raw = fixture::binding(model, ci);
    raw["part_ref"] = Value::Null;
    for binding in raw["library_bindings"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        if binding["binding_role"] == "part" {
            binding["target_object_id"] = Value::Null;
            binding["pinned_object_revision"] = Value::Null;
        }
    }
    raw
}
pub(crate) fn persisted(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    fn visit(dir: &Path, out: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, out);
            } else {
                out.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    visit(root, &mut out);
    out
}

#[test]
fn native_assignment_multigate_cancel_accept_and_repeated_cold_history() {
    let (root, mut model, members, ci, alternate) = setup("assignment-multigate-history");
    let original = model.component_instances[&ci].part_ref.unwrap();
    let before = fixture::raw(&model, members[0]);
    let metadata = owner_metadata(&model, ci);
    let identities = model.electrical_identities.clone();
    let partitions = electrical_test_partitions(&model, &[]).unwrap();
    for member in members {
        let choices = options(&model, member, &[]).unwrap();
        assert_eq!(choices.members.len(), 2);
        assert_eq!(choices.members[0].uuid, member);
        assert!(
            choices
                .choices
                .iter()
                .any(|choice| choice.part == alternate)
        );
        assert!(
            choices
                .choices
                .iter()
                .all(|choice| Some(choice.entity) == choices.original.entity
                    && Some(choice.gate) == choices.original.gate)
        );
    }
    let cancelled = draft(&mut model, &root, members[1], alternate);
    let summary = cancelled.assignment.as_ref().unwrap();
    assert_eq!(summary.symbols, vec![members[1], members[0]]);
    assert_eq!(summary.component_instance, ci);
    act(
        &mut model,
        &root,
        Action::Reject {
            proposal_id: cancelled.proposal_id,
        },
    );
    assert_eq!(fixture::raw(&model, members[0]), before);
    assert_eq!(owner_metadata(&model, ci), metadata);
    let prepared = draft(&mut model, &root, members[1], alternate);
    act(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: prepared.proposal_id,
        },
    );
    model = ProjectResolver::new(&root).resolve().unwrap();
    let mut after = before.clone();
    for member in members {
        after["symbols"][member.to_string()]["part"] = json!(alternate);
    }
    assert_eq!(fixture::raw(&model, members[0]), after);
    assert_eq!(model.component_instances[&ci].part_ref, Some(alternate));
    assert_eq!(owner_metadata(&model, ci), metadata);
    assert_eq!(model.electrical_identities, identities);
    assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
    let bytes = persisted(&root);
    let selected = options(&model, members[0], &[]).unwrap();
    assert!(
        build(
            &model,
            provenance(),
            members[0],
            &[],
            alternate,
            selected.original.gate.unwrap()
        )
        .is_err()
    );
    assert_eq!(persisted(&root), bytes);
    for _ in 0..2 {
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(fixture::raw(&model, members[0]), before);
        assert_eq!(model.component_instances[&ci].part_ref, Some(original));
        assert_eq!(owner_metadata(&model, ci), metadata);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(fixture::raw(&model, members[0]), after);
        assert_eq!(model.component_instances[&ci].part_ref, Some(alternate));
        assert_eq!(owner_metadata(&model, ci), metadata);
        assert_eq!(model.electrical_identities, identities);
        assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_assignment_multigate_refuses_gate_change_stale_and_incomplete_receipt() {
    let (root, mut model, members, _ci, alternate) = setup("assignment-multigate-refusals");
    let selected = options(&model, members[0], &[]).unwrap();
    let sibling_gate = selected.members[1].gate.unwrap();
    let before = persisted(&root);
    assert!(
        build(
            &model,
            provenance(),
            members[0],
            &[],
            alternate,
            sibling_gate
        )
        .is_err()
    );
    // The original baseline Part maps only the first Gate and cannot replace this owner.
    let mapped_part: Uuid = "90000000-0000-4000-8000-000000000001".parse().unwrap();
    assert!(
        !selected
            .choices
            .iter()
            .any(|choice| choice.part == mapped_part)
    );
    assert!(
        build(
            &model,
            provenance(),
            members[0],
            &[],
            mapped_part,
            selected.original.gate.unwrap()
        )
        .is_err()
    );
    assert_eq!(persisted(&root), before);
    let stale = draft(&mut model, &root, members[0], alternate);
    let write = (find_native_write_verb("datum.schematic.set_symbol_field")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"symbol":members[1],"instance_path":[],"field":"value","value":"new sibling value"}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = persisted(&root);
    let request = request(
        &model,
        &root,
        Action::AcceptApply {
            proposal_id: stale.proposal_id,
        },
    );
    assert!(proposal::execute(&mut model, &request).is_err());
    assert_eq!(persisted(&root), before);
    let prepared = draft(&mut model, &root, members[0], alternate);
    model
        .proposals
        .get_mut(&prepared.proposal_id)
        .unwrap()
        .batch
        .operations
        .retain(|op| {
            !matches!(op,
        Operation::SetSchematicSymbol { symbol_id,.. } if *symbol_id==members[1])
        });
    let before = persisted(&root);
    let request = super::request(
        &model,
        &root,
        Action::Preview {
            proposal_id: prepared.proposal_id,
        },
    );
    assert!(proposal::execute(&mut model, &request).is_err());
    assert_eq!(persisted(&root), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_assignment_multigate_refuses_different_entity_with_matching_native_geometry() {
    let (root, mut model, members, _, alternate) = setup("assignment-multigate-entity");
    let selected = options(&model, members[0], &[]).unwrap();
    let other_entity = Uuid::new_v4();
    let other_part = Uuid::new_v4();
    let mut entity =
        materialized_pool_object(&model, selected.original.entity.unwrap(), "entities").unwrap();
    entity["uuid"] = json!(other_entity);
    // Gate identities are globally unique even when native geometry is shared.
    let gates = entity["gates"].as_object_mut().unwrap();
    let originals = std::mem::take(gates);
    for (_, mut gate) in originals {
        let id = Uuid::new_v4();
        gate["uuid"] = json!(id);
        gates.insert(id.to_string(), gate);
    }
    let mut part = materialized_pool_object(&model, alternate, "parts").unwrap();
    part["uuid"] = json!(other_part);
    part["entity"] = json!(other_entity);
    let write = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![
            Spec::Create {
                target: Target::new("pool", "entities", other_entity),
                object: entity,
            },
            Spec::Create {
                target: Target::new("pool", "parts", other_part),
                object: part,
            },
        ],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = persisted(&root);
    for member in members {
        let selected = options(&model, member, &[]).unwrap();
        assert!(
            !selected
                .choices
                .iter()
                .any(|choice| choice.part == other_part)
        );
        assert!(
            build(
                &model,
                provenance(),
                member,
                &[],
                other_part,
                selected.original.gate.unwrap()
            )
            .is_err()
        );
    }
    assert_eq!(persisted(&root), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_assignment_multigate_locked_sibling_refuses_before_mutation() {
    let (root, mut model, members, _ci, alternate) = setup("assignment-multigate-locked");
    let selected = options(&model, members[1], &[]).unwrap();
    let original = selected.original;
    let mut raw = raw_symbol(&model, members[1]);
    raw["locked"] = json!(true);
    let mut operations = vec![Operation::SetSchematicSymbol {
        sheet_id: selected.sheet,
        symbol_id: members[1],
        symbol: raw,
    }];
    operations.extend(component::repin_symbols(&model, std::slice::from_ref(&original)).unwrap());
    let write = PreparedWrite {
        primary_object_id: Some(members[1]),
        batch: super::super::super::context::build_batch(&model, provenance(), operations).unwrap(),
    };
    commit_prepared(&mut model, &root, write).unwrap();
    let before = persisted(&root);
    assert!(
        options(&model, members[0], &[])
            .unwrap_err()
            .to_string()
            .contains("locked")
    );
    assert!(build(&model, provenance(), members[0], &[], alternate, ids().3).is_err());
    assert_eq!(persisted(&root), before);
    std::fs::remove_dir_all(root).unwrap();
}
