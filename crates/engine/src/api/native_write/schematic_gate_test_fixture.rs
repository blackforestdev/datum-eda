//! Canonical native library fixture shared by multi-gate lifecycle controls.
use super::*;
pub(crate) fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "native multi-gate lifecycle")
}
pub(crate) fn action(
    model: &mut DesignModel,
    root: &Path,
    action: Action,
) -> proposal::NativeProposalResponse {
    let req = request(model, root, action);
    proposal::execute(model, &req).unwrap()
}
pub(super) fn request(model: &DesignModel, root: &Path, action: Action) -> NativeProposalRequest {
    NativeProposalRequest {
        project_root: root.into(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
        action,
    }
}
pub(crate) fn raw(model: &DesignModel, id: Uuid) -> Value {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&id].source_shard_id)
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()
}
pub(crate) fn setup(name: &str) -> (PathBuf, DesignModel, Uuid, GatePlacementRequest) {
    let (root, mut model, first, part) =
        crate::api::native_write::schematic_assignment::tests::fixture(name);
    let entity: Uuid = "40000000-0000-4000-8000-000000000001".parse().unwrap();
    let first_gate: Uuid = "40000000-0000-4000-8000-000000000101".parse().unwrap();
    let gate = Uuid::new_v4();
    let mut entity_raw = materialized_pool_object(&model, entity, "entities").unwrap();
    let mut added = entity_raw["gates"][first_gate.to_string()].clone();
    added["uuid"] = json!(gate);
    added["name"] = json!("Second gate");
    entity_raw["gates"][gate.to_string()] = added;
    let write = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![Spec::Set {
            target: Target::new("pool", "entities", entity),
            object: entity_raw,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let draft = action(
        &mut model,
        &root,
        Action::CreateAssignment {
            proposal_id: Uuid::new_v4(),
            symbol: first,
            instance_path: vec![],
            part,
            gate: first_gate,
        },
    );
    let ci = draft.assignment.unwrap().component_instance;
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    // Real persisted raw complements exercise the shared repin and history owner.
    let previous = raw(&model, ci)["component_instance"].clone();
    let mut next = previous.clone();
    next["object_revision"] = json!(model.objects[&ci].object_revision.0 + 1);
    for binding in next["library_bindings"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        binding["provenance"] = json!({"extension":{"keep":["owner"]}});
    }
    next["placed_symbol_roles"][first.to_string()]["label"] = json!("Preserved member");
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetComponentInstance {
            component_instance_id: ci,
            previous_component_instance: previous,
            component_instance: next,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let choices = options(&model, first, &[]).unwrap();
    assert_eq!(choices.choices.len(), 1);
    let req = GatePlacementRequest {
        proposal_id: Uuid::new_v4(),
        component_instance: ci,
        sheet: choices.sheet,
        instance_path: vec![],
        gate,
        library_symbol: choices.choices[0].library_symbol,
        reference: "U2".into(),
        value: "second explicit gate".into(),
        x_nm: 50_000_000,
        y_nm: 20_000_000,
        rotation_deg: 0,
        mirrored: false,
    };
    (root, model, first, req)
}
pub(crate) fn binding(model: &DesignModel, id: Uuid) -> Value {
    let projected = &model.component_instances[&id];
    let mut value = raw(model, id)["component_instance"].clone();
    assert_eq!(
        value["object_revision"],
        json!(model.objects[&id].object_revision.0)
    );
    let mut seen = BTreeSet::new();
    for member in value["placed_symbol_refs"].as_array_mut().unwrap() {
        let symbol: Uuid = serde_json::from_value(member["object_id"].clone()).unwrap();
        assert!(seen.insert(symbol));
        assert_eq!(
            member["object_revision"],
            json!(model.objects[&symbol].object_revision.0)
        );
        member["object_revision"] = Value::Null;
    }
    assert_eq!(seen, projected.placed_symbol_refs.iter().copied().collect());
    assert!(
        model
            .diagnostics
            .iter()
            .all(|d| !d.code.starts_with("component_instance_")),
        "{:?}",
        model.diagnostics
    );
    value["object_revision"] = Value::Null;
    value
}
pub(super) fn native(model: &mut DesignModel, root: &Path, verb: &str, params: Value) {
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
pub(super) fn roundtrip(
    model: &mut DesignModel,
    root: &Path,
    ci: Uuid,
    before_sheet: &Value,
    before_binding: &Value,
    after_sheet: &Value,
) {
    let after_binding = binding(model, ci);
    model
        .commit_journal_undo(root, provenance().into())
        .unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(binding(model, ci), *before_binding);
    let symbol = model.component_instances[&ci].placed_symbol_refs[0];
    assert_eq!(raw(model, symbol), *before_sheet);
    model
        .commit_journal_redo(root, provenance().into())
        .unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(binding(model, ci), after_binding);
    let symbol = model.component_instances[&ci].placed_symbol_refs[0];
    assert_eq!(raw(model, symbol), *after_sheet);
}
