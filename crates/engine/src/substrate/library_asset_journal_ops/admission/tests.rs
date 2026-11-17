use super::*;
use crate::api::native_write::{self as native, WriteProvenance};
use crate::substrate::{CommitSource, ProjectResolver};
mod managed_assignment;
mod managed_bindings;
mod physical_adoption;

#[test]
fn actual_profile_admission_uses_sealed_candidate_and_refuses_changed_input() {
    const CHILD: &str = "DATUM_LIBRARY_ADMISSION_TEST_CHILD";
    const NAME: &str = "substrate::library_asset_journal_ops::admission::tests::actual_profile_admission_uses_sealed_candidate_and_refuses_changed_input";
    if std::env::var_os(CHILD).is_none() {
        // OnceLock policy is confined to an exact child test process. No
        // production initializer, wire setter or shared test state changes.
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", NAME, "--nocapture"])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
        return;
    }
    let root = std::env::temp_dir().join(format!("datum_admission_control_{}", Uuid::new_v4()));
    native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Admission control".into(),
            existing_ids: None,
        },
    )
    .unwrap();
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
    // Fixture-only bounds derive from actual bootstrap bytes/count/depth;
    // three new metadata files are lock, catalog and initial release.
    let limits = LibraryAssetLimits {
        metadata_bytes: bytes,
        blob_bytes: bytes,
        files: files + 3,
        depth: depth + 1,
        total_bytes: bytes * (files + 3),
    };
    crate::substrate::library_asset_resolver::configure_server_policy(limits).unwrap();
    let before = ProjectResolver::new(&root).resolve().unwrap();
    let original = std::fs::read(root.join("project.json")).unwrap();
    let prepared = native::library_assets::prepare_profile_adoption(
        &native::registry::NativeWriteContext {
            model: &before,
            project_root: &root,
        },
        WriteProvenance::new(
            "admission-test",
            CommitSource::Test,
            "Review separate format baseline",
        ),
        limits,
    )
    .unwrap();
    let mut batch = prepared.batch;
    // Commit strips validated guards before its private admission owner.
    batch
        .operations
        .retain(|op| !matches!(op, Operation::GuardObjectRevision { .. }));
    // The ordinary preflight succeeds; only the complete evaluated source
    // proof below authorizes publication in the real journaled commit.
    super::super::validate_batch(&before, &root, &batch).unwrap();
    let lease = ProjectWriteLease::acquire(&root).unwrap();
    let admission = prepare_under_lease(&root, &lease, &before, &batch, limits)
        .unwrap()
        .unwrap();
    let mut changed = batch.clone();
    changed.provenance.reason.push_str(" changed");
    assert!(admission.evaluation_permit(&before, &changed).is_err());
    let permit = admission.evaluation_permit(&before, &batch).unwrap();
    assert!(permit.validate_operations(&[]).is_err());
    assert!(permit.validate_operations(&batch.operations).is_ok());
    let writes =
        crate::substrate::journal::stage_operation_shard_writes(&root, &before, &batch).unwrap();
    let mut candidate = before.clone();
    crate::substrate::journal::update_staged_source_hashes(&mut candidate.source_shards, &writes)
        .unwrap();
    crate::substrate::electrical_transaction::retain_preimage_hashes(&before, &mut candidate);
    let inverse = crate::substrate::journal::inverse_operations_for_batch(&before, &batch).unwrap();
    let mut report = candidate
        .apply_library_candidate(batch, &admission)
        .unwrap();
    crate::substrate::journal::update_staged_source_hashes(&mut candidate.source_shards, &writes)
        .unwrap();
    report.transaction.inverse_operations = inverse.clone();
    candidate.journal.last_mut().unwrap().inverse_operations = inverse;
    crate::substrate::commit::refresh_journaled_report_revision(&mut candidate, &mut report, None)
        .unwrap();
    assert_eq!(
        candidate.journal_history_availability().unwrap(),
        (false, false)
    );
    admission
        .finish_evaluated(&root, &lease, &before, &candidate, &writes)
        .unwrap();
    assert_eq!(std::fs::read(root.join("project.json")).unwrap(), original);
    assert!(!root.join("library/lock.json").exists());
    assert_eq!(candidate.journal_cursor, before.journal_cursor);
    // Fault injection only: the actual evaluated record is appended without
    // integrity publication or source promotion. Empty/technical history
    // must not turn the remaining raw v1 header into a mixed managed model.
    crate::substrate::journal::append_transaction_journal(&root, &report.transaction).unwrap();
    drop(lease);
    let error = ProjectResolver::new(&root).resolve().unwrap_err();
    assert!(
        error.to_string().contains("unrecovered library transition"),
        "{error}"
    );
    assert_eq!(std::fs::read(root.join("project.json")).unwrap(), original);
    std::fs::remove_dir_all(root).unwrap();
}

