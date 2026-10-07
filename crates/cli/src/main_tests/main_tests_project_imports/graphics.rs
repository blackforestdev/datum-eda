//! Reuse the existing routed-board/saved-fill fixture through the actual CLI.
use super::*;
use eda_engine::substrate::{ProjectResolver, SourceShardKind, ZoneFillState};
struct OwnedRoot(PathBuf);
impl Drop for OwnedRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn setup() -> (OwnedRoot, PathBuf) {
    let root = unique_project_root("datum-cli-native-graphics");
    create_native_project(&root, Some("Native graphics".into())).unwrap();
    let source = write_temp_board(&root, "input.kicad_pcb");
    let mut contents = std::fs::read_to_string(&source).unwrap().replace(
        "(44 \"Edge.Cuts\" user)",
        "(37 \"F.SilkS\" user)\n    (49 \"F.Fab\" user)\n    (44 \"Edge.Cuts\" user)",
    );
    let at = contents.rfind(')').unwrap();
    let records = [
        ("gr_line", "(start 0 0) (end 2 0)"),
        ("gr_arc", "(start 0 0) (mid 1 1) (end 2 0)"),
        ("gr_rect", "(start 0 0) (end 2 3) (fill solid)"),
        ("gr_circle", "(center 0 0) (end 3 4) (fill none)"),
        (
            "gr_poly",
            "(pts (xy 0 0) (xy 2 0) (xy 2 3) (xy 0 3)) (fill solid)",
        ),
    ];
    let mut graphics = String::new();
    for (i, (form, body)) in records.into_iter().enumerate() {
        graphics.push_str(&format!("\n  ({form} {body} (stroke (width 0.1) (type default)) (layer \"F.SilkS\") (uuid {}))\n",Uuid::from_u128(100+i as u128)));
    }
    contents.insert_str(at, &graphics);
    std::fs::write(&source, contents).unwrap();
    (OwnedRoot(root), source)
}
fn import(root: &Path, source: &Path) -> Result<String> {
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "import-kicad-board",
            root.to_str().unwrap(),
            "--source",
            source.to_str().unwrap(),
        ])
        .unwrap(),
    )
}
#[test]
fn project_import_native_graphics_adopts_profile_separately_and_preserves_saved_copper_basis() {
    let (root, source) = setup();
    let output = import(&root.0, &source).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["imported_graphic_count"], 5);
    assert!(report["profile_adoption_transaction_id"].as_str().is_some());
    assert_eq!(
        report["graphic_dispositions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["disposition"] == "outline_contribution")
            .count(),
        4,
        "historical identifier-less outline contributors are not new graphics"
    );
    let model = ProjectResolver::new(&root.0).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    assert_eq!(board["schema_version"], 4);
    assert_eq!(board["graphics"].as_object().unwrap().len(), 5);
    assert_eq!(model.zone_fills.len(), 1);
    assert_eq!(
        model.journal.last().unwrap().after_model_revision,
        model.model_revision,
        "complete reopened journal tip"
    );
    for fill in model.zone_fills.values() {
        assert_eq!(fill.state, ZoneFillState::Filled);
        assert_eq!(
            fill.model_revision, model.model_revision,
            "saved fill belongs to the complete graphics-inclusive batch"
        );
    }
    let adoption: Vec<_> = model
        .journal
        .iter()
        .filter(|t| {
            t.operations.iter().any(|o| {
                matches!(
                    o,
                    eda_engine::substrate::Operation::AdoptBoardSourceProfile { .. }
                )
            })
        })
        .collect();
    assert_eq!(adoption.len(), 1);
    assert!(adoption[0].operations.iter().all(|o| matches!(
        o,
        eda_engine::substrate::Operation::AdoptBoardSourceProfile { .. }
            | eda_engine::substrate::Operation::GuardObjectRevision { .. }
    )));
    for n in 100..105 {
        let key = eda_engine::import::kicad::board_graphic_import_key(&source, Uuid::from_u128(n));
        let entry = &model.import_map[&key];
        assert_eq!(
            board["graphics"][entry.object_id.to_string()]["import_key"],
            key
        );
        assert!(model.objects.contains_key(&entry.object_id));
    }
    let source_bytes: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let output = import(&root.0, &source).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["created_object_count"], 0);
    assert_eq!(
        report["profile_adoption_transaction_id"],
        serde_json::Value::Null
    );
    let after = ProjectResolver::new(&root.0).resolve().unwrap();
    assert_eq!(after.journal.len(), model.journal.len());
    assert_eq!(after.model_revision, model.model_revision);
    for (path, bytes) in source_bytes {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
#[test]
fn project_import_unsupported_graphic_refuses_before_profile_adoption_or_source_writes() {
    let (root, source) = setup();
    let contents = std::fs::read_to_string(&source)
        .unwrap()
        .replace("(end 3 4)", "(end 1 1)");
    std::fs::write(&source, contents).unwrap();
    let before = ProjectResolver::new(&root.0).resolve().unwrap();
    let bytes: Vec<_> = before
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let error = import(&root.0, &source).unwrap_err().to_string();
    assert!(
        error.contains("graphic adoption refused before mutation"),
        "{error}"
    );
    assert!(error.contains("non-integer graphic radius"), "{error}");
    let after = ProjectResolver::new(&root.0).resolve().unwrap();
    assert_eq!(after.journal.len(), before.journal.len());
    assert_eq!(after.model_revision, before.model_revision);
    for (path, data) in bytes {
        assert_eq!(std::fs::read(path).unwrap(), data);
    }
}

#[path = "graphics/manufacturing.rs"]
mod manufacturing;
