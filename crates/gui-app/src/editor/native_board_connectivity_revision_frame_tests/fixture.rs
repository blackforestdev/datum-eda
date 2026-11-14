//! Representative native circuit prepared only through public engine writers.
use super::*;
use eda_engine::{
    api::native_write::{
        BatchComposer, PreparedWrite,
        registry::{NativeWriteContext, find_native_write_verb},
        schematic_board_transfer::{TransferRequest, prepare_transfer},
    },
    ir::geometry::Point,
    substrate::{ElectricalIdentity, ElectricalSelectionSnapshot},
};
use std::{collections::BTreeMap, path::Path};
pub(super) fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "connectivity-test",
        CommitSource::Manual,
        "Edit native circuit",
    )
}
pub(super) fn build(model: &DesignModel, root: &Path, verb: &str, params: Value) -> PreparedWrite {
    (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap()
}
pub(super) fn write(model: &mut DesignModel, root: &Path, verb: &str, params: Value) {
    let prepared = build(model, root, verb, params);
    commit_prepared(model, root, prepared).unwrap();
}
pub(super) struct Circuit {
    pub(super) root: PathBuf,
    pub(super) model: DesignModel,
    pub(super) sheet: Uuid,
}
pub(super) fn setup(root: &Path) -> Circuit {
    assert!(!root.exists(), "Proof fixture must use a fresh owned path");
    let root = root.to_path_buf();
    let genesis = native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Three resistor circuit".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let [
        unit,
        pin_a,
        pin_b,
        symbol,
        entity,
        gate,
        package,
        stack,
        footprint,
        pad_a,
        pad_b,
        part,
        map,
        sheet,
    ] = std::array::from_fn(|_| Uuid::new_v4());
    for (kind, id, fields) in [
        (
            "units",
            unit,
            json!({"name":"Resistor","pins":{
            pin_a.to_string():{"name":"1","electrical_type":"Passive"},
            pin_b.to_string():{"name":"2","electrical_type":"Passive"}}}),
        ),
        (
            "symbols",
            symbol,
            json!({"name":"Resistor","unit":unit,"drawings":[{"Rect":{"min":{"x":-500000,"y":-500000},"max":{"x":500000,"y":500000},"width":100000}}],"pin_anchors":[
            {"pin":pin_a,"position":{"x":0,"y":2000000},"style":{"orientation":"Down","length_nm":1000000,"decoration":"none"}},
            {"pin":pin_b,"position":{"x":0,"y":-2000000},"style":{"orientation":"Up","length_nm":1000000,"decoration":"none"}}]}),
        ),
        ("packages", package, json!({"name":"0603"})),
        (
            "padstacks",
            stack,
            json!({"name":"Copper","aperture":{"circle":{"diameter_nm":1000000}}}),
        ),
        (
            "footprints",
            footprint,
            json!({"name":"Two pads","package":package,"pads":{
            pad_a.to_string():{"uuid":pad_a,"name":"1","position":{"x":0,"y":2000000},"padstack":stack,"layer":1},
            pad_b.to_string():{"uuid":pad_b,"name":"2","position":{"x":0,"y":-2000000},"padstack":stack,"layer":1}}}),
        ),
        (
            "entities",
            entity,
            json!({"name":"Resistor","manufacturer":"Designer","prefix":"R","gates":{
            gate.to_string():{"uuid":gate,"name":"G","unit":unit,"symbol":symbol}}}),
        ),
        (
            "parts",
            part,
            json!({"manufacturer":"Designer","mpn":"10R-0603","description":"","datasheet":"","value":"10R","entity":entity,"package":package,"default_footprint":footprint}),
        ),
        (
            "pin_pad_maps",
            map,
            json!({"part":part,"footprint":footprint,"mappings":{
            pad_a.to_string():{"gate":gate,"pin":pin_a},pad_b.to_string():{"gate":gate,"pin":pin_b}},"assign_default":true}),
        ),
    ] {
        write(
            &mut model,
            &root,
            "datum.library.create",
            json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
        );
    }
    write(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    for (reference, x) in [("R1", 0), ("R2", 10000000), ("R3", 20000000)] {
        let component = native::schematic_library::part_placement::CompletePartSelection::capture(
            &model, part, gate, symbol,
        )
        .unwrap();
        write(
            &mut model,
            &root,
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"instance_path":[],"library_symbol":symbol,"component":component,"reference":reference,"value":"10R","x_nm":x,"y_nm":0}),
        );
        for direction in [-1, 1] {
            write(
                &mut model,
                &root,
                "datum.schematic.draw_wire",
                json!({"sheet":sheet,"instance_path":[],"points":[
                {"x":x,"y":direction * 2000000},{"x":x+2000000,"y":direction * 2000000},{"x":x+2000000,"y":direction * 6000000}]}),
            );
        }
    }
    for y in [-6000000, 6000000] {
        write(
            &mut model,
            &root,
            "datum.schematic.draw_wire",
            json!({"sheet":sheet,"instance_path":[],"points":[{"x":2000000,"y":y},{"x":22000000,"y":y}]}),
        );
    }
    let prepared = native::board_layout::source_profile::build_adopt_board_source_profile(
        &model,
        provenance(),
        genesis.board_uuid,
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let request = TransferRequest {
        sheet,
        choices: vec![],
        edit_seed: Uuid::new_v4(),
        origin: Point::zero(),
        pitch_nm: 10000000,
        layer: 1,
    };
    let prepared = prepare_transfer(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let projection = eda_engine::substrate::component_pad_source::component_pad_projection(
        &model,
        &board(&model),
    )
    .unwrap();
    for component in model.component_instances.values() {
        let maps: Vec<_> = component
            .library_bindings
            .values()
            .filter(|binding| {
                binding.binding_role == eda_engine::substrate::LibraryBindingRole::PinPadMap
            })
            .collect();
        assert_eq!(maps.len(), 1);
        assert_eq!(maps[0].target_object_id, map);
    }
    let raw = board(&model);
    for sources in raw["component_pads"].as_object().unwrap().values() {
        let pads: Vec<eda_engine::substrate::component_pad_source::ComponentPadSource> =
            serde_json::from_value(sources.clone()).unwrap();
        assert!(
            pads.iter().all(
                |pad| pad.source_binding.as_ref().unwrap().land_pattern.object_id == footprint
            )
        );
    }
    let mut groups = BTreeMap::<Uuid, Vec<_>>::new();
    for pad in projection.pads.values() {
        groups
            .entry(pad.net.unwrap())
            .or_default()
            .push(pad.clone());
    }
    assert_eq!(groups.len(), 2);
    for net in groups.keys() {
        write(
            &mut model,
            &root,
            "datum.project.create_rule",
            json!({"rule":{
            "uuid":Uuid::new_v4(),"name":"Explicit circuit clearance","scope":{"Net":net},
            "priority":0,"enabled":true,"rule_type":"clearance_copper","params":{"min_nm":100000}}}),
        );
    }
    for mut pads in groups.into_values() {
        pads.sort_by_key(|pad| pad.position.x);
        assert_eq!(pads.len(), 3);
        for pair in pads.windows(2) {
            let basis =
                native::board_routing::route::capture_route_basis(&model, pair[0].uuid, 1).unwrap();
            write(
                &mut model,
                &root,
                "datum.pcb.draw_track",
                json!({"basis":basis,"end_pad":pair[1].uuid,
                "points":[pair[0].position,pair[1].position],"width_nm":200000,"edit_seed":Uuid::new_v4()}),
            );
        }
    }
    Circuit { root, model, sheet }
}
pub(super) fn board(model: &DesignModel) -> Value {
    model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap()
}
pub(super) fn rewire(circuit: &mut Circuit) {
    rewire_to(circuit, -6000000);
}
pub(super) fn rewire_to(circuit: &mut Circuit, destination_y: i64) {
    let snapshot = ElectricalSelectionSnapshot::capture(&circuit.model).unwrap();
    let sheet = &snapshot.schematic_source().unwrap().sheets[&circuit.sheet];
    let anchors: Vec<_> = circuit
        .model
        .electrical_identities
        .values()
        .filter_map(|r| {
            if let ElectricalIdentity::Net {
                anchor,
                retired: false,
                ..
            } = &r.identity
            {
                Some(anchor.source_id)
            } else {
                None
            }
        })
        .collect();
    // Pick a branch whose pin and source edges are not identity anchors.
    let wire = sheet.wires.values().find(|wire| {
        wire.from.y == 2000000
            && wire.to.y == 6000000
            && !anchors.contains(&wire.uuid)
            && !sheet
                .symbols
                .values()
                .flat_map(|symbol| &symbol.pins)
                .any(|pin| {
                    pin.position == Point::new(wire.from.x - 2000000, 2000000)
                        && anchors.contains(&pin.uuid)
                })
            && !sheet
                .wires
                .values()
                .any(|stub| stub.to == wire.from && anchors.contains(&stub.uuid))
    });
    let wire = wire.unwrap().clone();
    let delete = native::schematic_connectivity::build_delete_schematic_wire(
        &circuit.model,
        provenance(),
        circuit.sheet,
        &wire,
    )
    .unwrap();
    let create = build(
        &circuit.model,
        &circuit.root,
        "datum.schematic.draw_wire",
        json!({
        "sheet":circuit.sheet,"instance_path":[],"points":[wire.from,
            {"x":wire.from.x,"y":1000000},
            {"x":wire.from.x+if wire.from.x>12000000 {-1000000} else {1000000},"y":1000000},
            {"x":wire.from.x+if wire.from.x>12000000 {-1000000} else {1000000},"y":destination_y}]}),
    );
    let prepared = BatchComposer::compose(&circuit.model, provenance())
        .push_ops(delete.batch.operations)
        .push_ops(create.batch.operations)
        .finish()
        .unwrap();
    commit_prepared(&mut circuit.model, &circuit.root, prepared).unwrap();
}

fn rectangle(x: i64, y: i64, w: i64, h: i64) -> eda_engine::ir::geometry::Polygon {
    eda_engine::ir::geometry::Polygon::new(vec![
        Point::new(x, y),
        Point::new(x + w, y),
        Point::new(x + w, y + h),
        Point::new(x, y + h),
    ])
}

pub(super) fn add_filled_zone(circuit: &mut Circuit) -> Uuid {
    let id = add_zone(circuit);
    refill(circuit, id);
    id
}

pub(super) fn add_zone(circuit: &mut Circuit) -> Uuid {
    use native::board_routing::zone;
    let source = board(&circuit.model);
    let board_id = serde_json::from_value(source["uuid"].clone()).unwrap();
    let outline = native::board_layout::build_set_board_outline(
        &circuit.model,
        provenance(),
        board_id,
        &rectangle(-10000000, -10000000, 40000000, 20000000),
    )
    .unwrap();
    commit_prepared(&mut circuit.model, &circuit.root, outline).unwrap();
    let projection = eda_engine::substrate::component_pad_source::component_pad_projection(
        &circuit.model,
        &board(&circuit.model),
    )
    .unwrap();
    let net = projection
        .pads
        .values()
        .find(|pad| pad.position.y > 0)
        .unwrap()
        .net
        .unwrap();
    let request = zone::ZoneRequest {
        basis: zone::capture_zone_basis(&circuit.model, net, 1).unwrap(),
        polygon: rectangle(-1000000, 3000000, 22000000, 2000000),
        priority: 1,
        thermal_relief: false,
        thermal_gap_nm: 0,
        thermal_spoke_width_nm: 0,
        edit_seed: Uuid::new_v4(),
    };
    let prepared = zone::prepare_zone(&circuit.model, provenance(), &request).unwrap();
    let id = prepared.primary_object_id.unwrap();
    commit_prepared(&mut circuit.model, &circuit.root, prepared).unwrap();
    id
}

pub(super) fn refill(circuit: &mut Circuit, id: Uuid) {
    use native::board_routing::zone;
    let request = zone::FillRequest {
        basis: zone::capture_board_basis(&circuit.model).unwrap(),
        zone: Some(id),
        net: None,
    };
    let prepared = zone::prepare_fill(&circuit.model, provenance(), &request).unwrap();
    commit_prepared(&mut circuit.model, &circuit.root, prepared).unwrap();
    assert!(
        circuit.model.current_zone_fill(id).is_some(),
        "explicit ordinary refill must produce current copper"
    );
}

fn route(circuit: &mut Circuit, start: Uuid, end: Uuid, points: Vec<Point>) {
    let basis =
        native::board_routing::route::capture_route_basis(&circuit.model, start, 1).unwrap();
    write(
        &mut circuit.model,
        &circuit.root,
        "datum.pcb.draw_track",
        json!({"basis":basis,
        "end_pad":end,"points":points,"width_nm":200000,"edit_seed":Uuid::new_v4()}),
    );
}

/// Use existing deletion/routing/refill builders, never change copper assignment.
pub(super) fn repair(circuit: &mut Circuit, zone: Uuid) {
    let source = board(&circuit.model);
    let mut delete = BatchComposer::compose(&circuit.model, provenance());
    for (id, track) in source["tracks"].as_object().unwrap() {
        let prepared = native::board_routing::build_delete_board_track(
            &circuit.model,
            provenance(),
            id.parse().unwrap(),
            track.clone(),
        )
        .unwrap();
        delete = delete.push_ops(prepared.batch.operations);
    }
    let prepared = delete.finish().unwrap();
    commit_prepared(&mut circuit.model, &circuit.root, prepared).unwrap();
    let projection = eda_engine::substrate::component_pad_source::component_pad_projection(
        &circuit.model,
        &board(&circuit.model),
    )
    .unwrap();
    let mut groups = BTreeMap::<Uuid, Vec<_>>::new();
    for pad in projection.pads.values() {
        groups
            .entry(pad.net.unwrap())
            .or_default()
            .push(pad.clone());
    }
    let mut upper = groups
        .values()
        .find(|pads| pads.len() == 2)
        .unwrap()
        .clone();
    upper.sort_by_key(|pad| pad.position.x);
    route(
        circuit,
        upper[0].uuid,
        upper[1].uuid,
        vec![
            upper[0].position,
            Point::new(upper[0].position.x, 6000000),
            Point::new(upper[1].position.x, 6000000),
            upper[1].position,
        ],
    );
    let lower = groups.values().find(|pads| pads.len() == 4).unwrap();
    let changed = lower.iter().find(|pad| pad.position.y > 0).unwrap();
    let mut bottom: Vec<_> = lower.iter().filter(|pad| pad.position.y < 0).collect();
    bottom.sort_by_key(|pad| pad.position.x);
    for pair in bottom.windows(2) {
        refill(circuit, zone);
        route(
            circuit,
            pair[0].uuid,
            pair[1].uuid,
            vec![pair[0].position, pair[1].position],
        );
    }
    let start = bottom
        .iter()
        .find(|pad| pad.position.x == changed.position.x)
        .unwrap();
    refill(circuit, zone);
    route(
        circuit,
        start.uuid,
        changed.uuid,
        vec![start.position, changed.position],
    );
    refill(circuit, zone);
}

pub(super) fn connectivity_codes(circuit: &Circuit) -> Vec<String> {
    connectivity_findings(circuit)
        .into_iter()
        .map(|finding| finding.code)
        .collect()
}
pub(super) fn connectivity_findings(circuit: &Circuit) -> Vec<eda_engine::drc::DrcViolation> {
    use std::process::Command;
    // The proof caller qualifies this exact same-candidate release companion.
    // No fallback, Cargo invocation, schema repair or local Board reconstruction.
    let cli = PathBuf::from(
        std::env::var_os("DATUM_CONNECTIVITY_DRC_CLI")
            .expect("Qualified matching native CLI is required for ordinary DRC"),
    );
    let expected = std::env::var("DATUM_CONNECTIVITY_DRC_CLI_SHA256").unwrap();
    assert_eq!(expected.len(), 64);
    let hash = Command::new("sha256sum")
        .arg("--")
        .arg(&cli)
        .output()
        .unwrap();
    assert!(hash.status.success());
    assert_eq!(
        String::from_utf8(hash.stdout)
            .unwrap()
            .split_whitespace()
            .next(),
        Some(expected.as_str())
    );
    let output = Command::new(&cli)
        .args(["--format", "json", "project", "query"])
        .arg(&circuit.root)
        .arg("drc")
        .output()
        .unwrap();
    // This read-only query returns zero with diagnostic findings; transport or
    // assembly failure is not a successful unavailable-connectivity finding.
    assert!(
        output.status.success(),
        "Native DRC query failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["project_id"],
        json!(circuit.model.project.project_id)
    );
    assert_eq!(
        report["model_revision"],
        json!(circuit.model.model_revision.0)
    );
    let findings: Vec<eda_engine::drc::DrcViolation> =
        serde_json::from_value(report["raw_report"]["drc"].clone()).unwrap();
    let cold = ProjectResolver::new(&circuit.root).resolve().unwrap();
    assert_eq!(cold.model_revision, circuit.model.model_revision);
    assert_eq!(
        cold.journal, circuit.model.journal,
        "DRC query must not journal a check run"
    );
    findings
        .into_iter()
        .filter(|finding| finding.rule_type == eda_engine::rules::ast::RuleType::Connectivity)
        .collect()
}