fn isolated(name: &str) -> bool {
    const KEY: &str = "DATUM_LIBRARY_PUBLIC_HISTORY_TEST_CHILD";
    if let Some(selected) = std::env::var_os(KEY) {
        assert_eq!(selected, name);
        return false;
    }
    // module_path! includes the crate; libtest's exact names do not.
    let harness_name = name
        .strip_prefix(concat!(env!("CARGO_CRATE_NAME"), "::"))
        .expect("public control name starts with this crate");
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", harness_name, "--nocapture"])
        .env(KEY, name)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "exact child control {harness_name} did not report one pass:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    true
}
fn public_provenance() -> WriteProvenance {
    WriteProvenance::new(
        "library-history-test",
        CommitSource::Test,
        "Review exact local revision",
    )
}
fn snapshot(root: &Path, limits: LibraryAssetLimits) -> LibraryAssetSnapshot {
    let state = read_state(root, limits).unwrap();
    resolve_state(root, &state, &BTreeMap::new(), limits).unwrap()
}
fn payload_bytes(
    root: &Path,
    snapshot: &LibraryAssetSnapshot,
    reference: &AssetRevisionRef,
    limits: LibraryAssetLimits,
) -> Vec<u8> {
    let path = crate::substrate::library_asset_resolver::blob_path(
        &snapshot.assets()[reference].revision().payload_ref,
    );
    crate::substrate::library_asset_resolver::read_content(
        root,
        &path,
        &BTreeMap::new(),
        limits.blob_bytes,
    )
    .unwrap()
}
fn public_change(root: &Path, model: &mut DesignModel, request: Value) -> Value {
    let change = native::library_assets::prepare_library_change(
        &native::registry::NativeWriteContext {
            model,
            project_root: root,
        },
        public_provenance(),
        request,
    )
    .unwrap();
    native::commit_prepared(model, root, change.prepared).unwrap();
    change.summary
}
fn adopt(
    root: &Path,
    model: &mut DesignModel,
    limits: LibraryAssetLimits,
    reference: &AssetRevisionRef,
) {
    let catalog = snapshot(root, limits).catalog().manifest_ref.clone();
    public_change(
        root,
        model,
        serde_json::json!({"action":"adopt", "request": {
            "expected_catalog":catalog, "roots":[reference],
        }}),
    );
}

