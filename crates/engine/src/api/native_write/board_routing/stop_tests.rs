use super::super::route::{
    prepare_route,
    tests::{Fixture, fixture, provenance},
};
use super::*;
use crate::api::native_write::registry::{NativeWriteContext, find_native_write_verb};
use crate::api::native_write::{self as native, BatchComposer, commit_prepared};
use crate::board::{StackupLayer, StackupLayerType, Via, Zone};
use crate::substrate::{
    Operation, ZONE_FILL_SCHEMA_VERSION, ZoneFill, ZoneFillCopperContext, ZoneFillState,
};
use serde_json::json;

fn commit(f: &mut Fixture, operations: Vec<Operation>) {
    let prepared = BatchComposer::compose(&f.model, provenance())
        .push_ops(operations)
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
}
fn foreign(f: &mut Fixture) -> Uuid {
    let net = Net::new(Uuid::new_v4(), "Foreign", Uuid::nil());
    commit(
        f,
        vec![Operation::CreateBoardNet {
            net_id: net.uuid,
            net: serde_json::to_value(&net).unwrap(),
        }],
    );
    f.request.points = vec![f.request.basis.start, *f.request.points.last().unwrap()];
    f.recapture();
    net.uuid
}
fn no_write(f: &Fixture, reason: &str) {
    let before = f.bytes();
    let error = prepare_route(&f.model, provenance(), &f.request)
        .unwrap_err()
        .to_string();
    assert!(error.contains(reason), "expected {reason}: {error}");
    assert_eq!(f.bytes(), before);
}
fn rect(x: i64, y: i64, width: i64, height: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, y),
        Point::new(x + width, y),
        Point::new(x + width, y + height),
        Point::new(x, y + height),
    ])
}
fn set_track(f: &mut Fixture, track: &Track) {
    let prepared =
        native::board_routing::build_set_board_track(&f.model, provenance(), track).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
}

#[test]
fn missing_clearance_requires_an_explicit_persisted_rule_with_cancellable_preview() {
    let mut f = fixture();
    let old_rule = f.request.basis.clearance.as_ref().unwrap().source;
    f.write("datum.project.delete_rule", json!({"rule_id":old_rule}));
    f.recapture();
    assert!(f.request.basis.clearance.is_none());
    no_write(&f, "Configure an explicit clearance rule");
    let before = f.bytes();
    let history = f.model.journal.len();
    let id = Uuid::new_v4();
    let rule = json!({"uuid":id,"name":"Chosen clearance","scope":{"Net":f.request.basis.net},
        "priority":0,"enabled":true,"rule_type":"clearance_copper","params":{"min_nm":175000}});
    let prepared = (find_native_write_verb("datum.project.create_rule")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        },
        provenance(),
        json!({"rule":rule}),
    )
    .unwrap();
    assert_eq!(
        f.bytes(),
        before,
        "discarding this prepared Rule is a zero-write cancellation"
    );
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    assert_eq!(f.model.journal.len(), history + 1);
    let policy = f.request.basis.clearance.as_ref().unwrap();
    assert_eq!(
        (policy.minimum_nm, policy.source, policy.kind),
        (175000, id, ClearanceSource::Rule)
    );
    let prepared = prepare_route(&f.model, provenance(), &f.request).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    assert_eq!(f.model.journal.len(), history + 2);
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.recapture();
    assert!(f.board()["tracks"].as_object().unwrap().is_empty());
    assert_eq!(f.request.basis.clearance.as_ref().unwrap().source, id);
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.recapture();
    assert!(f.request.basis.clearance.is_none());
}

