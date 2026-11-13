use super::super::route::tests::{Fixture, fixture, provenance};
use super::*;
use crate::api::native_write::{self as native, BatchComposer, commit_prepared};
use crate::ir::geometry::Point;
use crate::substrate::{Operation, ProjectResolver};
use serde_json::json;

fn rectangle(x: i64, y: i64, w: i64, h: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, y),
        Point::new(x + w, y),
        Point::new(x + w, y + h),
        Point::new(x, y + h),
    ])
}
fn commit(f: &mut Fixture, ops: Vec<Operation>) {
    let write = BatchComposer::compose(&f.model, provenance())
        .push_ops(ops)
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
}
fn setup() -> (Fixture, ZoneRequest) {
    // The existing native fixture creates library, two placed/wired symbols,
    // qualified electrical assignments, footprint transfer and explicit Rule.
    let mut f = fixture();
    let outline = rectangle(-10_000_000, -10_000_000, 30_000_000, 30_000_000);
    let write = native::board_layout::build_set_board_outline(
        &f.model,
        provenance(),
        f.request.basis.board,
        &outline,
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let request = ZoneRequest {
        basis: capture_zone_basis(&f.model, f.request.basis.net, 1).unwrap(),
        polygon: rectangle(-1_000_000, -1_000_000, 10_000_000, 5_000_000),
        priority: 2,
        thermal_relief: false,
        thermal_gap_nm: 0,
        thermal_spoke_width_nm: 0,
        edit_seed: Uuid::new_v4(),
    };
    (f, request)
}
fn add(f: &mut Fixture, request: &ZoneRequest) -> Uuid {
    let write = prepare_zone(&f.model, provenance(), request).unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    id
}
fn fill_request(f: &Fixture, zone: Uuid) -> FillRequest {
    FillRequest {
        basis: capture_board_basis(&f.model).unwrap(),
        zone: Some(zone),
        net: None,
    }
}
fn unsupported(f: &Fixture, id: Uuid, reason: &str) {
    let before = f.bytes();
    let fills = preview_fills(&f.model, &fill_request(f, id)).unwrap();
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].state, ZoneFillState::Unsupported);
    assert!(fills[0].islands.is_empty());
    assert!(
        fills[0].provenance.as_ref().unwrap().contains(reason),
        "{:?}",
        fills[0]
    );
    assert_eq!(before, f.bytes(), "Unsupported preview must not write");
}

#[test]
fn boundary_cancel_atomic_acceptance_history_and_cold_are_separate_from_fill() {
    let (mut f, request) = setup();
    let before = f.bytes();
    let history = f.model.journal.len();
    let a = prepare_zone(&f.model, provenance(), &request).unwrap();
    let b = prepare_zone(&f.model, provenance(), &request).unwrap();
    assert_eq!(a.primary_object_id, b.primary_object_id);
    assert_eq!(
        before,
        f.bytes(),
        "discarding either preview is no-write cancellation"
    );
    let id = a.primary_object_id.unwrap();
    commit_prepared(&mut f.model, &f.root, a).unwrap();
    assert_eq!(f.model.journal.len(), history + 1);
    assert!(prepare_zone(&f.model, provenance(), &request).is_err());
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.model.zone_fills[&id].state, ZoneFillState::Unfilled);
    let after = f.board();
    for key in [
        "nets", "pads", "packages", "tracks", "vias", "stackup", "outline",
    ] {
        let prior: serde_json::Value = serde_json::from_slice(&before["board/board.json"]).unwrap();
        assert_eq!(prior[key], after[key], "unrelated {key}");
    }
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert!(f.board()["zones"].get(id.to_string()).is_none());
    let mut reused = request.clone();
    reused.basis = capture_zone_basis(&f.model, request.basis.net, request.basis.layer).unwrap();
    let before_reuse = f.bytes();
    assert!(
        prepare_zone(&f.model, provenance(), &reused)
            .unwrap_err()
            .to_string()
            .contains("already used")
    );
    assert_eq!(f.bytes(), before_reuse);
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.board(), after);
    assert_eq!(f.model.zone_fills[&id].state, ZoneFillState::Unfilled);
}

