//! Ordinary managed-Part placement must keep the portable lock and native
//! declarations together through publication, cold resolve and compensation.
use super::*;
use native::schematic_library::part_placement::CompletePartSelection;

#[test]
fn public_managed_part_placement_keeps_exact_bindings_through_cold_and_history() {
    const NAME: &str = concat!(
        module_path!(),
        "::public_managed_part_placement_keeps_exact_bindings_through_cold_and_history"
    );
    if isolated(NAME) {
        return;
    }
    let (root, mut model) =
        native::schematic_library::part_placement::tests::complete_assignment_history_fixture();
    let original = model
        .component_instances
        .values()
        .find(|ci| ci.authority == ComponentInstanceAuthority::Authored)
        .unwrap()
        .clone();
    let part = original.part_ref.unwrap();
    let placed = original.placed_symbol_refs[0];
    let sources = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let (sheet_id, symbol) = sources
        .schematic_source()
        .unwrap()
        .sheets
        .iter()
        .find_map(|(id, sheet)| sheet.symbols.get(&placed).map(|symbol| (*id, symbol)))
        .unwrap();
    let gate = symbol.gate.unwrap();
    let library_symbol = Uuid::parse_str(symbol.lib_id.as_deref().unwrap()).unwrap();
    let position = symbol.position;
    fn inventory(path: &Path) -> (usize, usize, usize) {
        if path.is_file() {
            return (std::fs::metadata(path).unwrap().len() as usize, 1, 0);
        }
        std::fs::read_dir(path)
            .unwrap()
            .map(|entry| inventory(&entry.unwrap().path()))
            .fold((0, 0, 0), |(bytes, files, depth), (b, f, d)| {
                (bytes + b, files + f, depth.max(d + 1))
            })
    }
    let (bytes, files, depth) = inventory(&root);
    // Test-only policy derives from this actual fixture and the transitions
    // exercised below; it supplies no production default or wire override.
    let transitions = ["format", "capture", "adopt", "place", "undo", "redo"];
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
    let catalog = snapshot(&root, limits).catalog().manifest_ref.clone();
    let captured = public_change(
        &root,
        &mut model,
        serde_json::json!({"action":"save_candidate","request":{
            "expected_catalog":catalog,"change":{"action":"capture_native","asset_id":part}
        }}),
    );
    let reference: AssetRevisionRef =
        serde_json::from_value(captured["candidate"].clone()).unwrap();
    adopt(&root, &mut model, limits, &reference);
    let before = snapshot(&root, limits);
    assert_eq!(before.lock().bindings.len(), 1);
    let original_binding = before.lock().bindings[0].clone();
    assert_eq!(original_binding.component_instance_id, original.id);
    let selected = CompletePartSelection::capture(&model, part, gate, library_symbol).unwrap();
    selected.verify_declared(&model, original.id).unwrap();
    let prepared = (native::registry::find_native_write_verb("datum.schematic.place_symbol")
        .unwrap()
        .build)(
        &native::registry::NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        public_provenance(),
        serde_json::json!({
            "sheet":sheet_id,"instance_path":[],"library_symbol":library_symbol,
            "component":selected,"reference":"U2","value":selected.value,
            "x_nm":position.x + 10000000,"y_nm":position.y,
            "rotation_deg":0,"mirrored":false
        }),
    )
    .unwrap();
    let new_symbol = prepared.primary_object_id.unwrap();
    // Preparation is build-only: the project still has exactly its original
    // binding. The normal facade must publish both changes in one transaction.
    assert_eq!(
        snapshot(&root, limits).lock().bindings,
        before.lock().bindings
    );
    native::commit_prepared(&mut model, &root, prepared)
        .expect("ordinary managed-Part placement must publish its exact portable binding");
    assert_binding_history_faults(&model, limits);
    let added = model
        .component_instances
        .values()
        .find(|ci| ci.placed_symbol_refs.contains(&new_symbol))
        .unwrap()
        .id;
    assert_ne!(added, original.id);
    let assert_state = |current: &DesignModel, has_added: bool| {
        let active = snapshot(&root, limits);
        let mut expected = vec![original_binding.clone()];
        if has_added {
            let mut second = original_binding.clone();
            second.component_instance_id = added;
            expected.push(second);
        }
        expected.sort_by(|a, b| {
            (&a.source_path, &a.instance_path, a.component_instance_id).cmp(&(
                &b.source_path,
                &b.instance_path,
                b.component_instance_id,
            ))
        });
        assert_eq!(active.lock().bindings, expected);
        assert_eq!(active.lock().assets, before.lock().assets);
        assert_eq!(active.catalog(), before.catalog());
        assert_eq!(current.component_instances[&original.id], original);
        assert_eq!(current.objects.contains_key(&new_symbol), has_added);
        assert_eq!(current.component_instances.contains_key(&added), has_added);
        let choice = CompletePartSelection::capture(current, part, gate, library_symbol).unwrap();
        assert_eq!(choice.footprint, selected.footprint);
        assert_eq!(choice.pin_pad_map, selected.pin_pad_map);
        choice.verify_declared(current, original.id).unwrap();
        if has_added {
            choice.verify_declared(current, added).unwrap();
        }
    };
    assert_state(&model, true);
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_state(&model, true);
    model
        .commit_journal_undo(&root, public_provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_state(&model, false);
    model
        .commit_journal_redo(&root, public_provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_state(&model, true);
    std::fs::remove_dir_all(root).unwrap();
}

fn assert_binding_history_faults(model: &DesignModel, limits: LibraryAssetLimits) {
    use crate::substrate::library_asset_journal_ops::binding_history::exact_native_family;
    let target = model.journal.last().unwrap();
    assert!(exact_native_family(target, &model.journal));
    let mut wrong_source = target.clone();
    let state = wrong_source
        .operations
        .iter_mut()
        .find_map(|op| match op {
            Operation::LibraryAssets {
                operation: LibraryAssetOperation::SetState { state, .. },
            } => Some(state),
            _ => None,
        })
        .unwrap();
    let (mut lock, _) = state.decode(limits.metadata_bytes).unwrap();
    lock.bindings.last_mut().unwrap().source_path = "schematic/wrong.json".into();
    state.lock = crate::library_assets::encode_metadata(&lock).unwrap();
    assert!(!exact_native_family(&wrong_source, &model.journal));
    let mut mixed_pool = target.clone();
    mixed_pool.operations.push(
        model
            .journal
            .iter()
            .flat_map(|tx| &tx.operations)
            .find(|op| matches!(op, Operation::CreatePoolLibraryObject { .. }))
            .unwrap()
            .clone(),
    );
    assert!(!exact_native_family(&mixed_pool, &model.journal));
    let mut forged_inverse = target.clone();
    forged_inverse
        .inverse_operations
        .retain(|op| !matches!(op, Operation::LibraryAssets { .. }));
    assert!(!exact_native_family(&forged_inverse, &model.journal));
}
