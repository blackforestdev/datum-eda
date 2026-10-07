use super::*;
use crate::api::native_write::board_layout::source_profile::build_adopt_board_source_profile;
use crate::api::native_write::{commit_prepared, test_support};
use crate::board::graphics::GraphicGeometry;
use crate::ir::geometry::Point;
use crate::substrate::{CommitProvenance, CommitSource, ProjectResolver};
use uuid::Uuid;

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "graphic-test",
        CommitSource::Test,
        "PM057 source history proof",
    )
}
fn history() -> CommitProvenance {
    provenance().into()
}
fn fixture(present: bool) -> (std::path::PathBuf, DesignModel, Uuid) {
    let (root, _, board_id, _) = test_support::resolved_model_with_board_package("graphics");
    let path = root.join("board/board.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value["stackup"] = serde_json::json!({"layers":[
        {"id": 37, "name":"F.SilkS", "layer_type":"Silkscreen", "thickness_nm":0},
        {"id": 49, "name":"F.Fab", "layer_type":"Mechanical", "thickness_nm":0},
        {"id": 0, "name":"F.Cu", "layer_type":"Copper", "thickness_nm":35000}
    ]});
    std::fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let adoption = build_adopt_board_source_profile(&model, provenance(), board_id).unwrap();
    commit_prepared(&mut model, &root, adoption).unwrap();
    if present {
        let seed = graphic(GraphicGeometry::Line {
            from: Point::new(90, 90),
            to: Point::new(100, 90),
        });
        let write = build_create_board_graphic(&model, provenance(), board_id, &seed).unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        let write = build_delete_board_graphic(&model, provenance(), board_id, seed.uuid).unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }
    (root, model, board_id)
}
fn graphic(geometry: GraphicGeometry) -> BoardGraphic {
    BoardGraphic {
        uuid: Uuid::new_v4(),
        layer: 37,
        width: 2,
        geometry,
        import_key: None,
    }
}
fn variants() -> Vec<GraphicGeometry> {
    let p = Point::new;
    vec![
        GraphicGeometry::Line {
            from: p(0, 0),
            to: p(20, 0),
        },
        GraphicGeometry::Polyline {
            points: vec![p(0, 0), p(10, 20), p(20, 0)],
        },
        GraphicGeometry::Arc {
            from: p(0, 0),
            midpoint: p(10, 10),
            to: p(20, 0),
        },
        GraphicGeometry::Rectangle {
            from: p(0, 0),
            to: p(20, 20),
            filled: true,
        },
        GraphicGeometry::Circle {
            center: p(10, 10),
            radius: 10,
            filled: true,
        },
        GraphicGeometry::Polygon {
            outer: vec![p(0, 0), p(20, 0), p(20, 20), p(0, 20)],
            holes: vec![vec![p(5, 5), p(10, 5), p(10, 10), p(5, 10)]],
            filled: true,
        },
    ]
}

#[test]
fn every_variant_restores_absent_empty_and_populated_map_bytes_through_reopened_history() {
    for presence in 0..3 {
        for geometry in variants() {
            let (root, mut model, board_id) = fixture(presence == 1);
            if presence == 2 {
                let existing = graphic(GraphicGeometry::Line {
                    from: Point::new(90, 90),
                    to: Point::new(100, 90),
                });
                let write =
                    build_create_board_graphic(&model, provenance(), board_id, &existing).unwrap();
                commit_prepared(&mut model, &root, write).unwrap();
            }
            let path = root.join("board/board.json");
            let before = std::fs::read(&path).unwrap();
            let imports = model.import_map.clone();
            let unaffected: Vec<_> = model
                .source_shards
                .iter()
                .filter(|s| s.kind != SourceShardKind::BoardRoot)
                .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
                .collect();
            let graphic = graphic(geometry);
            let write =
                build_create_board_graphic(&model, provenance(), board_id, &graphic).unwrap();
            commit_prepared(&mut model, &root, write).unwrap();
            assert_eq!(model.objects[&graphic.uuid].domain, "board");
            let created = std::fs::read(&path).unwrap();
            let journal =
                std::fs::read(root.join(crate::substrate::JOURNAL_RELATIVE_PATH)).unwrap();
            model = reopened(&root, &model);
            assert_eq!(
                serde_json::from_value::<BoardGraphic>(
                    model
                        .materialized_source_shard_value(SourceShardKind::BoardRoot)
                        .unwrap()["graphics"][graphic.uuid.to_string()]
                    .clone()
                )
                .unwrap(),
                graphic
            );
            model.commit_journal_undo(&root, history()).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), before);
            assert!(!model.objects.contains_key(&graphic.uuid));
            assert_ne!(
                std::fs::read(root.join(crate::substrate::JOURNAL_RELATIVE_PATH)).unwrap(),
                journal
            );
            model = reopened(&root, &model);
            assert_eq!(std::fs::read(&path).unwrap(), before);
            model.commit_journal_redo(&root, history()).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), created);
            model = reopened(&root, &model);
            let mut changed = graphic.clone();
            changed.width = 5;
            let write = build_set_board_graphic(&model, provenance(), board_id, &changed).unwrap();
            commit_prepared(&mut model, &root, write).unwrap();
            let edited = std::fs::read(&path).unwrap();
            model = reopened(&root, &model);
            model.commit_journal_undo(&root, history()).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), created);
            model.commit_journal_redo(&root, history()).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), edited);
            let write =
                build_delete_board_graphic(&model, provenance(), board_id, graphic.uuid).unwrap();
            commit_prepared(&mut model, &root, write).unwrap();
            let deleted = std::fs::read(&path).unwrap();
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&deleted).unwrap()["graphics"]
                    .as_object()
                    .unwrap()
                    .len(),
                usize::from(presence == 2)
            );
            model = reopened(&root, &model);
            model.commit_journal_undo(&root, history()).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), edited);
            model.commit_journal_redo(&root, history()).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), deleted);
            assert_eq!(model.import_map, imports);
            for (p, bytes) in &unaffected {
                assert_eq!(&std::fs::read(p).unwrap(), bytes);
            }
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}

