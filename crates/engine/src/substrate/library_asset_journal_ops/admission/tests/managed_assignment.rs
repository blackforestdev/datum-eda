//! Real accepted managed assignment and refusal of mixed compensation records.
use super::*;
use crate::substrate::library_asset_journal_ops::{binding_history, history};
use native::{
    schematic_delete_proposal::NativeProposalAction as Action,
    schematic_library::part_placement::CompletePartSelection,
};

#[test]
fn public_managed_assignment_keeps_receipt_and_exact_bindings_through_history() {
    const NAME: &str = concat!(
        module_path!(),
        "::public_managed_assignment_keeps_receipt_and_exact_bindings_through_history"
    );
    if isolated(NAME) {
        return;
    }
    let (root, mut model) =
        native::schematic_library::part_placement::tests::complete_assignment_history_fixture();
    let ci = model
        .component_instances
        .values()
        .find(|ci| ci.authority == ComponentInstanceAuthority::Authored)
        .unwrap()
        .clone();
    let original = ci.part_ref.unwrap();
    let alternate = model
        .objects
        .values()
        .find(|o| o.domain == "pool" && o.kind == "parts" && o.object_id != original)
        .unwrap()
        .object_id;
    let placed = ci.placed_symbol_refs[0];
    let sources = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let symbol = sources
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .find_map(|sheet| sheet.symbols.get(&placed))
        .unwrap();
    let gate = symbol.gate.unwrap();
    let library_symbol = Uuid::parse_str(symbol.lib_id.as_deref().unwrap()).unwrap();
    let first = CompletePartSelection::capture(&model, original, gate, library_symbol).unwrap();
    // Author the second Part's factual map through the ordinary library verb;
    // the reused fixture initially gives only its selected Part a complete map.
    let map_object = &model.objects[&first.pin_pad_map];
    let map = model
        .materialized_pool_object_revision(
            first.pin_pad_map,
            "pin_pad_maps",
            map_object.object_revision,
        )
        .unwrap();
    let prepared = (native::registry::find_native_write_verb("datum.library.create").unwrap().build)(
        &native::registry::NativeWriteContext { model: &model, project_root: &root },
        public_provenance(),
        serde_json::json!({"kind":"pin_pad_maps","object_id":Uuid::new_v4(),"pool_path":"pool",
            "fields":{"part":alternate,"footprint":first.footprint,"mappings":map["mappings"],"assign_default":true}}),
    ).unwrap();
    native::commit_prepared(&mut model, &root, prepared).unwrap();
    let unmanaged = Uuid::new_v4();
    let prepared = (native::registry::find_native_write_verb("datum.library.create")
        .unwrap()
        .build)(
        &native::registry::NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        public_provenance(),
        serde_json::json!({"kind":"units","object_id":unmanaged,"pool_path":"pool",
            "fields":{"name":"Unmanaged control","pins":{}}}),
    )
    .unwrap();
    native::commit_prepared(&mut model, &root, prepared).unwrap();
    fn inventory(path: &Path) -> (usize, usize, usize) {
        if path.is_file() {
            return (std::fs::metadata(path).unwrap().len() as usize, 1, 0);
        }
        std::fs::read_dir(path)
            .unwrap()
            .map(|e| inventory(&e.unwrap().path()))
            .fold((0, 0, 0), |(b, f, d), (nb, nf, nd)| {
                (b + nb, f + nf, d.max(nd + 1))
            })
    }
    let (bytes, files, depth) = inventory(&root);
    let transitions = [
        "format",
        "capture-original",
        "capture-alternate",
        "adopt",
        "propose",
        "accept",
        "apply",
        "undo",
        "redo",
    ];
    let limits = LibraryAssetLimits {
        metadata_bytes: bytes * transitions.len(),
        blob_bytes: bytes * transitions.len(),
        files: files * transitions.len(),
        depth: depth + 1,
        total_bytes: bytes * files * transitions.len(),
    };
    crate::substrate::library_asset_resolver::configure_server_policy(limits).unwrap();
    public_change(
        &root,
        &mut model,
        serde_json::json!({"action":"adopt_profile"}),
    );
    let mut references = Vec::new();
    for part in [original, alternate] {
        let catalog = snapshot(&root, limits).catalog().manifest_ref.clone();
        let summary = public_change(
            &root,
            &mut model,
            serde_json::json!({"action":"save_candidate","request":{
            "expected_catalog":catalog,"change":{"action":"capture_native","asset_id":part}}}),
        );
        references.push(
            serde_json::from_value::<AssetRevisionRef>(summary["candidate"].clone()).unwrap(),
        );
    }
    let catalog = snapshot(&root, limits).catalog().manifest_ref.clone();
    public_change(
        &root,
        &mut model,
        serde_json::json!({"action":"adopt","request":{
        "expected_catalog":catalog,"roots":references}}),
    );
    let adopted = snapshot(&root, limits);
    assert!(
        !adopted
            .lock()
            .assets
            .iter()
            .any(|asset| asset.local_object_id == unmanaged)
    );
    let choice = CompletePartSelection::capture(&model, alternate, gate, library_symbol).unwrap();
    let proposal_id = Uuid::new_v4();
    native::schematic_gate_placement::tests::action(
        &mut model,
        &root,
        Action::CreateAssignment {
            proposal_id,
            symbol: placed,
            instance_path: vec![],
            part: alternate,
            gate,
            component: Some(choice),
        },
    );
    native::schematic_gate_placement::tests::action(
        &mut model,
        &root,
        Action::AcceptApply { proposal_id },
    );
    let assignment = model.journal.last().unwrap().clone();
    assert!(binding_history::assignment_family(&assignment));
    crate::substrate::component_instance::assignment_receipt(&assignment, placed).unwrap();
    let assert_state = |current: &DesignModel, part: Uuid, reference: &AssetRevisionRef| {
        let active = snapshot(&root, limits);
        assert_eq!(active.lock().assets, adopted.lock().assets);
        assert_eq!(active.catalog(), adopted.catalog());
        assert_eq!(active.lock().bindings.len(), 1);
        assert_eq!(active.lock().bindings[0].component_instance_id, ci.id);
        assert_eq!(&active.lock().bindings[0].component, reference);
        assert_eq!(current.component_instances[&ci.id].part_ref, Some(part));
        assert_eq!(
            current.component_instances[&ci.id].placed_symbol_refs,
            vec![placed]
        );
        CompletePartSelection::capture(current, part, gate, library_symbol)
            .unwrap()
            .verify_declared(current, ci.id)
            .unwrap();
    };
    assert_state(&model, alternate, &references[1]);
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_state(&model, alternate, &references[1]);
    model
        .commit_journal_undo(&root, public_provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_state(&model, original, &references[0]);
    assert_mixed_assignment_faults(&model, &assignment, unmanaged);
    model
        .commit_journal_redo(&root, public_provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_state(&model, alternate, &references[1]);
    std::fs::remove_dir_all(root).unwrap();
}

fn assert_mixed_assignment_faults(
    model: &DesignModel,
    assignment: &crate::substrate::TransactionRecord,
    unmanaged: Uuid,
) {
    let object = &model.objects[&unmanaged];
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == object.source_shard_id)
        .unwrap();
    let previous = model
        .materialized_pool_object_revision(unmanaged, "units", object.object_revision)
        .unwrap();
    let mut edited = previous.clone();
    edited["name"] = "Unrelated library edit".into();
    let pool = Operation::SetPoolLibraryObject {
        object_id: unmanaged,
        relative_path: shard.relative_path.clone(),
        object_kind: "units".into(),
        previous_object: previous,
        object: edited,
    };
    let symbol = assignment
        .operations
        .iter()
        .find(|op| matches!(op, Operation::SetSchematicSymbol { .. }))
        .unwrap();
    for (companion, classified) in [(&pool, false), (symbol, true)] {
        // Fault injection into copies of actual published records only. Never
        // publish the altered target or use it as a successful fixture history.
        let mut altered = assignment.clone();
        altered.operations.push(companion.clone());
        let mut inverse = Vec::new();
        if matches!(companion, Operation::SetSchematicSymbol { .. }) {
            inverse.push(
                assignment
                    .inverse_operations
                    .iter()
                    .find(|op| matches!(op, Operation::SetSchematicSymbol { .. }))
                    .unwrap()
                    .clone(),
            );
        } else {
            crate::substrate::journal_operation_hooks::inverse_non_core_operation(
                companion,
                &mut inverse,
            )
            .unwrap();
        }
        altered.inverse_operations.splice(0..0, inverse);
        assert_eq!(binding_history::assignment_family(&altered), classified);
        if !classified {
            assert!(history::requires_owned_compensation(
                &altered,
                &model.journal
            ));
        }
        let mut prior = model.journal[..model.journal.len() - 1].to_vec();
        *prior
            .iter_mut()
            .find(|tx| tx.transaction_id == assignment.transaction_id)
            .unwrap() = altered.clone();
        let mut compensation = model.journal.last().unwrap().clone();
        compensation.operations = altered.inverse_operations;
        compensation.inverse_operations = altered.operations;
        assert!(
            crate::substrate::transaction_links::validate_transaction_links(&compensation, &prior)
                .is_err()
        );
    }
}
