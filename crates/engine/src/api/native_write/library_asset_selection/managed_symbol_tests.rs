//! Managed exact shape adoption through actual sparse placements and history.
use super::*;

#[test]
fn public_managed_shared_symbol_geometry_uses_exact_adoption_and_cold_history() {
    const NAME: &str = concat!(
        module_path!(),
        "::public_managed_shared_symbol_geometry_uses_exact_adoption_and_cold_history"
    );
    if isolated(NAME) {
        return;
    }
    let (root, mut model) =
        native::schematic_library::part_placement::tests::complete_assignment_history_fixture();
    let first_ci = model
        .component_instances
        .values()
        .find(|ci| ci.authority == ComponentInstanceAuthority::Authored)
        .unwrap()
        .clone();
    let first = first_ci.placed_symbol_refs[0];
    let sources = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let (sheet, placed) = sources
        .schematic_source()
        .unwrap()
        .sheets
        .iter()
        .find_map(|(id, sheet)| sheet.symbols.get(&first).map(|p| (*id, p.clone())))
        .unwrap();
    let symbol = Uuid::parse_str(placed.lib_id.as_deref().unwrap()).unwrap();
    let part = first_ci.part_ref.unwrap();
    let choice = native::schematic_library::part_placement::CompletePartSelection::capture(
        &model,
        part,
        placed.gate.unwrap(),
        symbol,
    )
    .unwrap();
    let prepared = (native::registry::find_native_write_verb("datum.schematic.place_symbol")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"sheet":sheet,"library_symbol":symbol,"component":choice,
        "reference":"U99","value":choice.value,"x_nm":10000000,"y_nm":0}),
    )
    .unwrap();
    let second = prepared.primary_object_id.unwrap();
    native::commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(
        model
            .component_instances
            .values()
            .filter(|ci| ci.authority == ComponentInstanceAuthority::Authored)
            .all(|ci| ci
                .library_bindings
                .values()
                .all(|b| b.binding_role != crate::substrate::LibraryBindingRole::Symbol))
    );
    let unrelated = Uuid::new_v4();
    let original_payload = native::library::pool_library_object_payload(
        &model,
        &format!("pool/symbols/{symbol}.json"),
        symbol,
    )
    .unwrap();
    let prepared = (native::registry::find_native_write_verb("datum.library.create").unwrap().build)(
        &NativeWriteContext { model:&model,project_root:&root },provenance(),json!({
            "kind":"symbols","object_id":unrelated,"pool_path":"pool","fields":{
            "name":"Unrelated body","unit":original_payload["unit"],
            "drawings":original_payload["drawings"],"pin_anchors":original_payload["pin_anchors"]}})).unwrap();
    native::commit_prepared(&mut model, &root, prepared).unwrap();
    let old = model
        .materialized_placed_symbol_definition(first, symbol)
        .unwrap()
        .unwrap();
    let path = format!("pool/symbols/{symbol}.json");
    let mut changed = native::library::pool_library_object_payload(&model, &path, symbol).unwrap();
    changed["drawings"][0]["Rect"]["max"]["x"] = json!(3000000);
    let prepared = native::library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![native::library::PoolLibraryOperationSpec::Set {
            target: native::library::PoolLibraryObjectTarget::new("pool", "symbols", symbol),
            object: changed,
        }],
    )
    .unwrap();
    native::commit_prepared(&mut model, &root, prepared).unwrap();
    assert_eq!(
        model
            .materialized_placed_symbol_definition(first, symbol)
            .unwrap()
            .unwrap()["drawings"],
        old["drawings"]
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    let (bytes, files, depth) = inventory(&root);
    let transitions = [
        "format",
        "baseline",
        "adopt",
        "independent",
        "independent-adopt",
        "pending-independent",
        "shape",
        "adopt-shape",
        "undo",
        "redo",
        "cold",
        "older",
    ];
    let limits = LibraryAssetLimits {
        metadata_bytes: bytes * transitions.len(),
        blob_bytes: bytes * transitions.len(),
        files: files * transitions.len(),
        depth: depth + transitions.len(),
        total_bytes: bytes * files * transitions.len(),
    };
    crate::substrate::library_asset_resolver::configure_server_policy(limits).unwrap();
    let mut f = Fixture {
        root,
        model,
        limits,
    };
    f.change(json!({"action":"adopt_profile"}));
    let base = f.capture(part);
    f.adopt(&base);
    let before = f.active(symbol);
    let prior_symbol_revision = f.model.objects[&symbol].object_revision;
    let unrelated_base = f.capture(unrelated);
    f.adopt(&unrelated_base);
    let unrelated_target = f.revise(&unrelated_base, "Independent pending Symbol");
    let sheet_path = f
        .model
        .source_shards
        .iter()
        .find(|s| s.shard_id == f.model.objects[&first].source_shard_id)
        .unwrap()
        .relative_path
        .clone();
    let sheet_bytes = std::fs::read(f.root.join(&sheet_path)).unwrap();
    let mut body = f.snapshot().assets()[&before].payload().value().clone();
    body["drawings"][0]["Rect"]["max"]["x"] = json!(3500000);
    body["drawings"]
        .as_array_mut()
        .unwrap()
        .push(json!({"Line":{
        "from":{"x":0,"y":0},"to":{"x":4500000,"y":2000000},"width":100000}}));
    let target: AssetRevisionRef = serde_json::from_value(f.change(json!({"action":"save_candidate","request":{
        "expected_catalog":f.snapshot().catalog().manifest_ref,"change":{"action":"revise","base":before,
        "payload_json":serde_json::to_string(&body).unwrap()}}}))["candidate"].clone()).unwrap();
    let unchanged_lock = std::fs::read(f.root.join("library/lock.json")).unwrap();
    for id in [first, second] {
        assert_eq!(
            f.model
                .materialized_placed_symbol_definition(id, symbol)
                .unwrap()
                .unwrap()["drawings"],
            old["drawings"]
        );
    }
    drop(f.prepare(f.selection(&target)).unwrap());
    assert_eq!(
        std::fs::read(f.root.join("library/lock.json")).unwrap(),
        unchanged_lock
    );
    let summary = f.adopt(&target);
    assert_eq!(summary["affected_components"].as_array().unwrap().len(), 2);
    for id in [first, second] {
        assert_eq!(
            f.model
                .materialized_placed_symbol_definition(id, symbol)
                .unwrap()
                .unwrap()["drawings"],
            body["drawings"]
        );
    }
    assert_eq!(
        std::fs::read(f.root.join(&sheet_path)).unwrap(),
        sheet_bytes
    );
    let cold = ProjectResolver::new(&f.root).resolve().unwrap();
    for id in [first, second] {
        assert_eq!(
            cold.materialized_placed_symbol_definition(id, symbol)
                .unwrap()
                .unwrap()["drawings"],
            body["drawings"]
        );
    }
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    for id in [first, second] {
        assert_eq!(
            f.model
                .materialized_placed_symbol_definition(id, symbol)
                .unwrap()
                .unwrap()["drawings"],
            old["drawings"]
        );
    }
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    for id in [first, second] {
        assert_eq!(
            f.model
                .materialized_placed_symbol_definition(id, symbol)
                .unwrap()
                .unwrap()["drawings"],
            body["drawings"]
        );
    }
    for overrides in [true, false] {
        let mut conflict = f.model.clone();
        let owner = conflict
            .component_instances
            .values_mut()
            .find(|ci| ci.placed_symbol_refs.contains(&first))
            .unwrap();
        owner.library_bindings.insert(
            Uuid::new_v4(),
            crate::substrate::LibraryBinding {
                target_object_id: symbol,
                pinned_object_revision: if overrides {
                    f.model.objects[&symbol].object_revision
                } else {
                    prior_symbol_revision
                },
                binding_role: crate::substrate::LibraryBindingRole::Symbol,
                pool_ref: None,
                provenance: None,
                local_override_refs: if overrides {
                    vec![crate::substrate::RevisionedRef {
                        object_id: symbol,
                        object_revision: f.model.objects[&symbol].object_revision,
                    }]
                } else {
                    vec![]
                },
            },
        );
        assert!(
            conflict
                .materialized_placed_symbol_definition(first, symbol)
                .is_err()
        );
    }
    assert!(
        f.model
            .materialized_placed_symbol_definition(first, unrelated)
            .is_err()
    );
    // Both public placed-source paths must refuse persisted ownership that no
    // longer resolves, or a second runtime owner claiming the same placement.
    let first_owner = f
        .model
        .component_instances
        .values()
        .find(|ci| ci.placed_symbol_refs.contains(&first))
        .unwrap()
        .id;
    for duplicate in [false, true] {
        let mut invalid = f.model.clone();
        if duplicate {
            invalid
                .component_instances
                .values_mut()
                .find(|ci| ci.id != first_owner)
                .unwrap()
                .placed_symbol_refs
                .push(first);
        } else {
            invalid.component_instances.remove(&first_owner);
        }
        assert!(
            invalid
                .materialized_placed_symbol_definition(first, symbol)
                .is_err()
        );
        assert!(
            invalid
                .materialized_placed_entity_definition(
                    first,
                    placed.entity.unwrap(),
                    placed.gate.unwrap(),
                    symbol
                )
                .is_err()
        );
    }
    assert_eq!(f.active(unrelated), unrelated_base);
    assert_ne!(f.active(unrelated), unrelated_target);
    // Corrupt/missing common heads must refuse rather than render a birth or
    // unrelated locked source. These disk inputs are negative and restored.
    let lock_path = f.root.join("library/lock.json");
    let bytes = std::fs::read(&lock_path).unwrap();
    let mut broken: Value = serde_json::from_slice(&bytes).unwrap();
    broken["bindings"] = json!([]);
    std::fs::write(&lock_path, serde_json::to_vec(&broken).unwrap()).unwrap();
    assert!(
        f.model
            .materialized_placed_symbol_definition(first, symbol)
            .is_err()
    );
    std::fs::write(&lock_path, &bytes).unwrap();
    let missing_path = f.root.join("library/test-held-lock.json");
    std::fs::rename(&lock_path, &missing_path).unwrap();
    assert!(
        f.model
            .materialized_placed_symbol_definition(first, symbol)
            .is_err()
    );
    std::fs::rename(&missing_path, &lock_path).unwrap();
    assert_eq!(
        f.model
            .materialized_placed_symbol_definition(first, symbol)
            .unwrap()
            .unwrap()["drawings"],
        body["drawings"]
    );
    f.adopt(&before);
    for id in [first, second] {
        assert_eq!(
            f.model
                .materialized_placed_symbol_definition(id, symbol)
                .unwrap()
                .unwrap()["drawings"],
            old["drawings"]
        );
    }
    assert_eq!(f.active(unrelated), unrelated_base);
    let cold = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        cold.materialized_placed_symbol_definition(first, symbol)
            .unwrap()
            .unwrap()["drawings"],
        old["drawings"]
    );
    assert_eq!(
        std::fs::read(f.root.join(&sheet_path)).unwrap(),
        sheet_bytes
    );
    // A fresh Footprint repins its dependent Map and Part. The Map's isolated
    // graph lacks its contextual Part; only the assembled update is complete.
    for (id, field, value) in [
        (choice.footprint, "name", json!("Reviewed land name")),
        (
            choice.pin_pad_map,
            "tags",
            json!(["Reviewed mapping record"]),
        ),
    ] {
        let snapshot = f.snapshot();
        let catalog = inspect_closure(
            &snapshot.manifest().roots,
            snapshot.assets(),
            ClosureLimits {
                max_nodes: limits.files,
                max_depth: limits.depth,
            },
        )
        .unwrap();
        let base = catalog
            .assets()
            .keys()
            .find(|r| r.asset_id == id)
            .unwrap()
            .clone();
        let mut payload = snapshot.assets()[&base].payload().value().clone();
        payload[field] = value;
        let request = json!({"action":"save_candidate","request":{
            "expected_catalog":snapshot.catalog().manifest_ref,
            "change":{"action":"revise","base":base,"payload_json":serde_json::to_string(&payload).unwrap()}}});
        let selected: AssetRevisionRef =
            serde_json::from_value(f.change(request)["candidate"].clone()).unwrap();
        if id == choice.pin_pad_map {
            let snapshot = f.snapshot();
            let isolated = inspect_closure(
                std::slice::from_ref(&selected),
                snapshot.assets(),
                ClosureLimits {
                    max_nodes: limits.files,
                    max_depth: limits.depth,
                },
            )
            .unwrap();
            assert!(!isolated.graph_report().valid);
        }
        let old_lock = std::fs::read(f.root.join("library/lock.json")).unwrap();
        let preview = f.prepare(f.selection(&selected)).unwrap();
        let after: Vec<AssetRevisionRef> =
            serde_json::from_value(preview.summary["after"].clone()).unwrap();
        assert!(after.contains(&selected));
        assert!(after.iter().any(|r| r.asset_id == part));
        assert!(after.contains(&unrelated_base));
        assert!(!after.contains(&unrelated_target));
        assert_eq!(
            std::fs::read(f.root.join("library/lock.json")).unwrap(),
            old_lock
        );
        native::commit_prepared(&mut f.model, &f.root, preview.prepared).unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(f.active(id), selected);
        assert_eq!(f.active(unrelated), unrelated_base);
        assert_eq!(
            std::fs::read(f.root.join(&sheet_path)).unwrap(),
            sheet_bytes
        );
    }
    std::fs::remove_dir_all(f.root).unwrap();
}
