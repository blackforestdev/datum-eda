//! Assigned native library Symbols retain identity through existing property writers.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    registry::find_native_write_verb,
    schematic_assignment::tests::fixture,
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver, electrical_test_partitions};
use serde_json::{Value, json};
use std::path::Path;
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "bound Symbol fields")
}
fn assign(model: &mut DesignModel, root: &Path, symbol: Uuid, part: Uuid) -> Uuid {
    let proposal_id = Uuid::new_v4();
    let mut component = None;
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
            project_root: root.into(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        };
        component = proposal::execute(model, &request)
            .unwrap()
            .assignment
            .map(|s| s.component_instance);
    }
    component.unwrap()
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
fn source(model: &DesignModel, symbol: Uuid) -> Value {
    raw(model, symbol)["symbols"][symbol.to_string()].clone()
}
fn binding(model: &DesignModel, symbol: Uuid, id: Uuid, part: Uuid) -> Value {
    let resolved = &model.component_instances[&id];
    assert_eq!(resolved.part_ref, Some(part));
    assert_eq!(resolved.placed_symbol_refs, vec![symbol]);
    let mut value = raw(model, id)["component_instance"].clone();
    assert_eq!(
        value["placed_symbol_refs"][0]["object_revision"],
        json!(model.objects[&symbol].object_revision.0)
    );
    value.as_object_mut().unwrap().remove("object_revision");
    value["placed_symbol_refs"][0]
        .as_object_mut()
        .unwrap()
        .remove("object_revision");
    value
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
fn roundtrip(
    model: &mut DesignModel,
    root: &Path,
    symbol: Uuid,
    ci: Uuid,
    part: Uuid,
    verb: &str,
    params: Value,
) -> Value {
    let before = source(model, symbol);
    let original_binding = binding(model, symbol, ci, part);
    let identities = model.electrical_identities.clone();
    let partitions = electrical_test_partitions(model, &[]).unwrap();
    let len = model.journal.len();
    let write = build(model, root, verb, params).unwrap();
    assert_eq!(
        source(model, symbol),
        before,
        "form prepare/cancel has no source write"
    );
    assert_eq!(model.journal.len(), len);
    commit_prepared(model, root, write).unwrap();
    assert_eq!(model.journal.len(), len + 1);
    *model = ProjectResolver::new(root).resolve().unwrap();
    let after = source(model, symbol);
    assert_eq!(binding(model, symbol, ci, part), original_binding);
    assert_eq!(after["pins"], before["pins"]);
    assert_eq!(after["position"], before["position"]);
    assert_eq!(after["extension"], before["extension"]);
    assert_eq!(model.electrical_identities, identities);
    assert_eq!(electrical_test_partitions(model, &[]).unwrap(), partitions);
    model
        .commit_journal_undo(root, provenance().into())
        .unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(source(model, symbol), before);
    assert_eq!(binding(model, symbol, ci, part), original_binding);
    model
        .commit_journal_redo(root, provenance().into())
        .unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(source(model, symbol), after);
    assert_eq!(binding(model, symbol, ci, part), original_binding);
    assert_eq!(electrical_test_partitions(model, &[]).unwrap(), partitions);
    after
}
#[test]
fn native_bound_fields_properties_custom_presentation_and_history() {
    let (root, mut model, symbol, part) = fixture("bound-fields-history");
    let ci = assign(&mut model, &root, symbol, part);
    let first = source(&model, symbol)["fields"][0]["uuid"].clone();
    roundtrip(
        &mut model,
        &root,
        symbol,
        ci,
        part,
        "datum.schematic.set_symbol_reference",
        json!({"symbol":symbol,"instance_path":[],"reference":"U9","value":"literal value"}),
    );
    roundtrip(
        &mut model,
        &root,
        symbol,
        ci,
        part,
        "datum.schematic.set_symbol_value",
        json!({"symbol":symbol,"instance_path":[],"value":"updated value"}),
    );
    roundtrip(
        &mut model,
        &root,
        symbol,
        ci,
        part,
        "datum.schematic.set_symbol_field",
        json!({"symbol":symbol,"instance_path":[],"field":"custom","field_id":first,"key":"Duplicate","value":"visible", "visible":true,"position":{"x":1_270_000,"y":-1_270_000}}),
    );
    let with_new = roundtrip(
        &mut model,
        &root,
        symbol,
        ci,
        part,
        "datum.schematic.set_symbol_field",
        json!({"symbol":symbol,"instance_path":[],"field":"custom","key":"Duplicate","value":"second"}),
    );
    let second = with_new["fields"][1]["uuid"].clone();
    roundtrip(
        &mut model,
        &root,
        symbol,
        ci,
        part,
        "datum.schematic.set_symbol_field",
        json!({"symbol":symbol,"instance_path":[],"field":"custom","field_id":second,"visible":true,"position":{"x":2_540_000,"y":0}}),
    );
    roundtrip(
        &mut model,
        &root,
        symbol,
        ci,
        part,
        "datum.schematic.set_symbol_field",
        json!({"symbol":symbol,"instance_path":[],"field":"custom","field_id":second,"visible":false}),
    );
    roundtrip(
        &mut model,
        &root,
        symbol,
        ci,
        part,
        "datum.schematic.set_symbol_field",
        json!({"symbol":symbol,"instance_path":[],"field":"custom","field_id":first,"clear":true}),
    );
    let before = source(&model, symbol);
    let len = model.journal.len();
    for (verb, params) in [
        (
            "datum.schematic.set_symbol_reference",
            json!({"symbol":symbol,"instance_path":[],"reference":"U9"}),
        ),
        (
            "datum.schematic.set_symbol_field",
            json!({"symbol":symbol,"instance_path":[],"field":"custom","field_id":second,"visible":false}),
        ),
        (
            "datum.schematic.set_symbol_field",
            json!({"symbol":symbol,"instance_path":[],"field":"custom","field_id":first,"clear":true}),
        ),
        (
            "datum.schematic.set_symbol_value",
            json!({"symbol":symbol,"instance_path":[Uuid::new_v4()],"value":"wrong occurrence"}),
        ),
    ] {
        assert!(build(&model, &root, verb, params).is_err());
    }
    assert_eq!(source(&model, symbol), before);
    assert_eq!(model.journal.len(), len);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_bound_fields_reject_stale_and_physical_ownership_without_partial_write() {
    use crate::api::native_write::board_components::{
        BoardPackagePlacement, build_place_board_package,
    };
    use crate::{board::PlacedPackage, ir::geometry::Point};
    for physical in [false, true] {
        let (root, mut model, symbol, part) = fixture("bound-fields-refusal");
        let ci = assign(&mut model, &root, symbol, part);
        let write = if physical {
            let placement = BoardPackagePlacement {
                package: PlacedPackage {
                    uuid: Uuid::new_v4(),
                    part,
                    package: "50000000-0000-4000-8000-000000000001".parse().unwrap(),
                    reference: "U1".into(),
                    value: "OPAMP".into(),
                    position: Point::zero(),
                    rotation: 0,
                    layer: 1,
                    locked: false,
                },
                materialized: json!({}),
            };
            let write = build_place_board_package(&model, provenance(), &placement).unwrap();
            commit_prepared(&mut model, &root, write).unwrap();
            let previous = raw(&model, ci)["component_instance"].clone();
            let mut next = previous.clone();
            next["object_revision"] = json!(model.objects[&ci].object_revision.0 + 1);
            next["placed_package_refs"] = json!([{"object_id":placement.package.uuid,"object_revision":model.objects[&placement.package.uuid].object_revision.0}]);
            BatchComposer::compose(&model, provenance())
                .push_op(Operation::SetComponentInstance {
                    component_instance_id: ci,
                    previous_component_instance: previous,
                    component_instance: next,
                })
                .finish()
                .unwrap()
        } else {
            let sheet: Uuid = serde_json::from_value(raw(&model, symbol)["uuid"].clone()).unwrap();
            let mut payload = source(&model, symbol);
            payload["extension"] = json!("source advanced");
            BatchComposer::compose(&model, provenance())
                .push_op(Operation::SetSchematicSymbol {
                    sheet_id: sheet,
                    symbol_id: symbol,
                    symbol: payload,
                })
                .finish()
                .unwrap()
        };
        commit_prepared(&mut model, &root, write).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        let before = source(&model, symbol);
        let len = model.journal.len();
        for (verb, params) in [
            (
                "datum.schematic.set_symbol_reference",
                json!({"symbol":symbol,"instance_path":[],"reference":"U9"}),
            ),
            (
                "datum.schematic.set_symbol_field",
                json!({"symbol":symbol,"instance_path":[],"field":"custom","key":"new","value":"value"}),
            ),
        ] {
            assert!(build(&model, &root, verb, params).is_err());
        }
        assert_eq!(source(&model, symbol), before);
        assert_eq!(model.journal.len(), len);
        std::fs::remove_dir_all(root).unwrap();
    }
}
