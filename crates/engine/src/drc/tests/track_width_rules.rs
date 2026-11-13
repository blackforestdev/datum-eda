//! Native persisted width policy and routing must agree on the same source.
use super::empty_board;
use crate::api::native_write::{
    BatchComposer,
    board_routing::route::{
        capture_route_basis, prepare_route,
        tests::{Fixture, fixture, provenance},
    },
    commit_prepared,
};
use crate::board::{Board, Net, NetClass};
use crate::drc::{DrcReport, run_with_current_zone_fills_and_waivers};
use crate::rules::ast::RuleType;
use crate::substrate::{Operation, ProjectResolver};
use serde_json::{Value, json};
use uuid::Uuid;

fn setup() -> Fixture {
    let mut f = fixture();
    // One saved 200000nm track is the frozen counterexample; the reusable
    // routing fixture otherwise authors a two-segment chain.
    f.request.points.remove(1);
    let prepared = prepare_route(&f.model, provenance(), &f.request).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let class = NetClass {
        uuid: Uuid::new_v4(),
        name: "100000nm fallback".into(),
        clearance: 100000,
        track_width: 100000,
        via_drill: 300000,
        via_diameter: 600000,
        diffpair_width: 0,
        diffpair_gap: 0,
    };
    let mut net: Net =
        serde_json::from_value(f.board()["nets"][f.request.basis.net.to_string()].clone()).unwrap();
    net.class = class.uuid;
    let prepared = BatchComposer::compose(&f.model, provenance())
        .push_ops([
            Operation::CreateBoardNetClass {
                net_class_id: class.uuid,
                net_class: serde_json::to_value(&class).unwrap(),
            },
            Operation::SetBoardNet {
                net_id: net.uuid,
                net: serde_json::to_value(net).unwrap(),
            },
        ])
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    f
}
fn rule(scope: Value, priority: u64, enabled: bool, min: i64, max: i64) -> Value {
    json!({"uuid":Uuid::new_v4(),"name":"Authored width","scope":scope,"priority":priority,
        "enabled":enabled,"rule_type":"track_width","params":{"min_nm":min,"preferred_nm":min,"max_nm":max}})
}
fn add(f: &mut Fixture, rule: Value) {
    f.write("datum.project.create_rule", json!({"rule":rule}));
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
}
fn board(f: &Fixture) -> Board {
    let source = f.board();
    let mut board = empty_board();
    board.uuid = serde_json::from_value(source["uuid"].clone()).unwrap();
    board.tracks = serde_json::from_value(source["tracks"].clone()).unwrap();
    board.nets = serde_json::from_value(source["nets"].clone()).unwrap();
    board.net_classes = serde_json::from_value(source["net_classes"].clone()).unwrap();
    board
}
fn report(f: &Fixture) -> DrcReport {
    run_with_current_zone_fills_and_waivers(&board(f), &[RuleType::TrackWidth], &f.model, &[])
}
fn basis(
    f: &Fixture,
) -> Result<crate::api::native_write::board_routing::route::RouteBasis, crate::error::EngineError> {
    capture_route_basis(&f.model, f.request.basis.start_pad, 1)
}

#[test]
fn saved_authored_width_counterexample_reports_governing_minimum() {
    let mut f = setup();
    assert!(report(&f).passed);
    add(&mut f, rule(json!("All"), 1, true, 500000, 800000));
    assert_eq!(basis(&f).unwrap().minimum_width_nm, Some(500000));
    let before = f.bytes();
    let report = report(&f);
    assert_eq!(report.summary.errors, 1);
    assert_eq!(report.violations[0].code, "track_width_below_min");
    assert!(report.violations[0].message.contains("required 500000nm"));
    assert_eq!(f.bytes(), before, "checking never writes");
}

#[test]
fn all_net_class_and_layer_scopes_match_and_nonmatches_keep_class_fallback() {
    for scope_kind in ["All", "Net", "NetClass", "Layer"] {
        let mut f = setup();
        let net = f.request.basis.net;
        let class = f.board()["nets"][net.to_string()]["class"].clone();
        let matching = match scope_kind {
            "All" => json!("All"),
            "Net" => json!({"Net":net}),
            "NetClass" => json!({"NetClass":class}),
            _ => json!({"Layer":1}),
        };
        if scope_kind != "All" {
            let other = if scope_kind == "Layer" {
                json!({"Layer":2})
            } else {
                json!({scope_kind:Uuid::new_v4()})
            };
            add(&mut f, rule(other, u64::MAX, true, 600000, 800000));
            assert!(report(&f).passed);
            assert_eq!(basis(&f).unwrap().minimum_width_nm, Some(100000));
        }
        add(&mut f, rule(matching, 1, true, 500000, 800000));
        assert_eq!(basis(&f).unwrap().minimum_width_nm, Some(500000));
        assert_eq!(report(&f).violations[0].code, "track_width_below_min");
    }
}

