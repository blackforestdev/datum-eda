//! Public selected-update controls over the existing native assignment fixture.
use super::*;
use crate::api::native_write::{self as native, WriteProvenance};
use crate::library_assets::AssetKind;
use crate::substrate::{
    CommitSource, ComponentInstanceAuthority, DesignModel, ElectricalSelectionSnapshot,
    ProjectResolver,
};
use std::path::{Path, PathBuf};

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "selected-update-test",
        CommitSource::Test,
        "Review exact selected update",
    )
}

fn isolated(name: &str) -> bool {
    const KEY: &str = "DATUM_SELECTED_UPDATE_TEST_CHILD";
    if let Some(selected) = std::env::var_os(KEY) {
        assert_eq!(selected, name);
        return false;
    }
    let name_in_harness = name
        .strip_prefix(concat!(env!("CARGO_CRATE_NAME"), "::"))
        .unwrap();
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name_in_harness, "--nocapture"])
        .env(KEY, name)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("1 passed"));
    true
}

struct Fixture {
    root: PathBuf,
    model: DesignModel,
    limits: LibraryAssetLimits,
}

impl Fixture {
    fn snapshot(&self) -> crate::substrate::library_asset_resolver::LibraryAssetSnapshot {
        let state = read_state(&self.root, self.limits).unwrap();
        resolve_state(&self.root, &state, &BTreeMap::new(), self.limits).unwrap()
    }

    fn prepare(&self, request: Value) -> Result<PreparedLibraryChange, EngineError> {
        prepare_library_change(
            &NativeWriteContext {
                model: &self.model,
                project_root: &self.root,
            },
            provenance(),
            request,
        )
    }

    fn change(&mut self, request: Value) -> Value {
        let change = self.prepare(request).unwrap();
        native::commit_prepared(&mut self.model, &self.root, change.prepared).unwrap();
        change.summary
    }

