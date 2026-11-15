use super::*;
use crate::api::native_write::{
    self as nw, commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
    registry::find_native_write_verb,
};
use crate::substrate::CommitSource;
use serde_json::json;
use std::collections::BTreeMap;

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Acquire reusable native Part")
}
struct Project {
    root: PathBuf,
    model: DesignModel,
}
impl Project {
    fn new() -> Self {
        let root = nw::test_support::temp_project_root("native_part_acquisition");
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Native Part acquisition".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        Self {
            model: ProjectResolver::new(&root).resolve().unwrap(),
            root,
        }
    }
    fn create(&mut self, kind: &str, id: Uuid, fields: Value) {
        let prepared = (find_native_write_verb("datum.library.create")
            .unwrap()
            .build)(
            &NativeWriteContext {
                model: &self.model,
                project_root: &self.root,
            },
            provenance(),
            json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
        )
        .unwrap();
        commit_prepared(&mut self.model, &self.root, prepared).unwrap();
    }
    fn bytes(&self) -> BTreeMap<String, Vec<u8>> {
        self.model
            .source_shards
            .iter()
            .map(|s| (s.relative_path.clone(), std::fs::read(&s.path).unwrap()))
            .collect()
    }
    fn prepare(&self, snapshot: &NativePartPreview) -> Result<PreparedWrite, EngineError> {
        build(
            &NativeWriteContext {
                model: &self.model,
                project_root: &self.root,
            },
            provenance(),
            LibraryAcquisitionRequest {
                source_format: "datum_native_part".into(),
                source_path: snapshot.source_path.clone(),
                symbol_name: snapshot.part.to_string(),
                expected_source_hash: snapshot.digest.clone(),
            },
        )
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn complete() -> (Project, Uuid) {
    complete_with_metadata("Native source", &[], "Reusable package")
}
fn complete_with_metadata(description: &str, tags: &[&str], package_name: &str) -> (Project, Uuid) {
    let mut p = Project::new();
    let [
        unit,
        symbol,
        pin,
        entity,
        gate,
        gate_b,
        package,
        stack,
        footprint,
        pad,
        pad_b,
        part,
        map,
    ] = std::array::from_fn(|_| Uuid::new_v4());
    for (kind, id, fields) in [
        (
            "units",
            unit,
            json!({"name":"Reusable logical terminal","pins":{pin.to_string():{"name":"Signal","electrical_type":"Passive"}}}),
        ),
        (
            "symbols",
            symbol,
            json!({"name":"Reusable body","unit":unit,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}),
        ),
        ("packages", package, json!({"name":package_name})),
        (
            "padstacks",
            stack,
            json!({"name":"Rounded copper","aperture":{"round_rect":{"width_nm":800000,"height_nm":950000,"rratio_ppm":250000}}}),
        ),
        (
            "footprints",
            footprint,
            json!({"name":"Reusable land","package":package,"pads":{pad.to_string():{"uuid":pad,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":0},pad_b.to_string():{"uuid":pad_b,"name":"2","position":{"x":2000000,"y":0},"padstack":stack,"layer":0}}}),
        ),
        (
            "entities",
            entity,
            json!({"name":"Reusable entity","manufacturer":"","prefix":"U","gates":{gate.to_string():{"uuid":gate,"name":"A","unit":unit,"symbol":symbol},gate_b.to_string():{"uuid":gate_b,"name":"B","unit":unit,"symbol":symbol}}}),
        ),
        (
            "parts",
            part,
            json!({"manufacturer":"Designer","mpn":"REUSABLE","description":description,"tags":tags,"datasheet":"","value":"10k","entity":entity,"package":package,"default_footprint":footprint}),
        ),
        (
            "pin_pad_maps",
            map,
            json!({"part":part,"footprint":footprint,"mappings":{pad.to_string():{"gate":gate,"pin":pin},pad_b.to_string():{"gate":gate_b,"pin":pin}},"assign_default":true}),
        ),
    ] {
        p.create(kind, id, fields);
    }
    (p, part)
}

#[test]
fn native_catalog_exposes_declared_search_metadata_without_changing_source_or_qualification() {
    let (source, id) = complete_with_metadata(
        "Precision resistor",
        &["passive", "analog"],
        "0603 chip package",
    );
    let before = source.bytes();
    let original = preview(&source.root, id).unwrap();
    let choices = catalog(&source.root).unwrap();
    let choice = choices.iter().find(|row| row.id == id).unwrap();
    assert_eq!(choice.label, "Designer REUSABLE · 10k");
    assert!(choice.refusal.is_none());
    for declared in [
        "Precision resistor",
        "passive",
        "analog",
        "0603 chip package",
    ] {
        assert!(choice.search_text.contains(declared), "Missing {declared}");
    }
    assert_eq!(catalog(&source.root).unwrap(), choices);
    assert_eq!(preview(&source.root, id).unwrap(), original);
    assert_eq!(source.bytes(), before);
}
fn seed_symbols(target: &mut Project, snapshot: &NativePartPreview) {
    let specs = snapshot
        .nodes
        .iter()
        .filter(|n| matches!(n.kind.as_str(), "units" | "symbols"))
        .map(|n| PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", &n.kind, n.id),
            object: n.object.clone(),
        })
        .collect();
    let prepared =
        build_pool_library_write(&target.model, provenance(), Some("pool"), specs).unwrap();
    commit_prepared(&mut target.model, &target.root, prepared).unwrap();
}
#[test]
fn native_part_acquisition_reuses_identical_symbols_and_preserves_source_history_and_cold_geometry()
{
    let (source, id) = complete();
    let source_bytes = source.bytes();
    let snapshot = preview(&source.root, id).unwrap();
    assert_eq!(snapshot.nodes.len(), 8);
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|n| n.kind == "entities")
            .unwrap()
            .object["gates"]
            .as_object()
            .unwrap()
            .len(),
        2
    );
    let mut target = Project::new();
    seed_symbols(&mut target, &snapshot);
    let before = target.bytes();
    assert_eq!(snapshot.reused(&target.model).unwrap().len(), 2);
    drop(target.prepare(&snapshot).unwrap());
    assert_eq!(before, target.bytes());
    let prepared = target.prepare(&snapshot).unwrap();
    assert_eq!(
        prepared
            .batch
            .operations
            .iter()
            .filter(|o| matches!(o, Operation::CreatePoolLibraryObject { .. }))
            .count(),
        6
    );
    commit_prepared(&mut target.model, &target.root, prepared).unwrap();
    let committed = target.bytes();
    for n in &snapshot.nodes {
        assert_eq!(
            source::node(&target.model, n.id, &n.kind).unwrap().object,
            n.object
        );
    }
    for (path, bytes) in &before {
        if path.starts_with("pool/") {
            assert_eq!(committed[path], *bytes);
        }
    }
    assert!(
        target
            .prepare(&snapshot)
            .unwrap_err()
            .to_string()
            .contains("already in the Project")
    );
    target
        .model
        .commit_journal_undo(&target.root, provenance().into())
        .unwrap();
    assert!(!target.model.objects.contains_key(&id));
    for (path, bytes) in &before {
        if path.starts_with("pool/") {
            assert_eq!(std::fs::read(target.root.join(path)).unwrap(), *bytes);
        }
    }
    target
        .model
        .commit_journal_redo(&target.root, provenance().into())
        .unwrap();
    target.model = ProjectResolver::new(&target.root).resolve().unwrap();
    for n in &snapshot.nodes {
        assert_eq!(
            source::node(&target.model, n.id, &n.kind).unwrap().object,
            n.object
        );
    }
    assert_eq!(source_bytes, source.bytes());
}
#[test]
fn native_part_acquisition_refuses_changed_source_and_target_conflicts_without_writes() {
    let (mut source, id) = complete();
    let snapshot = preview(&source.root, id).unwrap();
    let mut target = Project::new();
    let symbol = snapshot.nodes.iter().find(|n| n.kind == "symbols").unwrap();
    let mut conflicting = symbol.object.clone();
    conflicting["name"] = json!("Different saved drawing");
    let prepared = build_pool_library_write(
        &target.model,
        provenance(),
        Some("pool"),
        vec![
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new(
                    "pool",
                    "units",
                    snapshot
                        .nodes
                        .iter()
                        .find(|n| n.kind == "units")
                        .unwrap()
                        .id,
                ),
                object: snapshot
                    .nodes
                    .iter()
                    .find(|n| n.kind == "units")
                    .unwrap()
                    .object
                    .clone(),
            },
            PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "symbols", symbol.id),
                object: conflicting,
            },
        ],
    )
    .unwrap();
    commit_prepared(&mut target.model, &target.root, prepared).unwrap();
    let before = target.bytes();
    assert!(
        target
            .prepare(&snapshot)
            .unwrap_err()
            .to_string()
            .contains("conflicts")
    );
    assert_eq!(before, target.bytes());
    let mut stale = snapshot.clone();
    stale.digest = "sha256:stale".into();
    assert!(
        target
            .prepare(&stale)
            .unwrap_err()
            .to_string()
            .contains("changed since preview")
    );
    assert_eq!(before, target.bytes());
    let blank = Project::new();
    let blank_before = blank.bytes();
    let mut changed = symbol.object.clone();
    changed["name"] = json!("Source drawing renamed after preview");
    let prepared = build_pool_library_write(
        &source.model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "symbols", symbol.id),
            object: changed,
        }],
    )
    .unwrap();
    commit_prepared(&mut source.model, &source.root, prepared).unwrap();
    assert!(
        blank
            .prepare(&snapshot)
            .unwrap_err()
            .to_string()
            .contains("changed since preview")
    );
    assert_eq!(blank_before, blank.bytes());
    let current = preview(&source.root, id).unwrap();
    assert_ne!(current.digest, snapshot.digest);
    assert!(blank.prepare(&current).is_ok());
}
#[test]
fn native_part_preview_refuses_linked_process_resources_instead_of_dropping_them() {
    let (mut source, id) = complete();
    let snapshot = preview(&source.root, id).unwrap();
    let footprint = snapshot
        .nodes
        .iter()
        .find(|n| n.kind == "footprints")
        .unwrap();
    let mut raw = footprint.object.clone();
    raw["process_aperture_policy"] = json!("Linked source process policy");
    let prepared = build_pool_library_write(
        &source.model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "footprints", footprint.id),
            object: raw,
        }],
    )
    .unwrap();
    commit_prepared(&mut source.model, &source.root, prepared).unwrap();
    ProjectResolver::new(&source.root).resolve().unwrap();
    assert!(
        preview(&source.root, id)
            .unwrap_err()
            .to_string()
            .contains("linked models or footprint process resources")
    );
}

