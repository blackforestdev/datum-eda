use super::*;
use crate::api::native_write::BatchComposer;
use serde_json::{Value, json};

fn raw_symbol(model: &DesignModel, id: Uuid) -> Value {
    let shard = model
        .source_shards
        .iter()
        .find(|shard| shard.shard_id == model.objects[&id].source_shard_id)
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()["symbols"][id.to_string()]
    .clone()
}

#[test]
fn selected_symbol_property_edits_preserve_raw_extensions_and_exact_history() {
    let (root, mut model, _, sheet, symbol) = fixture("selected_symbol_property_payload");
    let mut expected = raw_symbol(&model, symbol.uuid);
    expected["symbol_extension"] = json!({"schema":9,"items":["opaque",false]});
    expected["position"]["point_extension"] = json!("origin");
    expected["pins"][0]["pin_extension"] = json!({"style":"future"});
    expected["pins"][0]["position"]["point_extension"] = json!("pin");
    expected["fields"][0]["field_extension"] = json!([1,{"alignment":"future"}]);
    expected["fields"][0]["position"]["point_extension"] = json!("field");
    let seed = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetSchematicSymbol {
            sheet_id: sheet,
            symbol_id: symbol.uuid,
            symbol: expected.clone(),
        })
        .primary_object(symbol.uuid)
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, seed).unwrap();
    let identities = model.electrical_identities.clone();
    for (verb, changes) in [
        (
            "datum.schematic.set_symbol_reference",
            json!({"reference":"R2","value":"4k7"}),
        ),
        ("datum.schematic.set_symbol_value", json!({"value":""})),
        (
            "datum.schematic.set_symbol_reference",
            json!({"reference":"R3"}),
        ),
    ] {
        let before = raw_symbol(&model, symbol.uuid);
        let persisted = persisted_bytes(&root);
        let mut params = json!({"symbol":symbol.uuid,"instance_path":[]});
        params
            .as_object_mut()
            .unwrap()
            .extend(changes.as_object().unwrap().clone());
        let prepared = write(&model, &root, verb, params.clone()).unwrap();
        assert_eq!(
            persisted_bytes(&root),
            persisted,
            "prepare/cancel writes nothing"
        );
        let updates: Vec<_> = prepared
            .batch
            .operations
            .iter()
            .filter_map(|op| match op {
                Operation::SetSchematicSymbol { symbol, .. } => Some(symbol),
                _ => None,
            })
            .collect();
        for (key, value) in changes.as_object().unwrap() {
            expected[key] = value.clone();
        }
        assert_eq!(
            updates,
            vec![&expected],
            "only requested scalar properties change"
        );
        commit_prepared(&mut model, &root, prepared).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw_symbol(&model, symbol.uuid), expected);
        assert_eq!(model.electrical_identities, identities);
        let persisted = persisted_bytes(&root);
        assert!(
            write(&model, &root, verb, params)
                .unwrap_err()
                .to_string()
                .contains("unchanged")
        );
        assert_eq!(
            persisted_bytes(&root),
            persisted,
            "no-op refuses without writes"
        );
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(
            raw_symbol(&model, symbol.uuid),
            before,
            "one undo restores exact raw payload"
        );
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw_symbol(&model, symbol.uuid), expected);
        assert_eq!(model.electrical_identities, identities);
    }
    std::fs::remove_dir_all(root).unwrap();
}
