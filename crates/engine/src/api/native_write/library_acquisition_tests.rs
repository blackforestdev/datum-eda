//! Acquisition uses one canonical history operation and never writes on preview.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::genesis::{GenesisSpec, bootstrap_native_project};
use super::library_acquisition::{LibraryAcquisitionRequest, build_acquisition};
use super::registry::NativeWriteContext;
use super::{PreparedWrite, WriteProvenance, commit_prepared};
use crate::error::EngineError;
use crate::import::{ids_sidecar::compute_source_hash_file, kicad_library};
use crate::pool::Symbol;
use crate::substrate::{CommitSource, DesignModel, ProjectResolver};

const SOURCE: &str = r#"(kicad_symbol_lib (version 20241209)
  (symbol "Sense" (property "Reference" "U" (at 0 2 0))
    (symbol "Sense_0_1" (rectangle (start -1 -1) (end 1 1)
      (stroke (width 0.2) (type default)) (fill (type none))))
    (symbol "Sense_1_1" (pin input line (at -3 0 0) (length 2)
      (name "IN+") (number "3")))))"#;

struct Project {
    work: PathBuf,
    root: PathBuf,
    source: PathBuf,
}

impl Project {
    fn new() -> Self {
        let work = super::test_support::temp_project_root("library_acquisition");
        let root = work.join("project");
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Component acquisition".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let source = work.join("local.kicad_sym");
        std::fs::write(&source, SOURCE).unwrap();
        Self { work, root, source }
    }

    fn resolve(&self) -> DesignModel {
        ProjectResolver::new(&self.root).resolve().unwrap()
    }

    fn request(&self) -> LibraryAcquisitionRequest {
        request(&self.source, "Sense")
    }

    fn prepare(
        &self,
        model: &DesignModel,
        request: LibraryAcquisitionRequest,
    ) -> Result<PreparedWrite, EngineError> {
        build_acquisition(
            &NativeWriteContext {
                model,
                project_root: &self.root,
            },
            provenance(),
            request,
        )
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.work);
    }
}

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "datum-gui",
        CommitSource::Manual,
        "Acquire selected local library symbol",
    )
}

fn request(path: &Path, symbol: &str) -> LibraryAcquisitionRequest {
    LibraryAcquisitionRequest {
        source_format: "kicad_symbol".into(),
        source_path: path.to_path_buf(),
        symbol_name: symbol.into(),
        expected_source_hash: compute_source_hash_file(path).unwrap(),
    }
}