#[test]
fn native_part_owned_pad_identity_mismatch_and_alias_refuse_without_target_writes() {
    let (source, id) = complete();
    let snapshot = preview(&source.root, id).unwrap();
    let footprint = snapshot
        .nodes
        .iter()
        .find(|n| n.kind == "footprints")
        .unwrap();
    let (pad_key, pad) = footprint.object["pads"]
        .as_object()
        .unwrap()
        .iter()
        .next()
        .unwrap();
    let target = Project::new();
    let before = target.bytes();
    for alias in [false, true] {
        let mut raw = footprint.object.clone();
        if alias {
            let alias_key = pad_key.to_uppercase();
            assert_ne!(&alias_key, pad_key);
            raw["pads"]
                .as_object_mut()
                .unwrap()
                .insert(alias_key, pad.clone());
        } else {
            raw["pads"][pad_key]["uuid"] = json!(Uuid::new_v4());
        }
        std::fs::write(
            source.root.join(&footprint.source_path),
            serde_json::to_vec_pretty(&raw).unwrap(),
        )
        .unwrap();
        assert!(preview(&source.root, id).is_err());
        assert!(target.prepare(&snapshot).is_err());
        assert_eq!(before, target.bytes());
    }
}

#[test]
fn native_part_catalog_keeps_valid_part_when_an_unrelated_part_is_malformed() {
    assert!(
        catalog(Path::new(""))
            .unwrap_err()
            .to_string()
            .contains("Enter a saved native source")
    );
    let (source, id) = complete();
    let malformed = Uuid::new_v4();
    std::fs::write(
        source.root.join(format!("pool/parts/{malformed}.json")),
        serde_json::to_vec(&json!({
            "schema_version":1,"uuid":malformed,"manufacturer":"Incomplete source","mpn":"Broken"
        }))
        .unwrap(),
    )
    .unwrap();
    let choices = catalog(&source.root).unwrap();
    assert!(choices.iter().any(|p| p.id == id && p.refusal.is_none()));
    assert!(
        choices
            .iter()
            .any(|p| p.id == malformed && p.refusal.is_some())
    );
    assert!(preview(&source.root, malformed).is_err());
}