#[test]
fn explicit_fill_uses_current_geometry_preserves_authored_source_and_stales_after_edit() {
    let (mut f, request) = setup();
    let id = add(&mut f, &request);
    let authored = f.board();
    let request = fill_request(&f, id);
    let before = f.bytes();
    let preview = preview_fills(&f.model, &request).unwrap();
    assert_eq!(preview[0].state, ZoneFillState::Filled);
    let write = prepare_fill(&f.model, provenance(), &request).unwrap();
    assert_eq!(before, f.bytes());
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.board(), authored);
    assert_eq!(f.model.current_zone_fill(id), Some(&preview[0]));
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.model.zone_fills[&id].state, ZoneFillState::Unfilled);
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.model.current_zone_fill(id), Some(&preview[0]));
    let mut zone: Zone =
        serde_json::from_value(f.board()["zones"][id.to_string()].clone()).unwrap();
    zone.priority += 1;
    let write = super::super::build_set_board_zone(&f.model, provenance(), &zone).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.model.zone_fills[&id].state, ZoneFillState::Stale);
    assert!(f.model.current_zone_fill(id).is_none());
    let before = f.bytes();
    assert!(prepare_fill(&f.model, provenance(), &request).is_err());
    assert_eq!(f.bytes(), before);
}

#[test]
fn whole_refusal_covers_foreign_basis_scope_layer_and_malformed_boundary() {
    let (mut f, request) = setup();
    let before = f.bytes();
    for mutate in [0, 1, 2, 3, 4] {
        let mut bad = request.clone();
        match mutate {
            0 => bad.basis.source.project = Uuid::new_v4(),
            1 => bad.basis.source.board = Uuid::new_v4(),
            2 => bad.basis.layer = 999,
            3 => bad.edit_seed = Uuid::nil(),
            _ => bad.polygon.closed = false,
        }
        assert!(prepare_zone(&f.model, provenance(), &bad).is_err());
        assert_eq!(f.bytes(), before);
    }
    let id = add(&mut f, &request);
    for (zone, net) in [(Some(Uuid::new_v4()), None), (None, Some(Uuid::new_v4()))] {
        let bad = FillRequest {
            basis: capture_board_basis(&f.model).unwrap(),
            zone,
            net,
        };
        let before = f.bytes();
        assert!(prepare_fill(&f.model, provenance(), &bad).is_err());
        assert_eq!(before, f.bytes());
    }
    let before = f.bytes();
    let mut bad = fill_request(&f, id);
    bad.basis.tip = Some(Uuid::new_v4());
    assert!(prepare_fill(&f.model, provenance(), &bad).is_err());
    assert_eq!(before, f.bytes());
}

#[test]
fn thermal_outline_and_interacting_zone_requests_are_honest_unsupported() {
    let (mut f, mut request) = setup();
    request.thermal_relief = true;
    let id = add(&mut f, &request);
    unsupported(&f, id, "thermal relief");
    let write = prepare_fill(&f.model, provenance(), &fill_request(&f, id)).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.model.zone_fills[&id].state, ZoneFillState::Unsupported);
    assert!(f.model.current_zone_fill(id).is_none());
    let (mut f, mut request) = setup();
    request.polygon = rectangle(-11_000_000, -1_000_000, 20_000_000, 4_000_000);
    let id = add(&mut f, &request);
    unsupported(&f, id, "clipping");
    let (mut f, request) = setup();
    let id = add(&mut f, &request);
    let mut other = request.clone();
    other.basis = capture_zone_basis(&f.model, request.basis.net, 1).unwrap();
    other.edit_seed = Uuid::new_v4();
    other.priority = 9;
    add(&mut f, &other);
    unsupported(&f, id, "Interacting Zone");
}

#[test]
fn explicit_empty_fill_remains_authenticated_zero_copper_after_cold_history() {
    let (mut f, request) = setup();
    let id = add(&mut f, &request);
    let keepout = Keepout {
        uuid: Uuid::new_v4(),
        polygon: request.polygon.clone(),
        layers: vec![1],
        kind: "copper".into(),
    };
    commit(
        &mut f,
        vec![Operation::CreateBoardKeepout {
            keepout_id: keepout.uuid,
            keepout: serde_json::to_value(&keepout).unwrap(),
        }],
    );
    let fills = preview_fills(&f.model, &fill_request(&f, id)).unwrap();
    assert_eq!(fills[0].state, ZoneFillState::Filled);
    assert!(fills[0].islands.is_empty());
    let write = prepare_fill(&f.model, provenance(), &fill_request(&f, id)).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.model.current_zone_fill(id), Some(&fills[0]));
}