#[test]
fn invalid_identity_layer_geometry_presence_and_noncanonical_writes_are_atomic() {
    let (root, mut model, board_id) = fixture(false);
    let path = root.join("board/board.json");
    let before = std::fs::read(&path).unwrap();
    let journal_path = root.join(crate::substrate::JOURNAL_RELATIVE_PATH);
    let journal = std::fs::read(&journal_path).unwrap();
    let stages = || {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(".datum/stage"))
            .unwrap()
            .map(|p| p.unwrap().file_name())
            .collect();
        paths.sort();
        paths
    };
    let stage_before = stages();
    let base = graphic(variants().remove(0));
    let mut invalid = base.clone();
    invalid.layer = 0;
    let mut degenerate = base.clone();
    degenerate.geometry = GraphicGeometry::Arc {
        from: Point::new(0, 0),
        midpoint: Point::new(1, 0),
        to: Point::new(2, 0),
    };
    let mut nil = base.clone();
    nil.uuid = Uuid::nil();
    let mut unbound = base.clone();
    unbound.import_key = Some("unrecorded-import".into());
    for invalid in [invalid, degenerate, nil, unbound] {
        let write = build_create_board_graphic(&model, provenance(), board_id, &invalid).unwrap();
        assert!(commit_prepared(&mut model, &root, write).is_err());
    }
    let mut wrong_presence =
        build_create_board_graphic(&model, provenance(), board_id, &base).unwrap();
    if let Some(Operation::CreateBoardGraphic {
        map_was_present, ..
    }) = wrong_presence.batch.operations.last_mut()
    {
        *map_was_present = true;
    }
    assert!(commit_prepared(&mut model, &root, wrong_presence).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(std::fs::read(&journal_path).unwrap(), journal);
    assert_eq!(
        stages(),
        stage_before,
        "refusal must precede stage artifact creation"
    );
    let mut noncanonical = before.clone();
    noncanonical.push(b'\n');
    std::fs::write(&path, &noncanonical).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let write = build_create_board_graphic(&model, provenance(), board_id, &base).unwrap();
    assert!(commit_prepared(&mut model, &root, write).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), noncanonical);
    assert_eq!(std::fs::read(&journal_path).unwrap(), journal);
    std::fs::remove_dir_all(root).unwrap();
}

fn reopened(root: &std::path::Path, expected: &DesignModel) -> DesignModel {
    let model = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(
        model.journal.len(),
        expected.journal.len(),
        "{:?}",
        model.diagnostics
    );
    assert_eq!(
        model.model_revision, expected.model_revision,
        "{:?}",
        model.diagnostics
    );
    model
}

#[test]
fn stackup_edit_cannot_turn_registered_documentation_graphics_into_copper() {
    let (root, mut model, board_id) = fixture(false);
    let g = graphic(variants().remove(0));
    let write = build_create_board_graphic(&model, provenance(), board_id, &g).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let path = root.join("board/board.json");
    let before = std::fs::read(&path).unwrap();
    let journal_path = root.join(crate::substrate::JOURNAL_RELATIVE_PATH);
    let journal = std::fs::read(&journal_path).unwrap();
    let write = crate::api::native_write::board_layout::build_set_board_stackup(
        &model,
        provenance(),
        board_id,
        &[crate::board::StackupLayer::new(
            37,
            "F.Cu",
            crate::board::StackupLayerType::Copper,
            35_000,
        )],
    )
    .unwrap();
    assert!(commit_prepared(&mut model, &root, write).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(std::fs::read(&journal_path).unwrap(), journal);
    std::fs::remove_dir_all(root).unwrap();
}