#[test]
fn disabled_rules_and_weaker_highest_unsigned_priority_are_consistent() {
    let mut f = setup();
    add(&mut f, rule(json!("All"), u64::MAX, false, 900000, 1000000));
    assert!(report(&f).passed);
    add(&mut f, rule(json!("All"), 1, true, 500000, 800000));
    assert!(!report(&f).passed);
    add(
        &mut f,
        rule(json!("All"), u64::MAX - 1, true, 50000, 250000),
    );
    assert_eq!(basis(&f).unwrap().minimum_width_nm, Some(50000));
    assert!(
        report(&f).passed,
        "explicit weaker rule overrides the class"
    );
}

#[test]
fn native_width_maximum_and_inclusive_boundaries_agree_with_routing() {
    for (min, max, expected) in [
        (100000, 199999, Some("track_width_above_max")),
        (200000, 200000, None),
        (200001, 300000, Some("track_width_below_min")),
    ] {
        let mut f = setup();
        add(&mut f, rule(json!("All"), 1, true, min, max));
        f.request.basis = basis(&f).unwrap();
        // Fresh edit identity avoids collisions with the already saved track.
        f.request.edit_seed = Uuid::new_v4();
        assert_eq!(
            prepare_route(&f.model, provenance(), &f.request).is_ok(),
            expected.is_none()
        );
        let report = report(&f);
        assert_eq!(report.violations.first().map(|v| v.code.as_str()), expected);
    }
}

#[test]
fn ambiguous_and_unsupported_policy_refuses_both_consumers() {
    for unsupported in [false, true] {
        let mut f = setup();
        if unsupported {
            add(
                &mut f,
                rule(json!({"And":["All",{"Layer":1}]}), 1, true, 100000, 300000),
            );
        } else {
            add(&mut f, rule(json!("All"), 1, true, 100000, 300000));
            add(&mut f, rule(json!("All"), 1, true, 100000, 300000));
        }
        assert!(basis(&f).is_err());
        let report = report(&f);
        assert!(!report.passed);
        assert_eq!(report.violations[0].code, "track_width_policy_unresolved");
    }
}

#[test]
fn native_width_refuses_mismatched_board_source() {
    let f = setup();
    let mut board = board(&f);
    board.tracks.values_mut().next().unwrap().width = 900000;
    let report =
        run_with_current_zone_fills_and_waivers(&board, &[RuleType::TrackWidth], &f.model, &[]);
    assert_eq!(report.violations[0].code, "track_width_policy_unresolved");
}

#[test]
fn invalid_native_width_ranges_are_refused_without_writes() {
    use crate::api::native_write::registry::{NativeWriteContext, find_native_write_verb};
    let mut f = setup();
    let before = f.bytes();
    for params in [
        json!({"min_nm":0,"preferred_nm":100000,"max_nm":300000}),
        json!({"min_nm":300000,"preferred_nm":200000,"max_nm":400000}),
        json!({"min_nm":100000,"preferred_nm":400000,"max_nm":300000}),
        json!({"min_nm":100000,"preferred_nm":"200000","max_nm":300000}),
    ] {
        let mut invalid = rule(json!("All"), 1, true, 100000, 300000);
        invalid["params"] = params;
        let prepared = (find_native_write_verb("datum.project.create_rule")
            .unwrap()
            .build)(
            &NativeWriteContext {
                model: &f.model,
                project_root: &f.root,
            },
            provenance(),
            json!({"rule":invalid}),
        )
        .unwrap();
        assert!(commit_prepared(&mut f.model, &f.root, prepared).is_err());
        assert_eq!(f.bytes(), before);
    }
    assert!(report(&f).passed);
}

#[test]
fn lower_priority_ties_and_disabled_unsupported_scopes_do_not_hide_winner() {
    let mut f = setup();
    add(
        &mut f,
        rule(
            json!({"And":["All",{"Layer":1}]}),
            u64::MAX,
            false,
            900000,
            1000000,
        ),
    );
    for _ in 0..2 {
        add(&mut f, rule(json!("All"), 1, true, 500000, 800000));
    }
    add(&mut f, rule(json!("All"), 2, true, 100000, 300000));
    assert_eq!(basis(&f).unwrap().minimum_width_nm, Some(100000));
    assert!(report(&f).passed);
}

#[test]
fn native_width_qualification_accepts_explicit_null_optional_fields() {
    let mut f = setup();
    let source = f.board();
    let (id, mut track) = source["tracks"]
        .as_object()
        .unwrap()
        .iter()
        .next()
        .map(|(id, value)| (id.parse().unwrap(), value.clone()))
        .unwrap();
    track["midpoint"] = Value::Null;
    let mut net = source["nets"][f.request.basis.net.to_string()].clone();
    net["controlled_impedance"] = Value::Null;
    let prepared = BatchComposer::compose(&f.model, provenance())
        .push_ops([
            Operation::SetBoardTrack {
                track_id: id,
                track,
            },
            Operation::SetBoardNet {
                net_id: f.request.basis.net,
                net,
            },
        ])
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    add(&mut f, rule(json!("All"), 1, true, 500000, 800000));
    assert_eq!(basis(&f).unwrap().minimum_width_nm, Some(500000));
    let report = report(&f);
    assert_eq!(report.summary.errors, 1);
    assert_eq!(report.violations[0].code, "track_width_below_min");
}