#[test]
fn qualified_foreign_pad_cutout_respects_rotation_and_actual_clearance_without_defaults() {
    let (mut f, request) = setup();
    let id = add(&mut f, &request);
    let foreign = Net::new(Uuid::new_v4(), "Foreign", Uuid::nil());
    let mut pad = ElectricalSelectionSnapshot::capture(&f.model)
        .unwrap()
        .board_source()
        .unwrap()
        .pads
        .values()
        .next()
        .unwrap()
        .clone();
    pad.uuid = Uuid::new_v4();
    pad.net = Some(foreign.uuid);
    pad.package = Uuid::nil();
    pad.position = Point::new(2_000_000, 2_000_000);
    pad.shape = PadShape::Rect;
    pad.width = 2_000_000;
    pad.height = 1_000_000;
    pad.rotation = 90;
    commit(
        &mut f,
        vec![
            Operation::CreateBoardNet {
                net_id: foreign.uuid,
                net: serde_json::to_value(&foreign).unwrap(),
            },
            Operation::CreateBoardPad {
                pad_id: pad.uuid,
                pad: serde_json::to_value(&pad).unwrap(),
            },
        ],
    );
    let fills = preview_fills(&f.model, &fill_request(&f, id)).unwrap();
    assert_eq!(fills[0].state, ZoneFillState::Filled);
    assert!(fills[0].islands.len() > 1);
    assert!(
        fills[0]
            .islands
            .iter()
            .all(|p| !occupied_copper::polygon_contains_point(p, pad.position).unwrap())
    );
    let rule = f.request.basis.clearance.as_ref().unwrap().source;
    f.write("datum.project.delete_rule", json!({"rule_id":rule}));
    unsupported(&f, id, "positive persisted clearance");
}

#[test]
fn adjacent_zone_respects_the_other_nets_larger_persisted_clearance() {
    let (mut f, request) = setup();
    let id = add(&mut f, &request);
    let net = Net::new(Uuid::new_v4(), "Larger clearance", Uuid::nil());
    commit(
        &mut f,
        vec![Operation::CreateBoardNet {
            net_id: net.uuid,
            net: serde_json::to_value(&net).unwrap(),
        }],
    );
    f.write("datum.project.create_rule", json!({"rule":{"uuid":Uuid::new_v4(),"name":"Foreign minimum","scope":{"Net":net.uuid},"priority":0,"enabled":true,"rule_type":"clearance_copper","params":{"min_nm":500000}}}));
    let other = ZoneRequest {
        basis: capture_zone_basis(&f.model, net.uuid, 1).unwrap(),
        polygon: rectangle(9_300_000, 0, 1_000_000, 1_000_000),
        edit_seed: Uuid::new_v4(),
        ..request
    };
    add(&mut f, &other);
    // A 300um gap satisfies the first Net's 100um rule but violates the
    // adjacent Net's real 500um minimum. No false independent-zone fill.
    unsupported(&f, id, "Interacting Zone");
}

