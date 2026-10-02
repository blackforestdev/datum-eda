//! Native authored-source current-fill nominal DRC proofs, not outline checks.
use super::*;
use crate::board::{Board, NetClass};
use crate::drc::{DrcReport, run_with_current_zone_fills_and_waivers, run_with_zone_fills};
use crate::rules::ast::RuleType;

fn native_board(model: &DesignModel) -> Board {
    let mut value = model
        .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    // Existing Board projection defaults do not supply nominal copper authority.
    // Stackup, Tracks, pads/vias, Zones, Nets and classes remain actual source.
    value["outline"] = serde_json::to_value(Polygon::new(vec![])).unwrap();
    for key in ["rules", "keepouts", "dimensions", "texts"] {
        value[key] = serde_json::json!([]);
    }
    serde_json::from_value(value).unwrap()
}
fn rect(x: i64, y: i64, width: i64, height: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, y),
        Point::new(x + width, y),
        Point::new(x + width, y + height),
        Point::new(x, y + height),
    ])
}
fn setup(name: &str) -> (PathBuf, DesignModel, Track, Zone, NetClass) {
    let (root, mut model, net) = fixture(name);
    let class = NetClass {
        uuid: Uuid::new_v4(),
        name: "proof".into(),
        clearance: 0,
        track_width: 1,
        via_drill: 1,
        via_diameter: 3,
        diffpair_width: 0,
        diffpair_gap: 0,
    };
    let foreign = Net::new(Uuid::new_v4(), "same name", class.uuid);
    let write = BatchComposer::compose(&model, provenance())
        .push_ops([
            Operation::CreateBoardNetClass {
                net_class_id: class.uuid,
                net_class: serde_json::to_value(&class).unwrap(),
            },
            Operation::SetBoardNet {
                net_id: net,
                net: serde_json::to_value(Net::new(net, "same name", class.uuid)).unwrap(),
            },
            Operation::CreateBoardNet {
                net_id: foreign.uuid,
                net: serde_json::to_value(&foreign).unwrap(),
            },
        ])
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut arc = line(net, (-10, 0), (10, 0));
    arc.midpoint = Some(Point::new(0, 10));
    tracks(&mut model, &root, std::slice::from_ref(&arc));
    let mut zone = zone(&mut model, &root, foreign.uuid);
    // All fixture fill cells lie within the actual authored Zone boundary.
    zone.polygon = rect(-40, -40, 80, 80);
    let write = build_set_board_zone(&model, provenance(), &zone).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    (root, model, arc, zone, class)
}
fn report(model: &DesignModel) -> DrcReport {
    run_with_current_zone_fills_and_waivers(
        &native_board(model),
        &[RuleType::ClearanceCopper],
        model,
        &[],
    )
}
fn source_pair(report: &DrcReport, arc: &Track, zone: &Zone, code: &str) {
    let mut expected = vec![arc.uuid, zone.uuid];
    expected.sort();
    assert!(!report.passed);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].code, code);
    assert_eq!(report.violations[0].objects, expected);
    assert!(report.violations[0].fingerprint.is_some());
}

#[test]
fn current_fill_drc_uses_actual_arc_locus_strict_equality_and_authored_zone_identity() {
    let (root, mut model, arc, zone, mut class) = setup("current_fill_drc_boundary");
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rect(-1, -1, 2, 2)],
    );
    assert!(
        report(&model).passed,
        "filled chord region does not touch upper arc"
    );
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rect(-2, 11, 4, 5)],
    );
    assert!(
        report(&model).passed,
        "exact copper tangency is not strict zero-clearance violation"
    );
    class.clearance = 1;
    let write = build_set_board_net_class(&model, provenance(), &class).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    source_pair(&report(&model), &arc, &zone, "nominal_geometry_unavailable");
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rect(-2, 11, 4, 5)],
    );
    let before = model.clone();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let result = report(&model);
    source_pair(&result, &arc, &zone, "clearance_copper");
    assert_eq!(model, before);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(report(&reopened), result);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn current_fill_drc_certifies_holes_and_empty_and_refuses_every_unverified_basis() {
    let (root, mut model, arc, zone, _) = setup("current_fill_drc_basis");
    source_pair(&report(&model), &arc, &zone, "nominal_geometry_unavailable");
    // Copper outside the entire arc has a real hole containing it.
    let frame = vec![
        rect(-30, -30, 60, 18),
        rect(-30, 12, 60, 18),
        rect(-30, -12, 18, 24),
        rect(12, -12, 18, 24),
    ];
    fill(&mut model, &root, &zone, ZoneFillState::Filled, frame);
    assert!(report(&model).passed);
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rect(-2, 9, 4, 4), rect(-15, -5, 2, 2)],
    );
    source_pair(&report(&model), &arc, &zone, "clearance_copper");
    for state in [
        ZoneFillState::Unsupported,
        ZoneFillState::Unfilled,
        ZoneFillState::Stale,
    ] {
        fill(&mut model, &root, &zone, state, vec![]);
        source_pair(&report(&model), &arc, &zone, "nominal_geometry_unavailable");
    }
    fill(&mut model, &root, &zone, ZoneFillState::Filled, vec![]);
    assert!(
        report(&model).passed,
        "current successful empty proves no Zone copper"
    );
    // The legacy raw-map API cannot certify emptiness/current revision.
    source_pair(
        &run_with_zone_fills(
            &native_board(&model),
            &[RuleType::ClearanceCopper],
            &model.zone_fills,
        ),
        &arc,
        &zone,
        "nominal_geometry_unavailable",
    );
    let mut mismatched = native_board(&model);
    mismatched.tracks.get_mut(&arc.uuid).unwrap().width += 1;
    source_pair(
        &run_with_current_zone_fills_and_waivers(
            &mismatched,
            &[RuleType::ClearanceCopper],
            &model,
            &[],
        ),
        &arc,
        &zone,
        "nominal_geometry_unavailable",
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn current_fill_drc_refuses_tampered_captured_source_and_keeps_existing_waiver_finalization() {
    let (root, mut model, arc, zone, _) = setup("current_fill_drc_capture");
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rect(-2, 9, 4, 4)],
    );
    let result = report(&model);
    source_pair(&result, &arc, &zone, "clearance_copper");
    let waiver: crate::schematic::CheckWaiver = serde_json::from_value(serde_json::json!({
        "uuid":Uuid::new_v4(),"domain":"DRC","target":{"Fingerprint":result.violations[0].fingerprint.clone().unwrap()},"rationale":"native proof waiver"})).unwrap();
    let waived = run_with_current_zone_fills_and_waivers(
        &native_board(&model),
        &[RuleType::ClearanceCopper],
        &model,
        &[waiver],
    );
    assert!(waived.passed);
    assert_eq!(waived.summary.waived, 1);
    assert!(waived.violations[0].waived);
    let path = root.join("board/board.json");
    let bytes = std::fs::read(&path).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["name"] = serde_json::json!("external change");
    std::fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    source_pair(&report(&model), &arc, &zone, "nominal_geometry_unavailable");
    std::fs::write(path, bytes).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}