    fn capture(&mut self, id: Uuid) -> AssetRevisionRef {
        let catalog = self.snapshot().catalog().manifest_ref.clone();
        let summary = self.change(json!({"action":"save_candidate","request":{
            "expected_catalog":catalog,"change":{"action":"capture_native","asset_id":id}
        }}));
        let selected = summary["asset_changes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["asset_id"] == json!(id))
            .unwrap();
        assert_eq!(selected["review"]["comparison"], "first_managed_record");
        assert_eq!(selected["native_baseline"]["exists"], true);
        assert_eq!(selected["native_baseline"]["value"]["uuid"], json!(id));
        assert!(
            summary["native_materialization_changes"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        serde_json::from_value(summary["candidate"].clone()).unwrap()
    }

    fn revise(&mut self, base: &AssetRevisionRef, name: &str) -> AssetRevisionRef {
        let snapshot = self.snapshot();
        let mut payload = snapshot.assets()[base].payload().value().clone();
        payload["name"] = name.into();
        serde_json::from_value(self.change(json!({"action":"save_candidate","request":{
            "expected_catalog":snapshot.catalog().manifest_ref,
            "change":{"action":"revise","base":base,"payload_json":serde_json::to_string(&payload).unwrap()}
        }}))["candidate"].clone()).unwrap()
    }

    fn selection(&self, selected: &AssetRevisionRef) -> Value {
        json!({"action":"adopt_selection","request":{
            "expected_catalog":self.snapshot().catalog().manifest_ref,"selected":selected
        }})
    }

    fn adopt(&mut self, selected: &AssetRevisionRef) -> Value {
        self.change(self.selection(selected))
    }

    fn active(&self, id: Uuid) -> AssetRevisionRef {
        self.snapshot()
            .lock()
            .assets
            .iter()
            .find(|asset| asset.asset.asset_id == id)
            .unwrap()
            .asset
            .clone()
    }
}

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

#[test]
fn public_selected_update_retains_unrelated_active_and_exact_historical_choice() {
    const NAME: &str = concat!(
        module_path!(),
        "::public_selected_update_retains_unrelated_active_and_exact_historical_choice"
    );
    if isolated(NAME) {
        return;
    }
    let (root, mut model) =
        native::schematic_library::part_placement::tests::complete_assignment_history_fixture();
    let component = model
        .component_instances
        .values()
        .find(|ci| ci.authority == ComponentInstanceAuthority::Authored)
        .unwrap()
        .clone();
    let part = component.part_ref.unwrap();
    let sources = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let placed = sources
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .find_map(|sheet| sheet.symbols.get(&component.placed_symbol_refs[0]))
        .unwrap();
    let choice = native::schematic_library::part_placement::CompletePartSelection::capture(
        &model,
        part,
        placed.gate.unwrap(),
        Uuid::parse_str(placed.lib_id.as_deref().unwrap()).unwrap(),
    )
    .unwrap();
    let map = model
        .materialized_pool_object_revision(
            choice.pin_pad_map,
            "pin_pad_maps",
            model.objects[&choice.pin_pad_map].object_revision,
        )
        .unwrap();
    let part_source = model
        .source_shards
        .iter()
        .find(|source| source.shard_id == model.objects[&part].source_shard_id)
        .unwrap();
    let mut second_part: Value =
        serde_json::from_slice(&std::fs::read(root.join(&part_source.relative_path)).unwrap())
            .unwrap();
    let other_part = Uuid::new_v4();
    second_part["uuid"] = json!(other_part);
    second_part["object_revision"] = json!(1);
    let independent = Uuid::new_v4();
    use native::library::{PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec};
    let create = native::library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![
            Spec::Create {
                target: Target::new("pool", "parts", other_part),
                object: second_part,
            },
            Spec::Create {
                target: Target::new("pool", "units", independent),
                object: native::library::pool_unit_payload(independent, "Independent original", ""),
            },
        ],
    )
    .unwrap();
    native::commit_prepared(&mut model, &root, create).unwrap();
    // Pin maps belong to their exact Part. Reusing engineering geometry does
    // not authorize the cloned Part to claim the original Part's map.
    let map_create = (native::registry::find_native_write_verb("datum.library.create")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"kind":"pin_pad_maps","object_id":Uuid::new_v4(),"pool_path":"pool",
            "fields":{"part":other_part,"footprint":choice.footprint,
                "mappings":map["mappings"],"assign_default":true}}),
    )
    .unwrap();
    native::commit_prepared(&mut model, &root, map_create).unwrap();
    let (bytes, files, depth) = inventory(&root);
    // Explicit process-isolated fixture policy only, derived from its inventory
    // and exercised transitions. These figures install no production defaults.
    let transitions = [
        "format",
        "capture-a",
        "adopt-a",
        "capture-b",
        "adopt-b",
        "capture-c",
        "adopt-c",
        "revise-shared",
        "revise-c",
        "adopt-shared",
        "adopt-c",
        "revise-c-again",
        "historical-c",
    ];
    let limits = LibraryAssetLimits {
        metadata_bytes: bytes * transitions.len(),
        blob_bytes: bytes * transitions.len(),
        files: files * transitions.len(),
        depth: depth + 1,
        total_bytes: bytes * files * transitions.len(),
    };
    crate::substrate::library_asset_resolver::configure_server_policy(limits).unwrap();
    let mut f = Fixture {
        root,
        model,
        limits,
    };
    f.change(json!({"action":"adopt_profile"}));
    // A prepared first managed record must not describe a changed native file
    // as the preimage captured by the old model revision.
    let capture_preview = f
        .prepare(json!({"action":"save_candidate","request":{
            "expected_catalog":f.snapshot().catalog().manifest_ref,
            "change":{"action":"capture_native","asset_id":part}
        }}))
        .unwrap();
    let native_path = f.root.join(format!("pool/parts/{part}.json"));
    let original_native = std::fs::read(&native_path).unwrap();
    let shard_id = f.model.objects[&part].source_shard_id;
    assert!(
        f.model
            .capture_source_shard_value_bounded(&f.root, shard_id, original_native.len() - 1,)
            .is_err()
    );
    let (certified_native, observed_bytes) = f
        .model
        .capture_source_shard_value_bounded(&f.root, shard_id, original_native.len())
        .unwrap();
    assert_eq!(observed_bytes, original_native.len());
    assert_eq!(certified_native["uuid"], json!(part));
    assert!(
        f.model
            .capture_source_shard_value_bounded(
                &f.root.join("different-project"),
                shard_id,
                original_native.len(),
            )
            .is_err()
    );
    let mut changed_native = original_native.clone();
    changed_native.extend_from_slice(b"\n{");
    std::fs::write(&native_path, changed_native).unwrap();
    let refused = super::super::change_review::attach(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        &capture_preview.prepared,
        &mut capture_preview.summary.clone(),
        f.limits,
    );
    std::fs::write(&native_path, original_native).unwrap();
    assert!(
        refused.is_err(),
        "stale native preimage must not be presented"
    );
    let first_a = f.capture(part);
    f.adopt(&first_a);
    let first_b = f.capture(other_part);
    f.adopt(&first_b);
    let first_c = f.capture(independent);
    f.adopt(&first_c);
    let baseline = f.snapshot();
    let shared = baseline
        .lock()
        .assets
        .iter()
        .find(|asset| {
            asset.asset.asset_id != independent
                && baseline.assets()[&asset.asset].revision().kind == AssetKind::Unit
        })
        .unwrap()
        .asset
        .clone();
    let revised_shared = f.revise(&shared, "Reviewed shared unit");
    let stale_catalog = f.snapshot().catalog().manifest_ref.clone();
    let pending_c = f.revise(&first_c, "Unrelated pending unit");
    let lock_before = std::fs::read(f.root.join("library/lock.json")).unwrap();
    let c_path = f.root.join(format!("pool/units/{independent}.json"));
    let c_before = std::fs::read(&c_path).unwrap();
    let transaction_count = f.model.journal.len();
    let request = f.selection(&revised_shared);
    let preview = f.prepare(request.clone()).unwrap();
    let changes = preview.summary["asset_changes"].as_array().unwrap();
    let shared_review = &changes
        .iter()
        .find(|row| row["asset_id"] == json!(shared.asset_id))
        .unwrap()["review"];
    assert_eq!(shared_review["before"], json!(shared));
    assert_eq!(shared_review["after"], json!(revised_shared));
    assert_eq!(shared_review["checks"], "not_evaluated");
    assert!(
        shared_review["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["path"] == json!(["name"])
                && field["after"] == json!("\"Reviewed shared unit\""))
    );
    for parent in [part, other_part] {
        let review = &changes
            .iter()
            .find(|row| row["asset_id"] == json!(parent))
            .unwrap()["review"];
        assert!(review["engineering_basis_changed"].as_bool().unwrap());
        assert!(!review["dependencies"].as_array().unwrap().is_empty());
    }
    assert!(
        !changes
            .iter()
            .any(|row| row["asset_id"] == json!(independent))
    );
    assert!(
        preview.summary["native_materialization_changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["asset_id"] == json!(shared.asset_id)
                && row["after"]["name"] == "Reviewed shared unit"
                && row["created"] == false)
    );
    let after: Vec<AssetRevisionRef> =
        serde_json::from_value(preview.summary["after"].clone()).unwrap();
    assert!(after.contains(&revised_shared));
    assert!(after.contains(&first_c));
    assert!(!after.contains(&pending_c));
    assert!(!after.contains(&first_a));
    assert!(!after.contains(&first_b));
    assert!(after.iter().any(|reference| reference.asset_id == part));
    assert!(
        after
            .iter()
            .any(|reference| reference.asset_id == other_part)
    );
    assert_eq!(f.model.journal.len(), transaction_count);
    assert_eq!(
        std::fs::read(f.root.join("library/lock.json")).unwrap(),
        lock_before
    );
    assert_eq!(std::fs::read(&c_path).unwrap(), c_before);
    assert!(
        f.prepare(json!({"action":"adopt_selection","request":{
            "expected_catalog":stale_catalog,"selected":revised_shared
        }}))
        .unwrap_err()
        .to_string()
        .contains("catalog changed")
    );
    let mut unavailable = revised_shared.clone();
    unavailable.asset_id = Uuid::new_v4();
    assert!(
        f.prepare(f.selection(&unavailable))
            .unwrap_err()
            .to_string()
            .contains("unavailable")
    );
    native::commit_prepared(&mut f.model, &f.root, preview.prepared).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.active(shared.asset_id), revised_shared);
    assert_ne!(f.active(part), first_a);
    assert_ne!(f.active(other_part), first_b);
    assert_eq!(f.active(independent), first_c);
    assert_eq!(std::fs::read(&c_path).unwrap(), c_before);
    assert!(
        f.prepare(f.selection(&revised_shared))
            .unwrap_err()
            .to_string()
            .contains("already active")
    );
    // Historical shared selection chooses the recorded coherent ancestors of
    // BOTH dependent Parts, preserving the independent active revision.
    let stable_lock = std::fs::read(f.root.join("library/lock.json")).unwrap();
    let old_shared = f.prepare(f.selection(&shared)).unwrap();
    assert_eq!(
        std::fs::read(f.root.join("library/lock.json")).unwrap(),
        stable_lock
    );
    native::commit_prepared(&mut f.model, &f.root, old_shared.prepared).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.active(shared.asset_id), shared);
    assert_eq!(f.active(part), first_a);
    assert_eq!(f.active(other_part), first_b);
    assert_eq!(f.active(independent), first_c);
    // An independent historical revision can be deliberately selected even
    // while a newer local candidate exists. It is not replaced by catalog head.
    f.adopt(&pending_c);
    let newer_c = f.revise(&pending_c, "Newest pending independent unit");
    f.adopt(&first_c);
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.active(independent), first_c);
    assert_ne!(f.active(independent), newer_c);
    let native_c: Value = serde_json::from_slice(&std::fs::read(&c_path).unwrap()).unwrap();
    assert_eq!(native_c["name"], "Independent original");
    std::fs::remove_dir_all(f.root).unwrap();
}

#[path = "managed_symbol_tests.rs"]
mod managed_symbol_tests;