#[test]
fn public_candidate_adoption_cold_history_and_relocation_keep_exact_revisions() {
    const NAME: &str = concat!(
        module_path!(),
        "::public_candidate_adoption_cold_history_and_relocation_keep_exact_revisions"
    );
    if isolated(NAME) {
        return;
    }
    use native::library::{
        PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
        build_pool_library_write, pool_unit_payload,
    };
    let root =
        std::env::temp_dir().join(format!("datum_library_public_history_{}", Uuid::new_v4()));
    native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Local revision history".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let unit = Uuid::new_v4();
    let native_unit = build_pool_library_write(
        &model,
        public_provenance(),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "units", unit),
            object: pool_unit_payload(unit, "Original pins", ""),
        }],
    )
    .unwrap();
    native::commit_prepared(&mut model, &root, native_unit).unwrap();
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
    // Test-only budget is derived from this native fixture and the number of
    // planned state transitions. It installs no production defaults or caps.
    let transitions = [
        "format", "capture", "adopt", "revise", "adopt", "undo", "redo",
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
    assert_eq!(model.project.schema_version, Some(2));
    assert_eq!(
        model.journal_history_availability().unwrap(),
        (false, false)
    );
    assert!(
        model
            .commit_journal_undo(&root, public_provenance().into())
            .is_err()
    );
    let empty = snapshot(&root, limits);
    assert!(empty.lock().assets.is_empty());
    let original_lock = std::fs::read(root.join("library/lock.json")).unwrap();
    let original_source = std::fs::read(root.join(format!("pool/units/{unit}.json"))).unwrap();
    let catalog = empty.catalog().manifest_ref.clone();
    let summary = public_change(
        &root,
        &mut model,
        serde_json::json!({"action":"save_candidate", "request": {
            "expected_catalog":catalog, "change":{"action":"capture_native","asset_id":unit},
        }}),
    );
    let first: AssetRevisionRef = serde_json::from_value(summary["candidate"].clone()).unwrap();
    assert_eq!(
        std::fs::read(root.join("library/lock.json")).unwrap(),
        original_lock
    );
    assert_eq!(
        std::fs::read(root.join(format!("pool/units/{unit}.json"))).unwrap(),
        original_source
    );
    assert!(snapshot(&root, limits).lock().assets.is_empty());
    adopt(&root, &mut model, limits, &first);
    assert_eq!(snapshot(&root, limits).lock().assets[0].asset, first);
    model = ProjectResolver::new(&root).resolve().unwrap();
    let first_lock = std::fs::read(root.join("library/lock.json")).unwrap();
    let first_snapshot = snapshot(&root, limits);
    let payload = payload_bytes(&root, &first_snapshot, &first, limits);
    let mut edited = first_snapshot.assets()[&first].payload().value().clone();
    edited["name"] = "Reviewed replacement pins".into();
    let summary = public_change(
        &root,
        &mut model,
        serde_json::json!({"action":"save_candidate", "request": {
            "expected_catalog":first_snapshot.catalog().manifest_ref,
            "change":{"action":"revise","base":first,"payload_json":serde_json::to_string(&edited).unwrap()},
        }}),
    );
    let second: AssetRevisionRef = serde_json::from_value(summary["candidate"].clone()).unwrap();
    assert_ne!(first, second);
    assert_eq!(
        std::fs::read(root.join("library/lock.json")).unwrap(),
        first_lock
    );
    assert_eq!(
        payload_bytes(&root, &snapshot(&root, limits), &first, limits),
        payload
    );
    assert_eq!(snapshot(&root, limits).lock().assets[0].asset, first);
    let before_refusal = model.model_revision.clone();
    let journal_path = crate::substrate::transaction_journal_path(&root);
    let unit_path = root.join(format!("pool/units/{unit}.json"));
    let before_journal = std::fs::read(&journal_path).unwrap();
    let before_unit = std::fs::read(&unit_path).unwrap();
    let incoherent = build_pool_library_write(
        &model,
        public_provenance(),
        None,
        vec![Spec::Set {
            target: Target::new("pool", "units", unit),
            object: pool_unit_payload(unit, "Unreviewed bypass", ""),
        }],
    )
    .unwrap();
    assert!(native::commit_prepared(&mut model, &root, incoherent).is_err());
    assert_eq!(model.model_revision, before_refusal);
    assert_eq!(std::fs::read(&journal_path).unwrap(), before_journal);
    assert_eq!(std::fs::read(&unit_path).unwrap(), before_unit);
    let refused_cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(refused_cold.model_revision, model.model_revision);
    assert_eq!(refused_cold.objects, model.objects);
    assert_eq!(refused_cold.component_instances, model.component_instances);
    assert_eq!(
        std::fs::read(root.join("library/lock.json")).unwrap(),
        first_lock
    );
    adopt(&root, &mut model, limits, &second);
    let after = model.model_revision.clone();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(model.model_revision, after);
    assert_eq!(snapshot(&root, limits).lock().assets[0].asset, second);
    assert_eq!(
        model
            .materialized_source_shard_value_by_relative_path(&format!("pool/units/{unit}.json"))
            .unwrap()["name"],
        "Reviewed replacement pins"
    );
    model
        .commit_journal_undo(&root, public_provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(snapshot(&root, limits).lock().assets[0].asset, first);
    assert_eq!(
        model
            .materialized_source_shard_value_by_relative_path(&format!("pool/units/{unit}.json"))
            .unwrap()["name"],
        "Original pins"
    );
    model
        .commit_journal_redo(&root, public_provenance().into())
        .unwrap();
    assert_eq!(snapshot(&root, limits).lock().assets[0].asset, second);
    let relocated_root = root.with_extension("relocated");
    std::fs::rename(&root, &relocated_root).unwrap();
    assert!(!root.exists());
    let relocated = ProjectResolver::new(&relocated_root).resolve().unwrap();
    assert_eq!(relocated.model_revision, model.model_revision);
    assert_eq!(
        snapshot(&relocated_root, limits).lock().assets[0].asset,
        second
    );
    assert_eq!(
        payload_bytes(
            &relocated_root,
            &snapshot(&relocated_root, limits),
            &first,
            limits
        ),
        payload
    );
    std::fs::remove_dir_all(relocated_root).unwrap();
}

#[test]
fn public_part_adoption_preserves_declared_physical_choice_through_cold_and_history() {
    const NAME: &str = concat!(
        module_path!(),
        "::public_part_adoption_preserves_declared_physical_choice_through_cold_and_history"
    );
    if isolated(NAME) {
        return;
    }
    use native::schematic_library::part_placement::CompletePartSelection;
    // Reuse the existing ordinary guarded assignment fixture, including its
    // actual accepted assignment proposal and full eight-kind physical choice.
    let (root, mut model) =
        native::schematic_library::part_placement::tests::complete_assignment_history_fixture();
    let component = model
        .component_instances
        .values()
        .find(|ci| ci.authority == ComponentInstanceAuthority::Authored)
        .unwrap();
    let (ci, part, placed, before_revision) = (
        component.id,
        component.part_ref.unwrap(),
        component.placed_symbol_refs[0],
        component.object_revision,
    );
    let sources = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let symbol = sources
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .find_map(|sheet| sheet.symbols.get(&placed))
        .unwrap();
    let gate = symbol.gate.unwrap();
    let library_symbol = Uuid::parse_str(
        symbol
            .lib_id
            .as_deref()
            .expect("fixture has a library symbol ID"),
    )
    .unwrap();
    let original_choice =
        CompletePartSelection::capture(&model, part, gate, library_symbol).unwrap();
    let assert_choice = |current: &DesignModel| {
        let selected = CompletePartSelection::capture(current, part, gate, library_symbol).unwrap();
        assert_eq!(selected.footprint, original_choice.footprint);
        assert_eq!(selected.pin_pad_map, original_choice.pin_pad_map);
        selected.verify_declared(current, ci).unwrap();
    };
    assert_choice(&model);
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
    let transitions = [
        "format", "capture", "adopt", "revise", "adopt", "undo", "redo",
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
    let catalog = snapshot(&root, limits).catalog().manifest_ref.clone();
    let captured = public_change(
        &root,
        &mut model,
        serde_json::json!({"action":"save_candidate","request":{
            "expected_catalog":catalog,"change":{"action":"capture_native","asset_id":part},
        }}),
    );
    let first: AssetRevisionRef = serde_json::from_value(captured["candidate"].clone()).unwrap();
    adopt(&root, &mut model, limits, &first);
    let active = snapshot(&root, limits);
    assert_eq!(active.lock().bindings.len(), 1);
    assert_eq!(active.lock().bindings[0].component_instance_id, ci);
    assert_eq!(active.lock().bindings[0].component, first);
    assert_eq!(
        model.component_instances[&ci].object_revision,
        before_revision
    );
    // The legacy editor cannot change a managed factual pin mapping outside
    // explicit revision adoption, even when the replacement is schema-valid.
    let physical = CompletePartSelection::capture(&model, part, gate, library_symbol).unwrap();
    let map_id = physical.pin_pad_map;
    let map_asset = active
        .assets()
        .iter()
        .find(|(reference, _)| reference.asset_id == map_id)
        .unwrap()
        .1;
    let mut swapped = map_asset.payload().value().clone();
    let rows = swapped["mappings"].as_object_mut().unwrap();
    let keys: Vec<_> = rows.keys().cloned().collect();
    let one = rows[&keys[0]].clone();
    let two = rows[&keys[1]].clone();
    rows.insert(keys[0].clone(), two);
    rows.insert(keys[1].clone(), one);
    let bypass = native::library::build_pool_library_write(
        &model,
        public_provenance(),
        None,
        vec![native::library::PoolLibraryOperationSpec::Set {
            target: native::library::PoolLibraryObjectTarget::new("pool", "pin_pad_maps", map_id),
            object: swapped,
        }],
    )
    .unwrap();
    let before_bypass = model.model_revision.clone();
    let journal_path = crate::substrate::transaction_journal_path(&root);
    let map_path = root.join(format!("pool/pin_pad_maps/{map_id}.json"));
    let before_journal = std::fs::read(&journal_path).unwrap();
    let before_map = std::fs::read(&map_path).unwrap();
    assert!(native::commit_prepared(&mut model, &root, bypass).is_err());
    assert_eq!(model.model_revision, before_bypass);
    assert_eq!(std::fs::read(&journal_path).unwrap(), before_journal);
    assert_eq!(std::fs::read(&map_path).unwrap(), before_map);
    let refused_cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(refused_cold.model_revision, model.model_revision);
    assert_eq!(refused_cold.objects, model.objects);
    assert_eq!(refused_cold.component_instances, model.component_instances);
    assert_choice(&refused_cold);
    physical.verify_declared(&model, ci).unwrap();
    let mut edited = active.assets()[&first].payload().value().clone();
    edited["description"] = "Recorded engineering review".into();
    let revised = public_change(
        &root,
        &mut model,
        serde_json::json!({"action":"save_candidate","request":{
            "expected_catalog":active.catalog().manifest_ref,
            "change":{"action":"revise","base":first,"payload_json":serde_json::to_string(&edited).unwrap()},
        }}),
    );
    let second: AssetRevisionRef = serde_json::from_value(revised["candidate"].clone()).unwrap();
    assert_eq!(snapshot(&root, limits).lock().bindings[0].component, first);
    adopt(&root, &mut model, limits, &second);
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(snapshot(&root, limits).lock().bindings[0].component, second);
    assert!(model.component_instances[&ci].object_revision > before_revision);
    assert_eq!(
        model.component_instances[&ci].placed_symbol_refs,
        vec![placed]
    );
    assert_choice(&model);
    // A naked CI repin cannot stand in for a recorded library adoption.
    let mut without_adoption = model.clone();
    without_adoption
        .journal
        .last_mut()
        .unwrap()
        .operations
        .retain(|op| !matches!(op, Operation::LibraryAssets { .. }));
    assert!(
        CompletePartSelection::capture(&without_adoption, part, gate, library_symbol)
            .unwrap()
            .verify_declared(&without_adoption, ci)
            .is_err()
    );
    model
        .commit_journal_undo(&root, public_provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(snapshot(&root, limits).lock().bindings[0].component, first);
    assert_library_inverse_faults(&model);
    assert_choice(&model);
    model
        .commit_journal_redo(&root, public_provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(snapshot(&root, limits).lock().bindings[0].component, second);
    assert_choice(&model);
    // This exercises the declared physical choice and affected authored CI.
    // It is not proof of rematerializing a changed footprint on an existing PCB.
    std::fs::remove_dir_all(root).unwrap();
}

/// Mutate copies of an actual published compensation, never a synthetic
/// successful journal. Every altered record must fail the canonical link owner.
fn assert_library_inverse_faults(model: &DesignModel) {
    let transaction = model.journal.last().unwrap();
    let prior = &model.journal[..model.journal.len() - 1];
    crate::substrate::transaction_links::validate_transaction_links(transaction, prior).unwrap();
    for membership in [false, true] {
        let mut fault = transaction.clone();
        let component = fault
            .operations
            .iter_mut()
            .find_map(|op| match op {
                Operation::SetComponentInstance {
                    component_instance, ..
                } => Some(component_instance),
                _ => None,
            })
            .unwrap();
        if membership {
            component["placed_symbol_refs"][0]["object_id"] = serde_json::json!(Uuid::new_v4());
        } else {
            component["part_ref"]["object_revision"] =
                (component["part_ref"]["object_revision"].as_u64().unwrap() + 1).into();
        }
        assert!(
            crate::substrate::transaction_links::validate_transaction_links(&fault, prior).is_err()
        );
    }
    // An old literal inverse is not a valid fallback for this owned family.
    let target = prior
        .iter()
        .find(|tx| Some(tx.transaction_id) == transaction.undo_of)
        .unwrap();
    let mut obsolete = transaction.clone();
    obsolete.operations = target.inverse_operations.clone();
    obsolete.inverse_operations = target.operations.clone();
    assert!(
        crate::substrate::transaction_links::validate_transaction_links(&obsolete, prior).is_err()
    );
    let mut wrong_inverse = transaction.clone();
    wrong_inverse.inverse_operations.pop();
    assert!(
        crate::substrate::transaction_links::validate_transaction_links(&wrong_inverse, prior)
            .is_err()
    );
}