#[test]
fn foreign_track_exact_clearance_same_net_join_and_other_layer_are_distinct() {
    let mut f = fixture();
    let net = foreign(&mut f);
    let mut track = Track::straight(
        Uuid::new_v4(),
        net,
        Point::new(1000000, 299999),
        Point::new(4000000, 299999),
        200000,
        1,
    );
    let prepared =
        native::board_routing::build_place_board_track(&f.model, provenance(), &track).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    no_write(&f, &track.uuid.to_string());
    let context = StopContext::capture(&f.model, &f.request.basis).unwrap();
    let mut wrong_start = f.request.points.clone();
    wrong_start[0].x += 1;
    assert!(
        context
            .preview_path(&wrong_start, f.request.width_nm)
            .unwrap_err()
            .to_string()
            .contains("Pad centre")
    );
    for y in [300000, 300001] {
        track.from.y = y;
        track.to.y = y;
        set_track(&mut f, &track);
        assert!(
            prepare_route(&f.model, provenance(), &f.request).is_ok(),
            "exact clearance is legal"
        );
    }
    let class = NetClass {
        uuid: Uuid::new_v4(),
        name: "Foreign minimum".into(),
        clearance: 300000,
        track_width: 200000,
        via_drill: 300000,
        via_diameter: 600000,
        diffpair_width: 200000,
        diffpair_gap: 200000,
    };
    let mut net_source: Net =
        serde_json::from_value(f.board()["nets"][net.to_string()].clone()).unwrap();
    net_source.class = class.uuid;
    commit(
        &mut f,
        vec![
            Operation::CreateBoardNetClass {
                net_class_id: class.uuid,
                net_class: serde_json::to_value(class).unwrap(),
            },
            Operation::SetBoardNet {
                net_id: net,
                net: serde_json::to_value(net_source).unwrap(),
            },
        ],
    );
    f.recapture();
    no_write(&f, "clearance");
    track.from.y = 500000;
    track.to.y = 500000;
    set_track(&mut f, &track);
    assert!(prepare_route(&f.model, provenance(), &f.request).is_ok());
    track.net = f.request.basis.net;
    track.from.y = 0;
    track.to.y = 0;
    set_track(&mut f, &track);
    assert!(
        prepare_route(&f.model, provenance(), &f.request).is_ok(),
        "same-net copper can join"
    );
    let mut layers: Vec<StackupLayer> =
        serde_json::from_value(f.board()["stackup"]["layers"].clone()).unwrap();
    layers.push(StackupLayer::new(
        16,
        "Bottom",
        StackupLayerType::Copper,
        35000,
    ));
    let prepared = native::board_layout::build_set_board_stackup(
        &f.model,
        provenance(),
        f.request.basis.board,
        &layers,
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    track.net = net;
    track.layer = 16;
    set_track(&mut f, &track);
    assert!(
        prepare_route(&f.model, provenance(), &f.request).is_ok(),
        "other copper layers do not collide"
    );
    track.layer = 1;
    track.width = 0;
    set_track(&mut f, &track);
    no_write(&f, "geometry unavailable");
}

#[test]
fn actual_pad_and_via_copper_reject_contact_and_preserve_exact_spacing_boundary() {
    let mut f = fixture();
    let net = foreign(&mut f);
    let projection =
        crate::substrate::component_pad_source::component_pad_projection(&f.model, &f.board())
            .unwrap();
    let mut pad = projection.pads[&f.request.end_pad.unwrap()].clone();
    pad.uuid = Uuid::new_v4();
    pad.net = Some(net);
    pad.position = Point::new(2500000, 699999);
    let prepared =
        native::board_routing::build_place_board_pad(&f.model, provenance(), &pad).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    no_write(&f, &pad.uuid.to_string());
    pad.position.y = 700000;
    let prepared =
        native::board_routing::build_set_board_pad(&f.model, provenance(), &pad).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    assert!(prepare_route(&f.model, provenance(), &f.request).is_ok());
    let mut via = Via {
        uuid: Uuid::new_v4(),
        net,
        position: Point::new(2500000, 499999),
        drill: 300000,
        diameter: 600000,
        from_layer: 1,
        to_layer: 1,
    };
    let prepared =
        native::board_routing::build_place_board_via(&f.model, provenance(), &via).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    no_write(&f, &via.uuid.to_string());
    via.position.y = 500000;
    let prepared =
        native::board_routing::build_set_board_via(&f.model, provenance(), &via).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    assert!(prepare_route(&f.model, provenance(), &f.request).is_ok());
}

#[test]
fn keepout_and_exact_rectangular_outline_stop_full_copper_not_only_centerline() {
    let mut f = fixture();
    foreign(&mut f);
    let mut keepout = Keepout {
        uuid: Uuid::new_v4(),
        polygon: rect(2000000, -50000, 1000000, 100000),
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
    f.recapture();
    no_write(&f, &keepout.uuid.to_string());
    for kind in ["component", "via", "route", "mixed"] {
        keepout.kind = kind.into();
        commit(
            &mut f,
            vec![Operation::SetBoardKeepout {
                keepout_id: keepout.uuid,
                keepout: serde_json::to_value(&keepout).unwrap(),
            }],
        );
        f.recapture();
        match kind {
            "component" | "via" => {
                assert!(prepare_route(&f.model, provenance(), &f.request).is_ok())
            }
            "route" => no_write(&f, &keepout.uuid.to_string()),
            _ => no_write(&f, "unsupported kind"),
        }
    }
    commit(
        &mut f,
        vec![Operation::DeleteBoardKeepout {
            keepout_id: keepout.uuid,
            keepout: serde_json::to_value(&keepout).unwrap(),
        }],
    );
    f.recapture();
    assert!(
        !StopContext::capture(&f.model, &f.request.basis)
            .unwrap()
            .outline_defined()
    );
    for (y, allowed) in [(-100000, true), (-99999, false)] {
        let outline = rect(-1000000, y, 7000000, 3000000);
        let prepared = native::board_layout::build_set_board_outline(
            &f.model,
            provenance(),
            f.request.basis.board,
            &outline,
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        f.recapture();
        assert!(
            StopContext::capture(&f.model, &f.request.basis)
                .unwrap()
                .outline_defined()
        );
        if allowed {
            assert!(prepare_route(&f.model, provenance(), &f.request).is_ok());
        } else {
            no_write(&f, "outside the board outline");
        }
    }
    let triangle = Polygon::new(vec![
        Point::new(-1000000, -1000000),
        Point::new(6000000, -1000000),
        Point::new(2500000, 5000000),
    ]);
    let prepared = native::board_layout::build_set_board_outline(
        &f.model,
        provenance(),
        f.request.basis.board,
        &triangle,
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    no_write(&f, "full-stroke containment");
    let bounds = Rect::new(Point::new(0, 0), Point::new(10, 10));
    let exact = Track::straight(
        Uuid::nil(),
        Uuid::nil(),
        Point::new(1, 1),
        Point::new(9, 9),
        2,
        1,
    );
    assert!(inside_rectangle(&exact, bounds));
    assert!(
        !inside_rectangle(&Track { width: 3, ..exact }, bounds),
        "odd-width half-nanometre extent is not rounded away"
    );
}

fn fill(f: &mut Fixture, zone: &Zone, keepouts: Vec<Keepout>) {
    let projection =
        crate::substrate::component_pad_source::component_pad_projection(&f.model, &f.board())
            .unwrap();
    let context = ZoneFillCopperContext {
        pads: projection.pads.into_values().collect(),
        keepouts,
        net_clearance_nm: BTreeMap::from([(f.request.basis.net, 100000), (zone.net, 100000)]),
        ..Default::default()
    };
    let (state, islands, reason) = crate::substrate::compute_bounded_zone_fill(zone, &context);
    assert_eq!(state, ZoneFillState::Filled, "{reason}");
    let value = ZoneFill {
        schema_version: ZONE_FILL_SCHEMA_VERSION,
        zone_id: zone.uuid,
        state,
        source_zone_revision: f.model.objects[&zone.uuid].object_revision,
        model_revision: f.model.model_revision.clone(),
        islands,
        provenance: Some(reason),
    };
    let prepared =
        native::board_routing::build_set_zone_fills(&f.model, provenance(), &[value]).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    assert!(f.model.current_zone_fill(zone.uuid).is_some());
}

#[test]
fn foreign_zone_uses_current_islands_and_authenticated_empty_not_authored_boundary() {
    let mut f = fixture();
    let net = foreign(&mut f);
    f.write("datum.project.create_rule",json!({"rule":{"uuid":Uuid::new_v4(),"name":"Foreign clearance",
        "scope":{"Net":net},"priority":0,"enabled":true,"rule_type":"clearance_copper","params":{"min_nm":100000}}}));
    let mut zone = Zone {
        uuid: Uuid::new_v4(),
        net,
        polygon: rect(1000000, 199999, 3000000, 1000000),
        layer: 1,
        priority: 0,
        thermal_relief: false,
        thermal_gap: 0,
        thermal_spoke_width: 0,
    };
    let prepared =
        native::board_routing::build_place_board_zone(&f.model, provenance(), &zone).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    no_write(&f, "fill");
    fill(&mut f, &zone, vec![]);
    no_write(&f, &zone.uuid.to_string());
    zone.polygon = rect(1000000, 200000, 3000000, 1000000);
    let prepared =
        native::board_routing::build_set_board_zone(&f.model, provenance(), &zone).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    no_write(&f, "fill");
    fill(&mut f, &zone, vec![]);
    assert!(
        prepare_route(&f.model, provenance(), &f.request).is_ok(),
        "exact clearance to actual island boundary"
    );
    zone.polygon = rect(1000000, 199999, 3000000, 1000000);
    let prepared =
        native::board_routing::build_set_board_zone(&f.model, provenance(), &zone).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let keepout = Keepout {
        uuid: Uuid::new_v4(),
        polygon: zone.polygon.clone(),
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
    fill(&mut f, &zone, vec![keepout]);
    assert!(
        f.model
            .current_zone_fill(zone.uuid)
            .unwrap()
            .islands
            .is_empty()
    );
    assert!(
        prepare_route(&f.model, provenance(), &f.request).is_ok(),
        "empty authenticated fill is not authored-zone copper"
    );
    f.write(
        "datum.project.set_name",
        json!({"name":"Changed source basis"}),
    );
    f.recapture();
    no_write(&f, "fill");
}
