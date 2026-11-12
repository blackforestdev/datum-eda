//! Canonical mapped-Pad routing, read-only previews and whole-chain refusals.
use super::*;
use crate::api::native_write::registry::{NativeWriteContext, find_native_write_verb};
use crate::api::native_write::schematic_board_transfer::{TransferChoice, TransferRequest};
use crate::api::native_write::{self as native, commit_prepared};
use crate::substrate::{
    CommitSource, ElectricalOccurrence, ElectricalSelectionSnapshot, ProjectResolver,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(crate) fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "route-test",
        CommitSource::Manual,
        "Manual native copper route",
    )
}

pub(crate) struct Fixture {
    pub(crate) root: PathBuf,
    pub(crate) model: DesignModel,
    pub(crate) request: RouteRequest,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
impl Fixture {
    pub(crate) fn write(&mut self, verb: &str, params: Value) {
        let prepared = (find_native_write_verb(verb).unwrap().build)(
            &NativeWriteContext {
                model: &self.model,
                project_root: &self.root,
            },
            provenance(),
            params,
        )
        .unwrap();
        commit_prepared(&mut self.model, &self.root, prepared).unwrap();
    }
    pub(crate) fn recapture(&mut self) {
        self.model = ProjectResolver::new(&self.root).resolve().unwrap();
        self.request.basis =
            capture_route_basis(&self.model, self.request.basis.start_pad, 1).unwrap();
    }
    pub(crate) fn board(&self) -> Value {
        self.model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap()
    }
    pub(crate) fn bytes(&self) -> BTreeMap<String, Vec<u8>> {
        let mut result: BTreeMap<_, _> = self
            .model
            .source_shards
            .iter()
            .map(|s| {
                (
                    s.relative_path.clone(),
                    std::fs::read(self.root.join(&s.relative_path)).unwrap(),
                )
            })
            .collect();
        for path in [
            crate::substrate::transaction_journal_path(&self.root),
            self.root.join(".datum/journal/cursor.json"),
        ] {
            if path.exists() {
                result.insert(
                    path.strip_prefix(&self.root)
                        .unwrap()
                        .to_string_lossy()
                        .into(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
        result
    }
}

pub(crate) fn fixture() -> Fixture {
    let root = native::test_support::temp_project_root("native_manual_route");
    let genesis = native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Manual route".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, symbol, entity, gate, package, stack, footprint, pad, part, map, sheet) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let mut write = |verb: &str, params: Value| {
        let prepared = (find_native_write_verb(verb).unwrap().build)(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            provenance(),
            params,
        )
        .unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
    };
    for (kind, id, fields) in [
        (
            "units",
            unit,
            json!({"name":"Passive pin","pins":{pin.to_string():{"name":"Signal","electrical_type":"Passive"}}}),
        ),
        (
            "symbols",
            symbol,
            json!({"name":"Body","unit":unit,"drawings":[{"Rect":{"min":{"x":-500000,"y":-500000},"max":{"x":500000,"y":500000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}),
        ),
        ("packages", package, json!({"name":"Package"})),
        (
            "padstacks",
            stack,
            json!({"name":"Copper","aperture":{"circle":{"diameter_nm":1000000}}}),
        ),
        (
            "footprints",
            footprint,
            json!({"name":"One pad","package":package,"pads":{pad.to_string():{"uuid":pad,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":1}}}),
        ),
        (
            "entities",
            entity,
            json!({"name":"Entity","manufacturer":"Designer","prefix":"U","gates":{gate.to_string():{"uuid":gate,"name":"G","unit":unit,"symbol":symbol}}}),
        ),
        (
            "parts",
            part,
            json!({"manufacturer":"Designer","mpn":"ROUTE","description":"","datasheet":"","value":"Signal","entity":entity,"package":package,"default_footprint":footprint}),
        ),
        (
            "pin_pad_maps",
            map,
            json!({"part":part,"footprint":footprint,"mappings":{pad.to_string():{"gate":gate,"pin":pin}}}),
        ),
    ] {
        write(
            "datum.library.create",
            json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
        );
    }
    write(
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    for (reference, x) in [("U1", 0), ("U2", 10000000)] {
        // Capture against the updated model before each public placement.
        let selection = native::schematic_library::part_placement::PartGateSelection::capture(
            &model, part, gate, symbol,
        )
        .unwrap();
        let prepared = (find_native_write_verb("datum.schematic.place_symbol").unwrap().build)(
            &NativeWriteContext { model: &model, project_root: &root }, provenance(),
            json!({"sheet":sheet,"instance_path":[],"library_symbol":symbol,"explicit_part":selection,"reference":reference,"value":"Signal","x_nm":x,"y_nm":0}),
        ).unwrap();
        commit_prepared(&mut model, &root, prepared).unwrap();
    }
    let prepared = (find_native_write_verb("datum.schematic.draw_wire").unwrap().build)(
        &NativeWriteContext { model: &model, project_root: &root }, provenance(),
        json!({"sheet":sheet,"instance_path":[],"points":[{"x":-2000000,"y":0},{"x":8000000,"y":0}]}),
    ).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let prepared = native::board_layout::source_profile::build_adopt_board_source_profile(
        &model,
        provenance(),
        genesis.board_uuid,
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let transfer = TransferRequest {
        sheet,
        choices: model
            .component_instances
            .values()
            .map(|ci| TransferChoice {
                component_instance: ci.id,
                pin_pad_map: map,
            })
            .collect(),
        edit_seed: Uuid::new_v4(),
        origin: Point::zero(),
        pitch_nm: 5_000_000,
        layer: 1,
    };
    let prepared =
        native::schematic_board_transfer::prepare_transfer(&model, provenance(), &transfer)
            .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let projection = source::component_pad_projection(&model, &board).unwrap();
    let mut pads: Vec<_> = projection.pads.values().collect();
    pads.sort_by_key(|p| p.position.x);
    assert_eq!(pads.len(), 2);
    // An explicit test-authored Rule supplies intent. Production routing has
    // no manufactured clearance default when a new board has no rules/class.
    let rule = json!({
        "uuid":Uuid::new_v4(),"name":"Chosen routing clearance",
        "scope":{"Net":pads[0].net.unwrap()},"priority":0,"enabled":true,
        "rule_type":"clearance_copper","params":{"min_nm":100000},
    });
    let prepared = (find_native_write_verb("datum.project.create_rule")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        json!({"rule":rule}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let basis = capture_route_basis(&model, pads[0].uuid, 1).unwrap();
    let request = RouteRequest {
        basis,
        end_pad: Some(pads[1].uuid),
        points: vec![
            pads[0].position,
            Point::new(2_500_000, 2_000_000),
            pads[1].position,
        ],
        width_nm: 200_000,
        edit_seed: Uuid::new_v4(),
    };
    Fixture {
        root,
        model,
        request,
    }
}

fn run_members(f: &Fixture) -> std::collections::BTreeSet<ElectricalOccurrence> {
    ElectricalSelectionSnapshot::capture(&f.model)
        .unwrap()
        .board_run(
            &f.model.model_revision,
            &ElectricalOccurrence {
                class: "pads".into(),
                source_id: f.request.basis.start_pad,
                instance_path: vec![],
            },
            None,
        )
        .unwrap()
        .members
}

#[test]
fn mapped_route_is_one_canonical_chain_with_readonly_preview_and_cold_history() {
    let mut f = fixture();
    let before = f.bytes();
    let board_before = f.board();
    let journal_before = f.model.journal.len();
    assert_eq!(
        f.request.basis.preferred_width_nm, None,
        "no invented netclass width"
    );
    assert_eq!(
        run_members(&f).iter().filter(|o| o.class == "pads").count(),
        1
    );
    let prepared = prepare_route(&f.model, provenance(), &f.request).unwrap();
    let second = prepare_route(&f.model, provenance(), &f.request).unwrap();
    assert_eq!(prepared.batch.operations, second.batch.operations);
    assert_eq!(prepared.batch.operations.len(), 2);
    assert_eq!(
        f.bytes(),
        before,
        "building/cancelling previews must not write"
    );
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.model.journal.len(), journal_before + 1);
    let board_after = f.board();
    assert_eq!(board_after["tracks"].as_object().unwrap().len(), 2);
    for track in board_after["tracks"].as_object().unwrap().values() {
        assert_eq!(track["net"], json!(f.request.basis.net));
        assert_eq!(track["width"], json!(200000));
        assert_eq!(track["layer"], json!(1));
    }
    for key in [
        "packages",
        "component_pads",
        "nets",
        "net_classes",
        "stackup",
    ] {
        assert_eq!(
            board_before[key], board_after[key],
            "unrelated {key} source"
        );
    }
    let members = run_members(&f);
    assert_eq!(members.iter().filter(|o| o.class == "pads").count(), 2);
    assert_eq!(members.iter().filter(|o| o.class == "tracks").count(), 2);
    let stable = f.bytes();
    assert!(
        prepare_route(&f.model, provenance(), &f.request).is_err(),
        "old acceptance cannot replay"
    );
    assert_eq!(f.bytes(), stable);
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.board(), board_before);
    assert_eq!(
        run_members(&f).iter().filter(|o| o.class == "pads").count(),
        1
    );
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.board(), board_after);
    assert_eq!(run_members(&f), members);
    f.recapture();
    assert!(
        prepare_route(&f.model, provenance(), &f.request)
            .unwrap_err()
            .to_string()
            .contains("already been used")
    );
}

#[test]
fn invalid_route_basis_endpoints_width_layer_and_segments_wholly_refuse() {
    let mut f = fixture();
    let before = f.bytes();
    let mut cases = Vec::new();
    let mut bad = f.request.clone();
    bad.basis.net = Uuid::new_v4();
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.basis.project = Uuid::new_v4();
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.basis.revision = ModelRevision("stale".into());
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.basis.tip = None;
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.basis.start_pad = Uuid::new_v4();
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.basis.layer = 2;
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.width_nm = 0;
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.width_nm = -1;
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.points.clear();
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.points[0].x += 1;
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.points[1] = bad.points[0];
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.points[1].x = i64::MAX;
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.end_pad = Some(Uuid::new_v4());
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.points.last_mut().unwrap().x += 1;
    cases.push(bad);
    let mut bad = f.request.clone();
    bad.edit_seed = Uuid::nil();
    cases.push(bad);
    for (index, request) in cases.iter().enumerate() {
        assert!(
            prepare_route(&f.model, provenance(), request).is_err(),
            "case {index}"
        );
    }
    let mut params = serde_json::to_value(&f.request).unwrap();
    params["unreviewed_override"] = json!(true);
    assert!(
        (find_native_write_verb("datum.pcb.draw_track")
            .unwrap()
            .build)(
            &NativeWriteContext {
                model: &f.model,
                project_root: &f.root
            },
            provenance(),
            params,
        )
        .is_err()
    );
    assert_eq!(f.bytes(), before);
    let prepared = prepare_route(&f.model, provenance(), &f.request).unwrap();
    f.write(
        "datum.project.set_name",
        json!({"name":"Concurrent project change"}),
    );
    let current = f.bytes();
    assert!(
        commit_prepared(&mut f.model, &f.root, prepared).is_err(),
        "changed model rejects the entire prepared chain"
    );
    assert_eq!(f.bytes(), current);
    assert!(f.board()["tracks"].as_object().unwrap().is_empty());
}

#[test]
fn actual_native_width_rules_override_class_by_priority_and_enforce_range() {
    let mut f = fixture();
    let class = NetClass {
        uuid: Uuid::new_v4(),
        name: "Native width".into(),
        clearance: 100000,
        track_width: 250000,
        via_drill: 300000,
        via_diameter: 600000,
        diffpair_width: 250000,
        diffpair_gap: 200000,
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
    assert_eq!(f.request.basis.minimum_width_nm, Some(250000));
    assert!(prepare_route(&f.model, provenance(), &f.request).is_err());
    let rule = |priority, min, preferred, max| {
        json!({
            "uuid":Uuid::new_v4(),"name":"Routing width","scope":"All","priority":priority,"enabled":true,
            "rule_type":"track_width","params":{"min_nm":min,"preferred_nm":preferred,"max_nm":max},
        })
    };
    f.write(
        "datum.project.create_rule",
        json!({"rule":rule(1,300000,400000,600000)}),
    );
    f.recapture();
    assert_eq!(f.request.basis.preferred_width_nm, Some(400000));
    let before = f.bytes();
    assert!(prepare_route(&f.model, provenance(), &f.request).is_err());
    assert_eq!(f.bytes(), before);
    f.write(
        "datum.project.create_rule",
        json!({"rule":rule(2,100000,150000,200000)}),
    );
    f.recapture();
    assert_eq!(f.request.basis.preferred_width_nm, Some(150000));
    f.request.width_nm = 150000;
    assert!(prepare_route(&f.model, provenance(), &f.request).is_ok());
    f.request.width_nm = 200001;
    assert!(prepare_route(&f.model, provenance(), &f.request).is_err());
    f.write(
        "datum.project.create_rule",
        json!({"rule":rule(2,100000,150000,200000)}),
    );
    let before = f.bytes();
    assert!(
        capture_route_basis(&f.model, f.request.basis.start_pad, 1)
            .unwrap_err()
            .to_string()
            .contains("ambiguous")
    );
    assert_eq!(f.bytes(), before);
    // A matching width rule cannot hide a dangling, non-nil class source.
    let mut net: Net =
        serde_json::from_value(f.board()["nets"][f.request.basis.net.to_string()].clone()).unwrap();
    net.class = Uuid::new_v4();
    let prepared =
        native::board_routing::build_set_board_net(&f.model, provenance(), &net).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let before = f.bytes();
    assert!(
        capture_route_basis(&f.model, f.request.basis.start_pad, 1)
            .unwrap_err()
            .to_string()
            .contains("Net class source")
    );
    assert_eq!(f.bytes(), before);
}

#[test]
fn foreign_net_pad_and_unsupported_rule_scope_cannot_author_a_chain() {
    let mut f = fixture();
    let board = f.board();
    let projection = source::component_pad_projection(&f.model, &board).unwrap();
    let mut pad = projection.pads[&f.request.end_pad.unwrap()].clone();
    let net = Net::new(Uuid::new_v4(), "Foreign", Uuid::nil());
    pad.uuid = Uuid::new_v4();
    pad.net = Some(net.uuid);
    pad.position = Point::new(20000000, 0);
    let prepared = BatchComposer::compose(&f.model, provenance())
        .push_ops([
            Operation::CreateBoardNet {
                net_id: net.uuid,
                net: serde_json::to_value(net).unwrap(),
            },
            Operation::CreateBoardPad {
                pad_id: pad.uuid,
                pad: serde_json::to_value(&pad).unwrap(),
            },
        ])
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.recapture();
    f.request.end_pad = Some(pad.uuid);
    *f.request.points.last_mut().unwrap() = pad.position;
    let before = f.bytes();
    assert!(
        prepare_route(&f.model, provenance(), &f.request)
            .unwrap_err()
            .to_string()
            .contains("same Net")
    );
    assert_eq!(f.bytes(), before);
    f.write(
        "datum.project.create_rule",
        json!({"rule":{
            "uuid":Uuid::new_v4(),"name":"Unsupported routing scope","scope":{"And":["All","All"]},
            "priority":10,"enabled":true,"rule_type":"track_width",
            "params":{"min_nm":100000,"preferred_nm":200000,"max_nm":300000},
        }}),
    );
    let before = f.bytes();
    assert!(matches!(
        capture_route_basis(&f.model, f.request.basis.start_pad, 1),
        Err(EngineError::UnsupportedScope { .. })
    ));
    assert_eq!(f.bytes(), before);
}
