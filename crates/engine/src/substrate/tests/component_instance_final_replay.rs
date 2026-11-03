//! Current reference diagnostics follow accepted journal epoch reconstruction.
use super::*;
use crate::api::native_write::{
    BatchComposer, WriteProvenance, commit_prepared,
    library::{self, PoolLibraryObjectTarget, PoolLibraryOperationSpec},
    schematic_assignment::tests::fixture,
    schematic_delete_proposal::{self as proposal, NativeProposalAction, NativeProposalRequest},
};
#[test]
fn component_instance_final_replay_keeps_current_and_excludes_stale_references() {
    for stale_library in [false, true] {
        let (root, mut model, symbol, part) = fixture("component-final-replay");
        let id = Uuid::new_v4();
        for action in [
            NativeProposalAction::CreateAssignment {
                proposal_id: id,
                symbol,
                instance_path: vec![],
                part,
                gate: "40000000-0000-4000-8000-000000000101".parse().unwrap(),
            },
            NativeProposalAction::AcceptApply { proposal_id: id },
        ] {
            let request = NativeProposalRequest {
                project_root: root.clone(),
                expected_model_revision: model.model_revision.0.clone(),
                expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
                action,
            };
            proposal::execute(&mut model, &request).unwrap();
        }
        model = ProjectResolver::new(&root).resolve().unwrap();
        let ci = model
            .component_instances
            .values()
            .find(|c| c.placed_symbol_refs == vec![symbol])
            .unwrap()
            .id;
        assert!(
            !model
                .diagnostics
                .iter()
                .any(|d| d.code.starts_with("component_instance_")),
            "{:?}",
            model.diagnostics
        );
        let provenance =
            WriteProvenance::new("test", CommitSource::Manual, "advance referenced source");
        let write = if stale_library {
            let object = &model.objects[&part];
            let shard = model
                .source_shards
                .iter()
                .find(|s| s.shard_id == object.source_shard_id)
                .unwrap();
            let mut payload = model
                .materialized_source_shard_value_by_relative_path(&shard.relative_path)
                .unwrap();
            payload["description"] = serde_json::json!("Changed library source revision");
            library::build_pool_library_write(
                &model,
                provenance,
                Some("pool"),
                vec![PoolLibraryOperationSpec::Set {
                    target: PoolLibraryObjectTarget::new("pool", "parts", part),
                    object: payload,
                }],
            )
            .unwrap()
        } else {
            let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
            let sheet = snapshot
                .schematic_source()
                .unwrap()
                .sheets
                .values()
                .find(|s| s.symbols.contains_key(&symbol))
                .unwrap();
            let shard = model
                .source_shards
                .iter()
                .find(|s| s.shard_id == model.objects[&symbol].source_shard_id)
                .unwrap();
            let source = model
                .materialized_source_shard_value_by_relative_path(&shard.relative_path)
                .unwrap();
            let mut payload = source["symbols"][symbol.to_string()].clone();
            payload["extension"] = serde_json::json!("Advanced source without a CI repin");
            BatchComposer::compose(&model, provenance)
                .push_op(Operation::SetSchematicSymbol {
                    sheet_id: sheet.uuid,
                    symbol_id: symbol,
                    symbol: payload,
                })
                .finish()
                .unwrap()
        };
        commit_prepared(&mut model, &root, write).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert!(!model.component_instances.contains_key(&ci));
        let code = if stale_library {
            "component_instance_invalid_part_ref"
        } else {
            "component_instance_unresolved_ref"
        };
        let diagnostics: Vec<_> = model
            .diagnostics
            .iter()
            .filter(|d| d.code == code)
            .collect();
        assert_eq!(diagnostics.len(), 1, "{:?}", model.diagnostics);
        assert!(diagnostics[0].message.contains(&ci.to_string()));
        std::fs::remove_dir_all(root).unwrap();
    }
}
