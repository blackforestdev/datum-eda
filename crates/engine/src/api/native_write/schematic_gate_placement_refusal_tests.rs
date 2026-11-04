//! Whole-owner qualification and stale/immutable Proposal controls.
use super::tests::*;
use super::*;
use crate::api::native_write::{
    commit_prepared,
    context::BatchComposer,
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
};
#[test]
fn native_gate_placement_refuses_used_foreign_stale_and_inconsistent_members() {
    let (root, mut model, first, req) = setup("gate-refusal");
    let before = raw(&model, first);
    let ci = binding(&model, req.component_instance);
    let count = model.journal.len();
    let mut bad = req.clone();
    bad.reference = String::new();
    assert!(preview(&model, &bad).is_err());
    let mut bad = req.clone();
    bad.instance_path = vec![Uuid::new_v4()];
    assert!(preview(&model, &bad).is_err());
    bad = req.clone();
    bad.component_instance = Uuid::new_v4();
    assert!(preview(&model, &bad).is_err());
    bad = req.clone();
    bad.reference = "U1".into();
    assert!(preview(&model, &bad).is_err());
    bad = req.clone();
    bad.rotation_deg = 37;
    assert!(preview(&model, &bad).is_err());
    bad = req.clone();
    bad.gate = "40000000-0000-4000-8000-000000000101".parse().unwrap();
    assert!(preview(&model, &bad).is_err());
    bad = req.clone();
    bad.library_symbol = Uuid::new_v4();
    assert!(preview(&model, &bad).is_err());
    assert_eq!(model.journal.len(), count);
    assert_eq!(raw(&model, first), before);
    assert_eq!(binding(&model, req.component_instance), ci);
    let second = preview(&model, &req).unwrap().symbol.uuid;
    action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    // A legitimate field edit changes the design while the proposal is open.
    let write = (crate::api::native_write::registry::find_native_write_verb(
        "datum.schematic.set_symbol_value",
    )
    .unwrap()
    .build)(
        &crate::api::native_write::registry::NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"symbol":first,"instance_path":[],"value":"changed while reviewing"}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let accept = NativeProposalRequest {
        project_root: root.clone(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
        action: Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    };
    assert!(proposal::execute(&mut model, &accept).is_err());
    assert!(!model.objects.contains_key(&second));
    action(
        &mut model,
        &root,
        Action::Reject {
            proposal_id: req.proposal_id,
        },
    );
    let mut req = req;
    req.proposal_id = Uuid::new_v4();
    let second = preview(&model, &req).unwrap().symbol.uuid;
    action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    assert!(options(&model, first, &[]).unwrap().choices.is_empty());
    // Mutate a sibling through canonical low-level operations to establish a
    // resolved yet unsupported duplicated Gate. Ordinary writes must refuse it.
    let sheet = raw(&model, first);
    let mut altered = sheet["symbols"][second.to_string()].clone();
    altered["gate"] = sheet["symbols"][first.to_string()]["gate"].clone();
    let original =
        serde_json::from_value::<PlacedSymbol>(sheet["symbols"][second.to_string()].clone())
            .unwrap();
    let pins = super::super::schematic_assignment::repin_symbols(&model, &[original]).unwrap();
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetSchematicSymbol {
            sheet_id: req.sheet,
            symbol_id: second,
            symbol: altered,
        })
        .push_ops(pins)
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(options(&model, first, &[]).is_err());
    assert!(
        super::super::schematic_delete_proposal::bound_symbol_owner(&model, first, &[]).is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_gate_placement_unique_child_occurrence_and_repeated_source_refusal() {
    use crate::api::native_write::schematic_sheets::{
        build_create_schematic_definition, build_create_schematic_sheet_instance,
    };
    use crate::schematic::{SheetDefinition, SheetInstance};
    let (root, mut model, first, mut req) = setup("gate-occurrence");
    let schematic: Uuid = serde_json::from_value(
        model
            .materialized_source_shard_value(crate::substrate::SourceShardKind::SchematicRoot)
            .unwrap()["uuid"]
            .clone(),
    )
    .unwrap();
    let definition = SheetDefinition {
        uuid: Uuid::new_v4(),
        root_sheet: req.sheet,
        name: "Unique gate source".into(),
    };
    let write = build_create_schematic_definition(
        &model,
        provenance(),
        schematic,
        definition.uuid,
        "definitions/gate-source.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let first_instance = Uuid::new_v4();
    let instance = |uuid| SheetInstance {
        uuid,
        definition: definition.uuid,
        parent_sheet: None,
        position: Point::new(0, 0),
        name: "Gate source instance".into(),
        ports: vec![],
    };
    let write = build_create_schematic_sheet_instance(
        &model,
        provenance(),
        schematic,
        first_instance,
        serde_json::to_value(instance(first_instance)).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    req.instance_path = vec![first_instance];
    let prepared = preview(&model, &req).unwrap();
    assert_eq!(prepared.summary.instance_path, req.instance_path);
    let created = action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    assert_eq!(
        created.gate_placement.unwrap().instance_path,
        req.instance_path
    );
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    binding(&model, req.component_instance);
    let write = (crate::api::native_write::registry::find_native_write_verb(
        "datum.schematic.set_symbol_value",
    )
    .unwrap()
    .build)(
        &crate::api::native_write::registry::NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"symbol":first,"instance_path":req.instance_path,"value":"qualified child"}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    binding(&model, req.component_instance);
    let second_instance = Uuid::new_v4();
    let write = build_create_schematic_sheet_instance(
        &model,
        provenance(),
        schematic,
        second_instance,
        serde_json::to_value(instance(second_instance)).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = raw(&model, first);
    let count = model.journal.len();
    assert!(options(&model, first, &req.instance_path).is_err());
    assert!(
        crate::api::native_write::schematic_delete_proposal::bound_symbol_owner(
            &model,
            first,
            &req.instance_path
        )
        .is_err()
    );
    assert_eq!(model.journal.len(), count);
    assert_eq!(raw(&model, first), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_gate_placement_refuses_an_unselected_sibling_owned_by_another_component() {
    let (root, mut model, first, req) = setup("gate-ambiguous-sibling");
    let second = preview(&model, &req).unwrap().symbol.uuid;
    action(&mut model, &root, Action::CreateGatePlacement(req.clone()));
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: req.proposal_id,
        },
    );
    let duplicate = Uuid::new_v4();
    let mut owner = raw(&model, req.component_instance)["component_instance"].clone();
    owner["uuid"] = json!(duplicate);
    owner["object_revision"] = json!(0);
    owner["placed_symbol_refs"] =
        json!([{"object_id":second,"object_revision":model.objects[&second].object_revision.0}]);
    owner["placed_symbol_roles"] = json!({second.to_string():{"role":"gate"}});
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::CreateComponentInstance {
            component_instance_id: duplicate,
            component_instance: owner,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = raw(&model, first);
    let count = model.journal.len();
    assert!(options(&model, first, &[]).is_err());
    assert!(
        crate::api::native_write::schematic_delete_proposal::bound_symbol_owner(&model, first, &[])
            .is_err()
    );
    assert_eq!(raw(&model, first), before);
    assert_eq!(model.journal.len(), count);
    std::fs::remove_dir_all(root).unwrap();
}