fn disk_snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn walk(root: &Path, directory: &Path, out: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            if path.is_dir() {
                out.insert(relative, None);
                walk(root, &path, out);
            } else {
                out.insert(relative, Some(std::fs::read(path).unwrap()));
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

fn persisted_symbol(model: &DesignModel, id: uuid::Uuid) -> Symbol {
    let object = &model.objects[&id];
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == object.source_shard_id)
        .unwrap();
    serde_json::from_value(
        model
            .materialized_source_shard_value_by_relative_path(&shard.relative_path)
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn acquisition_preview_cancel_and_refusals_leave_project_and_source_unchanged() {
    let project = Project::new();
    let model = project.resolve();
    let before = disk_snapshot(&project.work);
    let prepared = project.prepare(&model, project.request()).unwrap();
    assert_eq!(
        disk_snapshot(&project.work),
        before,
        "prepare writes nothing"
    );
    drop(prepared); // The user cancels the preview.
    assert_eq!(
        disk_snapshot(&project.work),
        before,
        "cancel writes nothing"
    );
    assert_eq!(project.resolve().model_revision, model.model_revision);
    let mut stale = project.request();
    stale.expected_source_hash = "sha256:stale".into();
    assert!(project.prepare(&model, stale).is_err());
    let mut format = project.request();
    format.source_format = "unsupported".into();
    assert!(project.prepare(&model, format).is_err());
    let mut missing = project.request();
    missing.symbol_name = "Missing".into();
    assert!(project.prepare(&model, missing).is_err());
    assert_eq!(disk_snapshot(&project.work), before);
    let stale = project.request();
    std::fs::write(&project.source, SOURCE.replace("IN+", "Changed")).unwrap();
    let after_source_edit = disk_snapshot(&project.work);
    assert!(project.prepare(&model, stale).is_err());
    assert_eq!(disk_snapshot(&project.work), after_source_edit);
}

#[test]
fn acquisition_commit_reopen_preserves_pin_identity_provenance_and_atomic_history() {
    let project = Project::new();
    let mut model = project.resolve();
    let converted = kicad_library::convert(&project.source, "Sense").unwrap();
    let ids = [converted.unit.uuid, converted.symbol.uuid];
    let source_bytes = std::fs::read(&project.source).unwrap();
    let before_objects = model.objects.clone();
    let before_map = model.import_map.clone();
    let prepared = project.prepare(&model, project.request()).unwrap();
    commit_prepared(&mut model, &project.root, prepared).unwrap();
    let reopened = project.resolve();
    let symbol = persisted_symbol(&reopened, converted.symbol.uuid);
    assert_eq!(symbol, converted.symbol);
    let pins = super::schematic_library::materialize_pool_symbol_pins(
        &reopened,
        Some(&symbol.uuid.to_string()),
    )
    .unwrap();
    assert_eq!(pins.len(), 1);
    assert_eq!(pins[0].name, "IN+");
    assert_eq!(pins[0].number, "3");
    assert_eq!(pins[0].library_pin, Some(symbol.pin_anchors[0].pin));
    assert_eq!(pins[0].position, symbol.pin_anchors[0].position);
    assert_eq!(
        pins[0].electrical_type,
        crate::schematic::PinElectricalType::Input
    );
    let provenance = symbol.provenance.as_ref().unwrap();
    assert_eq!(
        provenance.source_hash.as_deref(),
        Some(converted.source_hash.as_str())
    );
    assert!(provenance.reviewed_by.is_none());
    assert!(symbol.check_state.is_none());
    for id in ids {
        let entry = reopened
            .import_map
            .values()
            .find(|e| e.object_id == id)
            .expect("both acquired objects have source identity");
        assert_eq!(entry.source_hash, converted.source_hash);
        assert!(entry.source_path.ends_with("local.kicad_sym"));
        assert!(entry.source_tool.contains("kicad"));
    }
    let committed_map = reopened.import_map.clone();
    let committed_disk = disk_snapshot(&project.work);
    assert!(
        project.prepare(&reopened, project.request()).is_err(),
        "existing identities are not silently updated"
    );
    assert_eq!(disk_snapshot(&project.work), committed_disk);
    model
        .commit_journal_undo(&project.root, self::provenance().into())
        .unwrap();
    let undone = project.resolve();
    for id in ids {
        assert!(!undone.objects.contains_key(&id));
    }
    assert_eq!(undone.objects, before_objects);
    assert_eq!(undone.import_map, before_map);
    model
        .commit_journal_redo(&project.root, self::provenance().into())
        .unwrap();
    let redone = project.resolve();
    assert_eq!(persisted_symbol(&redone, symbol.uuid), symbol);
    assert_eq!(redone.import_map, committed_map);
    assert_eq!(std::fs::read(&project.source).unwrap(), source_bytes);
}

#[test]
#[ignore = "requires installed local KiCad libraries; explicitly run for canonical real-source proof"]
fn installed_real_components_acquire_through_canonical_commit() {
    let project = Project::new();
    let mut model = project.resolve();
    for (library, name, count) in [("Device", "R", 2), ("Amplifier_Operational", "LM741", 8)] {
        let source = PathBuf::from(format!("/usr/share/kicad/symbols/{library}.kicad_sym"));
        let converted = kicad_library::convert(&source, name).unwrap();
        let prepared = project.prepare(&model, request(&source, name)).unwrap();
        commit_prepared(&mut model, &project.root, prepared).unwrap();
        let reopened = project.resolve();
        assert_eq!(
            persisted_symbol(&reopened, converted.symbol.uuid),
            converted.symbol
        );
        let pins = super::schematic_library::materialize_pool_symbol_pins(
            &reopened,
            Some(&converted.symbol.uuid.to_string()),
        )
        .unwrap();
        assert_eq!(pins.len(), count);
        if name == "LM741" {
            assert_eq!(pins.iter().find(|p| p.number == "3").unwrap().name, "+");
        }
    }
}