#[test]
fn via_span_uses_physical_stack_order_and_unknown_endpoints_are_not_invisible() {
    use crate::board::{StackupLayer, Via};
    let (mut f, request) = setup();
    let layers: Vec<_> = [1, 10, 2]
        .into_iter()
        .map(|id| StackupLayer::new(id, format!("Copper {id}"), StackupLayerType::Copper, 35000))
        .collect();
    let write = native::board_layout::build_set_board_stackup(
        &f.model,
        provenance(),
        request.basis.source.board,
        &layers,
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let mut request = request;
    request.basis = capture_zone_basis(&f.model, request.basis.net, 10).unwrap();
    let id = add(&mut f, &request);
    let net = Net::new(Uuid::new_v4(), "Via Net", Uuid::nil());
    let mut via = Via {
        uuid: Uuid::new_v4(),
        net: net.uuid,
        position: Point::new(2_000_000, 2_000_000),
        diameter: 1_000_000,
        drill: 400000,
        from_layer: 1,
        to_layer: 2,
    };
    commit(
        &mut f,
        vec![
            Operation::CreateBoardNet {
                net_id: net.uuid,
                net: serde_json::to_value(&net).unwrap(),
            },
            Operation::CreateBoardVia {
                via_id: via.uuid,
                via: serde_json::to_value(&via).unwrap(),
            },
        ],
    );
    let source = f.board();
    let fills = preview_fills(&f.model, &fill_request(&f, id)).unwrap();
    assert_eq!(fills[0].state, ZoneFillState::Filled, "{:?}", fills[0]);
    assert!(fills[0].islands.len() > 1);
    assert!(
        fills[0]
            .islands
            .iter()
            .all(|p| !occupied_copper::polygon_contains_point(p, via.position).unwrap())
    );
    assert!(
        fills[0]
            .provenance
            .as_ref()
            .unwrap()
            .contains("not a minimal per-pair cut")
    );
    assert_eq!(
        f.board(),
        source,
        "physical source Via span must remain 1..2"
    );
    via.to_layer = 999;
    commit(
        &mut f,
        vec![Operation::SetBoardVia {
            via_id: via.uuid,
            via: serde_json::to_value(&via).unwrap(),
        }],
    );
    unsupported(&f, id, "Via copper layers unavailable");
}

#[test]
fn zone_clearance_readback_is_shared_with_fill_and_preserves_equal_largest_sources() {
    let (mut f, mut request) = setup();
    let own = request.basis.net;
    let initial = capture_zone_clearance_readback(&f.model, own, 1).unwrap();
    let a = Net::new(Uuid::new_v4(), "Larger A", Uuid::nil());
    commit(
        &mut f,
        vec![Operation::CreateBoardNet {
            net_id: a.uuid,
            net: serde_json::to_value(&a).unwrap(),
        }],
    );
    assert!(
        capture_zone_basis(&f.model, a.uuid, 1)
            .unwrap()
            .clearance
            .is_none()
    );
    assert_eq!(
        capture_zone_clearance_readback(&f.model, own, 1).unwrap(),
        initial,
        "a foreign Net without policy remains ignored, never a new blanket refusal"
    );
    let rule_a = Uuid::new_v4();
    f.write("datum.project.create_rule",json!({"rule":{"uuid":rule_a,"name":"Larger A minimum",
        "scope":{"Net":a.uuid},"priority":0,"enabled":true,"rule_type":"clearance_copper","params":{"min_nm":500000}}}));
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let readback = capture_zone_clearance_readback(&f.model, own, 1).unwrap();
    assert_eq!(readback.minimum_nm, 500000);
    assert_eq!(readback.sources.len(), 1);
    assert_eq!(readback.sources[0].source, rule_a);
    assert_eq!(readback.sources[0].name, "Larger A minimum");
    assert_eq!(
        readback.sources[0].kind,
        super::super::stop::ClearanceSource::Rule
    );
    request.basis = capture_zone_basis(&f.model, own, 1).unwrap();
    let id = add(&mut f, &request);
    let fills = preview_fills(&f.model, &fill_request(&f, id)).unwrap();
    assert!(
        fills[0]
            .provenance
            .as_ref()
            .unwrap()
            .contains("conservative cutout clearance 500000 nm")
    );
    let b = Net::new(Uuid::new_v4(), "Larger B", Uuid::nil());
    commit(
        &mut f,
        vec![Operation::CreateBoardNet {
            net_id: b.uuid,
            net: serde_json::to_value(&b).unwrap(),
        }],
    );
    let rule_b = Uuid::new_v4();
    f.write("datum.project.create_rule",json!({"rule":{"uuid":rule_b,"name":"Larger B minimum",
        "scope":{"Net":b.uuid},"priority":0,"enabled":true,"rule_type":"clearance_copper","params":{"min_nm":500000}}}));
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let before = f.bytes();
    let readback = capture_zone_clearance_readback(&f.model, own, 1).unwrap();
    assert_eq!(readback.minimum_nm, 500000);
    assert_eq!(readback.sources.len(), 2);
    assert!(readback.sources.iter().any(|p| p.source == rule_a));
    assert!(readback.sources.iter().any(|p| p.source == rule_b));
    assert_eq!(f.bytes(), before, "readback never writes or fills");
}
#[test]
fn zone_clearance_readback_refuses_missing_own_nonpositive_and_ambiguous_policy() {
    let (mut f, request) = setup();
    let mut invalid = request.basis.clone();
    invalid.clearance.as_mut().unwrap().minimum_nm = 0;
    assert!(required_zone_clearance(&invalid).is_err());
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    f.write(
        "datum.project.delete_rule",
        json!({"rule_id":request.basis.clearance.as_ref().unwrap().source}),
    );
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let before = f.bytes();
    assert!(
        capture_zone_clearance_readback(&f.model, request.basis.net, 1)
            .unwrap_err()
            .to_string()
            .contains("positive persisted")
    );
    assert_eq!(f.bytes(), before);
    let (mut f, request) = setup();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    f.write("datum.project.create_rule",json!({"rule":{"uuid":Uuid::new_v4(),"name":"Ambiguous equal priority",
        "scope":{"Net":request.basis.net},"priority":0,"enabled":true,"rule_type":"clearance_copper","params":{"min_nm":200000}}}));
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let before = f.bytes();
    assert!(capture_zone_clearance_readback(&f.model, request.basis.net, 1).is_err());
    assert_eq!(f.bytes(), before);
}
#[test]
fn zone_clearance_readback_refuses_changed_captured_board_or_rules_sources() {
    let (f, request) = setup();
    for kind in [SourceShardKind::BoardRoot, SourceShardKind::RulesRoot] {
        let shard = f
            .model
            .source_shards
            .iter()
            .find(|s| s.kind == kind)
            .unwrap();
        let bytes = std::fs::read(&shard.path).unwrap();
        let mut changed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        changed["readback_tamper"] = json!("outside accepted journal");
        std::fs::write(&shard.path, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(capture_zone_clearance_readback(&f.model, request.basis.net, 1).is_err());
        std::fs::write(&shard.path, &bytes).unwrap();
        assert!(capture_zone_clearance_readback(&f.model, request.basis.net, 1).is_ok());
    }
}
