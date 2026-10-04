//! Canonical source/evidence adoption proof; no renderer-generated copper.
use super::*;
use eda_engine::board::occupied_copper::polygon_contains_point;
use eda_engine::ir::geometry::Point;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, ProjectResolver, SourceShardKind, ZoneFillState,
};

fn setup(name: &str, invalid: bool) -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("imported-zone-fill-{name}-{}", Uuid::new_v4()));
    create_native_project(&root, Some(name.into())).unwrap();
    let source = root.join("source.kicad_pcb");
    let points = if invalid {
        "(xy 0 0) (xy 10 10) (xy 0 10) (xy 10 0)"
    } else {
        "(xy 0 0) (xy 10 0) (xy 10 10) (xy 0 10) (xy 0 0) (xy 2 2) (xy 2 4) (xy 4 4) (xy 4 2) (xy 2 2)"
    };
    std::fs::write(
        &source,
        format!(
            r#"(kicad_pcb
  (version 20221018)
  (layers
    (0 "F.Cu" signal)
    (31 "B.Cu" signal)
    (44 "Edge.Cuts" user))
  (net 1 "GND")
  (footprint "Demo:Pad"
    (layer "F.Cu")
    (uuid "22222222-2222-2222-2222-222222222222")
    (at 40 40)
    (property "Reference" "P1" (at 0 0 0))
    (pad "1" smd rect (at 0 0) (size 1 1) (layers "F.Cu") (net 1 "GND") (uuid "33333333-3333-3333-3333-333333333333")))
  (zone
    (net 1)
    (net_name "GND")
    (layer "F.Cu")
    (uuid "11111111-1111-1111-1111-111111111111")
    (polygon (pts (xy 0 0) (xy 30 0) (xy 30 10) (xy 0 10)))
    (filled_polygon
      (layer "F.Cu")
      (pts {points}))
    (filled_polygon
      (layer "F.Cu")
      (pts (xy 20 0) (xy 22 0) (xy 22 2) (xy 20 2))))
)"#
        ),
    )
    .unwrap();
    (root, source)
}
fn import(root: &Path, source: &Path) -> anyhow::Result<String> {
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
fn check(root: &Path) -> Uuid {
    let model = ProjectResolver::new(root).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    assert_eq!(
        board["zones"].as_object().unwrap().len(),
        1,
        "islands share one authored Zone"
    );
    let id: Uuid = board["zones"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(
        board["zones"][id.to_string()]["polygon"]["vertices"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    let fill = model.current_zone_fill(id).unwrap_or_else(|| {
        panic!(
            "fill {:?}; model {:?}; source {:?}; diagnostics {:?}",
            model.zone_fills.get(&id),
            model.model_revision,
            model.objects.get(&id),
            model.diagnostics
        )
    });
    assert!(fill.islands.len() > 2);
    assert!(
        fill.islands
            .iter()
            .all(|p| !polygon_contains_point(p, Point::new(3_000_000, 3_000_000)).unwrap()),
        "hole remains void"
    );
    for point in [(8_000_000, 8_000_000), (21_000_000, 1_000_000)] {
        assert!(
            fill.islands
                .iter()
                .any(|p| polygon_contains_point(p, Point::new(point.0, point.1)).unwrap())
        );
    }
    let snapshot = eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
    let origin = eda_engine::substrate::ElectricalOccurrence {
        class: "zones".into(),
        source_id: id,
        instance_path: vec![],
    };
    let first = snapshot
        .board_run(
            &model.model_revision,
            &origin,
            Some(Point::new(8_000_000, 8_000_000)),
        )
        .unwrap();
    let second = snapshot
        .board_run(
            &model.model_revision,
            &origin,
            Some(Point::new(21_000_000, 1_000_000)),
        )
        .unwrap();
    assert_eq!(first.members, [origin.clone()].into_iter().collect());
    assert_eq!(second.members, first.members);
    assert_ne!(
        first.zone_regions, second.zone_regions,
        "disconnected copper has separate captured components"
    );
    assert!(
        snapshot
            .board_run(
                &model.model_revision,
                &origin,
                Some(Point::new(3_000_000, 3_000_000))
            )
            .is_err(),
        "hole cannot acquire copper"
    );
    id
}
#[test]
fn saved_fill_adoption_is_complete_atomic_and_survives_history() {
    let (root, source) = setup("history", false);
    import(&root, &source).unwrap();
    let id = check(&root);
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let tx = model.journal.last().unwrap();
    assert!(
        tx.operations
            .iter()
            .any(|op| matches!(op, eda_engine::substrate::Operation::CreateBoardZone { .. }))
    );
    assert!(
        tx.operations
            .iter()
            .any(|op| matches!(op, eda_engine::substrate::Operation::SetZoneFill { .. }))
    );
    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "test".into(),
                source: CommitSource::Test,
                reason: "undo import".into(),
            },
        )
        .unwrap();
    let undone = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        !undone.objects.contains_key(&id),
        "diagnostics: {:?}; zone: {:?}",
        undone.diagnostics,
        undone.objects.get(&id)
    );
    let mut reopened = ProjectResolver::new(&root).resolve().unwrap();
    reopened
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "test".into(),
                source: CommitSource::Test,
                reason: "redo import".into(),
            },
        )
        .unwrap();
    assert_eq!(check(&root), id);
    let before = ProjectResolver::new(&root).resolve().unwrap();
    import(&root, &source).unwrap();
    assert_eq!(check(&root), id);
    let after = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(before.model_revision, after.model_revision);
    assert_eq!(
        before.journal.len(),
        after.journal.len(),
        "unchanged reimport is a true no-op"
    );
    // Rebinding must not grant changed geometry or an arbitrary revision.
    let mut journal = after.journal;
    let last = journal.last_mut().unwrap();
    for op in &mut last.operations {
        if let eda_engine::substrate::Operation::SetZoneFill { zone_fill, .. } = op {
            zone_fill["model_revision"] = serde_json::json!("forged");
        }
    }
    let bytes = journal
        .iter()
        .map(|tx| format!("{}\n", serde_json::to_string(tx).unwrap()))
        .collect::<String>();
    std::fs::write(root.join(".datum/journal/transactions.jsonl"), bytes).unwrap();
    let refused = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        refused
            .diagnostics
            .iter()
            .any(|d| d.code == "journal_link_mismatch")
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn malformed_saved_fill_refuses_before_any_native_adoption() {
    let (root, source) = setup("invalid", true);
    let before = std::fs::read(root.join("board/board.json")).unwrap();
    let revision = ProjectResolver::new(&root)
        .resolve()
        .unwrap()
        .model_revision;
    assert!(import(&root, &source).is_err());
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        before
    );
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(model.model_revision, revision);
    assert!(
        model
            .zone_fills
            .values()
            .all(|f| f.state == ZoneFillState::Unfilled)
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_saved_fill_stays_unfilled() {
    let (root, source) = setup("missing", false);
    let text = std::fs::read_to_string(&source).unwrap();
    let start = text.find("    (filled_polygon").unwrap();
    std::fs::write(&source, format!("{}\n  )\n)", &text[..start])).unwrap();
    import(&root, &source).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(model.zone_fills.len(), 1);
    let count = model.journal.len();
    import(&root, &source).unwrap();
    assert_eq!(
        ProjectResolver::new(&root).resolve().unwrap().journal.len(),
        count
    );

    assert!(
        model
            .zone_fills
            .values()
            .all(|f| f.state == ZoneFillState::Unfilled && f.islands.is_empty())
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_edit_invalidates_saved_fill_and_reimport_cannot_revive_it() {
    let (root, source) = setup("stale", false);
    import(&root, &source).unwrap();
    let id = check(&root);
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let mut zone: eda_engine::board::Zone =
        serde_json::from_value(board["zones"][id.to_string()].clone()).unwrap();
    zone.priority += 1;
    let prepared = eda_engine::api::native_write::board_routing::build_set_board_zone(
        &model,
        eda_engine::api::native_write::WriteProvenance::new(
            "test",
            CommitSource::Test,
            "change Zone priority",
        ),
        &zone,
    )
    .unwrap();
    eda_engine::api::native_write::commit_prepared(&mut model, &root, prepared).unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.zone_fills[&id].state, ZoneFillState::Stale);
    let revision = reopened.model_revision;
    let before = std::fs::read(root.join("board/board.json")).unwrap();
    assert!(
        import(&root, &source).is_err(),
        "edited source is not a legacy migration"
    );
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(reopened.model_revision, revision);
    assert_eq!(reopened.zone_fills[&id].state, ZoneFillState::Stale);
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn proven_legacy_islands_migrate_without_changing_authored_identity() {
    use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
    use eda_engine::substrate::{ImportMapEntry, ImportMapEntryStatus};
    let (root, source) = setup("migration", false);
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::BoardRoot)
        .unwrap();
    let board_id = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap()["uuid"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let (mut board, _, identities) =
        eda_engine::import::kicad::import_board_document_with_import_map_identities(
            &source,
            &model.import_map,
        )
        .unwrap();
    assert_eq!(board.zones.len(), 2);
    for pad in board.pads.values_mut() {
        pad.layer_connection = eda_engine::board::PadLayerConnection::Unknown;
    }
    let hash = eda_engine::import::ids_sidecar::compute_source_hash_file(&source).unwrap();
    let entries = identities
        .iter()
        .map(|identity| ImportMapEntry {
            import_key: identity.import_key.clone(),
            object_id: identity.object_id,
            source_shard_id: shard.shard_id,
            status: ImportMapEntryStatus::Active,
            source_tool: "kicad".into(),
            source_path: source.display().to_string(),
            source_object_ref: "legacy fixture".into(),
            source_hash: hash.clone(),
        })
        .collect();
    let original = identities
        .iter()
        .find(|i| i.object_family == "board_zone")
        .unwrap()
        .object_id;
    let write = eda_engine::api::native_write::imports::build_kicad_board_import(
        &model,
        WriteProvenance::new("test", CommitSource::Test, "legacy import"),
        board_id,
        &board,
        entries,
        &source,
    )
    .unwrap();
    commit_prepared(&mut model, &root, write.prepared.unwrap()).unwrap();
    import(&root, &source).unwrap();
    assert_eq!(check(&root), original);
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let adopted = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    for pad in board.pads.values() {
        let mut expected = pad.clone();
        expected.layer_connection = eda_engine::board::PadLayerConnection::Separate;
        assert_eq!(
            adopted["pads"][pad.uuid.to_string()],
            serde_json::to_value(expected).unwrap()
        );
    }
    for retired in board.zones.keys().filter(|id| **id != original) {
        assert!(!model.objects.contains_key(retired));
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn authored_pad_connection_change_is_not_a_converter_migration() {
    use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
    let (root, source) = setup("edited-pad", false);
    import(&root, &source).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let mut pad: eda_engine::board::PlacedPad = serde_json::from_value(
        board["pads"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap()
            .clone(),
    )
    .unwrap();
    pad.layer_connection = eda_engine::board::PadLayerConnection::Unknown;
    let write = eda_engine::api::native_write::board_routing::build_set_board_pad(
        &model,
        WriteProvenance::new("test", CommitSource::Test, "explicit pad connection edit"),
        &pad,
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = std::fs::read(root.join("board/board.json")).unwrap();
    let revision = model.model_revision;
    assert!(import(&root, &source).is_err());
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        before
    );
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        revision
    );
    std::fs::remove_dir_all(root).unwrap();
}
