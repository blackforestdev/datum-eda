use super::*;
use crate::board::{Stackup, StackupLayer};
use crate::ir::geometry::Point;

fn board() -> Board {
    let (mut board, _) = super::super::skeleton::parse_board_skeleton(
        Path::new("graphics.kicad_pcb"),
        "(kicad_pcb)\n",
        None,
        None,
    )
    .unwrap();
    board.stackup = Stackup {
        layers: vec![
            StackupLayer::new(37, "F.SilkS", StackupLayerType::Silkscreen, 0),
            StackupLayer::new(49, "F.Fab", StackupLayerType::Mechanical, 0),
            StackupLayer::new(0, "F.Cu", StackupLayerType::Copper, 35000),
        ],
    };
    board
}
fn source(form: &str, body: &str, uuid: Uuid) -> String {
    format!(
        "(kicad_pcb ({form} {body} (stroke (width 0.1) (type default)) (layer \"F.SilkS\") (uuid {uuid})))"
    )
}
#[test]
fn exact_graphic_import_reuses_uuid_qualified_map_and_accounts_outline_once() {
    let path = Path::new("graphics.kicad_pcb");
    let source_uuid = Uuid::from_u128(1);
    let inputs = [
        ("gr_line", "(start 0 0) (end 2 0)"),
        ("gr_arc", "(start 0 0) (mid 1 1) (end 2 0)"),
        ("gr_rect", "(start 0 0) (end 2 3) (fill solid)"),
        ("gr_circle", "(center 0 0) (end 3 4) (fill none)"),
        (
            "gr_poly",
            "(pts (xy 0 0) (xy 2 0) (xy 2 3) (xy 0 3)) (fill solid)",
        ),
    ];
    let board = board();
    for (form, body) in inputs {
        let contents = source(form, body, source_uuid);
        let plan = parse(path, &contents, &BTreeMap::new(), &board).unwrap();
        assert_eq!(plan.graphics.len(), 1);
        assert_eq!(plan.identities[0].source_uuid, source_uuid);
        assert_eq!(plan.dispositions[0].disposition, "supported");
        let key = board_graphic_import_key(path, source_uuid);
        let assigned = Uuid::from_u128(999);
        let entry = ImportMapEntry {
            import_key: key.clone(),
            object_id: assigned,
            source_shard_id: Uuid::from_u128(2),
            status: ImportMapEntryStatus::Active,
            source_tool: "kicad".into(),
            source_path: path.display().to_string(),
            source_object_ref: format!("board-graphic:{source_uuid}"),
            source_hash: "historical".into(),
        };
        let map = BTreeMap::from([(key, entry)]);
        let reused = parse(path, &contents, &map, &board).unwrap();
        assert_eq!(
            reused.graphics[0].uuid, assigned,
            "engine map allocation wins over source UUID"
        );
        if form == "gr_circle" {
            assert_eq!(
                reused.graphics[0].geometry,
                G::Circle {
                    center: Point::new(0, 0),
                    radius: 5_000_000,
                    filled: false
                }
            );
        }
        if form == "gr_arc" {
            assert_eq!(
                reused.graphics[0].geometry,
                G::Arc {
                    from: Point::new(0, 0),
                    midpoint: Point::new(1_000_000, 1_000_000),
                    to: Point::new(2_000_000, 0)
                }
            );
        }
    }
    let contents =
        source("gr_line", "(start 0 0) (end 2 0)", source_uuid).replace("F.SilkS", "Edge.Cuts");
    let plan = parse(path, &contents, &BTreeMap::new(), &board).unwrap();
    assert!(plan.graphics.is_empty());
    assert!(plan.identities.is_empty());
    assert_eq!(plan.dispositions[0].disposition, "pending_outline");
}
#[test]
fn ambiguous_unsupported_or_inexact_graphic_source_refuses_with_complete_report() {
    let uuid = Uuid::from_u128(1);
    let valid = source("gr_line", "(start 0 0) (end 2 0)", uuid);
    for invalid in [
        valid.replace(&format!("(uuid {uuid})"), ""),
        valid.replace("F.SilkS", "F.Cu"),
        valid.replace("(end 2 0)", "(end 0 0)"),
        valid.replace("(start 0 0)", "(start 0.0000001 0)"),
        valid.replace("(type default)", "(type dash)"),
        valid.replace("(type default)", "(type default) (color 1 0 0 1)"),
        valid.replace("(end 2 0)", "(end 2 0) (fill solid)"),
        source("gr_circle", "(center 0 0) (end 1 1)", uuid),
        source("gr_curve", "(start 0 0) (end 2 0)", uuid),
        valid.replace("(end 2 0)", "(end 2 0) (net 1)"),
        valid.replace("(end 2 0)", "(end 2 0) (end 3 0)"),
        valid.replace("(uuid", &format!("(tstamp {uuid}) (uuid")),
    ] {
        let error = parse(
            Path::new("graphics.kicad_pcb"),
            &invalid,
            &BTreeMap::new(),
            &board(),
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("adoption refused before mutation"),
            "{error}"
        );
        assert!(error.contains("\"disposition\":\"refused\""), "{error}");
        assert!(error.contains("\"form\""), "{error}");
    }
    let duplicate = valid.replace(
        ")))",
        &format!(
            ")) (gr_line (start 0 0) (end 1 0) (width 0.1) (layer \"F.SilkS\") (uuid {uuid})))"
        ),
    );
    assert!(
        parse(
            Path::new("graphics.kicad_pcb"),
            &duplicate,
            &BTreeMap::new(),
            &board()
        )
        .is_err()
    );
}
