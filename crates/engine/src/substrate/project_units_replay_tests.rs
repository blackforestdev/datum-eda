//! Units initialization remains replayable across actual native Symbol births.
use crate::api::native_write::{
    WriteProvenance, commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
    registry::{NativeWriteContext, find_native_write_verb},
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver, SourceShardKind};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Native alignment App proof")
}
fn write(model: &mut DesignModel, root: &Path, verb: &str, params: Value) -> Uuid {
    // Match the ordinary writer companion: resolve each request from persisted source.
    *model = ProjectResolver::new(root).resolve().unwrap();
    let p = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = p.primary_object_id.unwrap_or_default();
    commit_prepared(model, root, p).unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
    id
}
fn setup(root: &Path) -> (Vec<Uuid>, PathBuf) {
    bootstrap_native_project(
        root,
        GenesisSpec {
            project_name: "Units replay".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    let crate::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        crate::ir::units::migrate_pre_feature_project_units(None).unwrap()
    else {
        panic!("Fresh fixture requires the canonical units migration");
    };
    let initialize = crate::api::native_write::project::build_initialize_project_display_units(
        &model,
        provenance(),
        profile,
        &receipt,
    )
    .unwrap();
    commit_prepared(&mut model, root, initialize).unwrap();
    model = ProjectResolver::new(root).resolve().unwrap();
    let (unit, pin, sheet) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    write(
        &mut model,
        root,
        "datum.library.create",
        json!({"kind":"units","pool_path":"pool","object_id":unit,"fields":{"name":"Passive","pins":{pin.to_string():{"name":"Signal","electrical_type":"Passive"}}}}),
    );
    write(
        &mut model,
        root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    let mut symbols = Vec::new();
    for (i, (x, y, left, right)) in [
        (0, 0, -1000000, 1000000),
        (10000000, 4000000, -3000000, 2000000),
    ]
    .into_iter()
    .enumerate()
    {
        let library = Uuid::new_v4();
        write(
            &mut model,
            root,
            "datum.library.create",
            json!({"kind":"symbols","pool_path":"pool","object_id":library,"fields":{"name":format!("Body{i}"),"unit":unit,"drawings":[{"Rect":{"min":{"x":left,"y":-500000},"max":{"x":right,"y":500000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}}),
        );
        symbols.push(write(&mut model, root, "datum.schematic.place_symbol", json!({"sheet":sheet,"instance_path":[],"library_symbol":library,"reference":format!("U{}",i+1),"value":"A long caption does not define body bounds","x_nm":x,"y_nm":y})));
    }
    write(
        &mut model,
        root,
        "datum.schematic.draw_wire",
        json!({"sheet":sheet,"instance_path":[],"points":[{"x":-2000000,"y":0},{"x":8000000,"y":4000000}]}),
    );
    let path = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::SchematicSheet)
        .unwrap()
        .path
        .clone();
    (symbols, path)
}

fn temp_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("datum-{name}-{}", Uuid::new_v4()))
}

fn physical_count(root: &Path) -> usize {
    std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .unwrap()
        .lines()
        .count()
}
fn cold(root: &Path) -> DesignModel {
    let model = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(
        model.journal.len(),
        physical_count(root),
        "Every persisted transaction must remain loaded"
    );
    assert_eq!(
        model.journal_cursor.applied_transaction_count,
        model.journal.len()
    );
    assert!(
        !model
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("journal_")),
        "{:?}",
        model.diagnostics
    );
    model
}
fn preimage(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, at: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().into(),
                    std::fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn initialized_units_survive_symbol_birth_later_writes_updates_and_cold_history() {
    use crate::api::native_write::{
        history::{HistoryGuards, commit_history},
        project,
    };
    use crate::ir::units::{LengthUnit, LengthUnitChoice};
    let root = temp_root("units_symbol_birth_replay");
    setup(&root);
    let mut model = cold(&root);
    assert_eq!(model.journal.len(), 8);
    write(
        &mut model,
        &root,
        "datum.project.set_name",
        json!({"name":"After source births"}),
    );
    model = cold(&root);
    let original = project::project_display_units(&model).unwrap();
    let receipt = model.project.project_units_seed_receipt.clone();
    let mut changed = original;
    changed.schematic.unit = LengthUnitChoice::Explicit(LengthUnit::Mil);
    let update = project::build_set_project_display_units(&model, provenance(), changed).unwrap();
    commit_prepared(&mut model, &root, update).unwrap();
    model = cold(&root);
    assert_eq!(project::project_display_units(&model).unwrap(), changed);
    assert_eq!(model.project.project_units_seed_receipt, receipt);
    let history_tip = |model: &DesignModel| HistoryGuards {
        expected_tip: crate::revision::transaction_tip(&model.journal).unwrap(),
    };
    let guards = history_tip(&model);
    commit_history(&mut model, &root, guards, false, provenance()).unwrap();
    model = cold(&root);
    assert_eq!(project::project_display_units(&model).unwrap(), original);
    let guards = history_tip(&model);
    commit_history(&mut model, &root, guards, true, provenance()).unwrap();
    model = cold(&root);
    assert_eq!(project::project_display_units(&model).unwrap(), changed);
    assert_eq!(model.project.project_units_seed_receipt, receipt);
    assert_eq!(model.journal.len(), 12);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn repeated_live_initialization_refuses_in_both_builder_and_raw_batch_without_writes() {
    use crate::api::native_write::{context::BatchComposer, project};
    use crate::substrate::Operation;
    let root = temp_root("units_duplicate_live_initialization");
    setup(&root);
    let mut model = cold(&root);
    let before = preimage(&root);
    let crate::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        crate::ir::units::migrate_pre_feature_project_units(None).unwrap()
    else {
        unreachable!()
    };
    let error =
        project::build_initialize_project_display_units(&model, provenance(), profile, &receipt)
            .unwrap_err();
    assert!(error.to_string().contains("already initialized"), "{error}");
    assert_eq!(preimage(&root), before);
    let raw: Value =
        serde_json::from_slice(&std::fs::read(root.join("project.json")).unwrap()).unwrap();
    let prepared = BatchComposer::compose(&model, provenance())
        .push_op(Operation::InitializeProjectDisplayUnits {
            project_id: model.project.project_id,
            profile: raw["project_display_units"].clone(),
            receipt: raw["project_units_seed_receipt"].clone(),
        })
        .finish()
        .unwrap();
    let error = commit_prepared(&mut model, &root, prepared).unwrap_err();
    assert!(error.to_string().contains("already initialized"), "{error}");
    assert_eq!(
        preimage(&root),
        before,
        "Raw batch cannot bypass live one-time admission"
    );
    cold(&root);
    std::fs::remove_dir_all(root).unwrap();
}
