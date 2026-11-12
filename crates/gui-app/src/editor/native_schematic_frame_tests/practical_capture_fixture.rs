//! Canonical fixture preparation; these writes are not claimed as GUI drawing.
use eda_engine::{
    api::native_write::{
        WriteProvenance, commit_prepared,
        genesis::{GenesisSpec, bootstrap_native_project},
        registry::{NativeWriteContext, find_native_write_verb},
        schematic_child_sheet::ChildSheetRequest,
        schematic_connectivity::build_create_schematic_port,
        schematic_delete_proposal::{self, NativeProposalAction, NativeProposalRequest},
        schematic_interface_binding::InterfaceBindingRequest,
        schematic_library::part_placement::PartGateSelection,
    },
    ir::geometry::Point,
    schematic::{HierarchicalPort, PortDirection},
    substrate::{CommitSource, DesignModel, ElectricalSelectionSnapshot, ProjectResolver},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub(super) struct Amplifier {
    pub root: PathBuf,
    pub parent: Uuid,
    pub child: Uuid,
    pub instance: Uuid,
    pub bias: [Uuid; 2],
    pub emitter: Uuid,
    pub fault_pin: Uuid,
    pub source_file: PathBuf,
}
pub(super) fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "test",
        CommitSource::Manual,
        "Practical native amplifier workflow",
    )
}
fn write(model: &mut DesignModel, root: &Path, verb: &str, params: Value) -> Uuid {
    *model = ProjectResolver::new(root).resolve().unwrap();
    let prepared = (find_native_write_verb(verb)
        .unwrap_or_else(|| panic!("unregistered fixture verb: {verb}"))
        .build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = prepared.primary_object_id.unwrap_or_default();
    commit_prepared(model, root, prepared).unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
    id
}
fn proposal(model: &mut DesignModel, root: &Path, action: NativeProposalAction) {
    schematic_delete_proposal::execute(
        model,
        &NativeProposalRequest {
            project_root: root.into(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: eda_engine::revision::transaction_tip(&model.journal),
            action,
        },
    )
    .unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
}
fn library(model: &mut DesignModel, root: &Path, kind: &str, id: Uuid, fields: Value) {
    write(
        model,
        root,
        "datum.library.create",
        json!({"kind":kind,"pool_path":"pool","object_id":id,"fields":fields}),
    );
}
struct Device {
    symbol: Uuid,
    entity: Uuid,
    gate: Uuid,
    package: Uuid,
}
fn device(
    model: &mut DesignModel,
    root: &Path,
    name: &str,
    prefix: &str,
    pins: &[(Uuid, &str, i64, i64, &str)],
    drawings: Value,
) -> Device {
    let (unit, symbol, entity, gate, package) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let logical: BTreeMap<_, _> = pins
        .iter()
        .map(|(id, name, ..)| {
            (
                id.to_string(),
                json!({"name":name,"electrical_type":"Passive"}),
            )
        })
        .collect();
    library(
        model,
        root,
        "units",
        unit,
        json!({"name":name,"pins":logical}),
    );
    let anchors: Vec<_> = pins.iter().map(|(id,name,x,y,orientation)|json!({"pin":id,"number":name,"position":{"x":x,"y":y},"style":{"orientation":orientation,"length_nm":1000000,"decoration":"none"}})).collect();
    let template = json!({"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}});
    let simple = drawings
        .as_array()
        .is_some_and(|rows| rows.len() == 1 && rows[0].get("Rect").is_some());
    let initial = if simple {
        drawings.clone()
    } else {
        json!([template.clone()])
    };
    library(
        model,
        root,
        "symbols",
        symbol,
        json!({"name":name,"unit":unit,"drawings":initial,"pin_anchors":anchors}),
    );
    if !simple {
        write(
            model,
            root,
            "datum.library.edit",
            json!({"kind":"symbols","pool_path":"pool","object_id":symbol,"fields":{"rect_removals":[{"index":0,"previous":template}],"graphic_additions":drawings}}),
        );
    }
    library(
        model,
        root,
        "packages",
        package,
        json!({"name":format!("{name} logical package")}),
    );
    library(
        model,
        root,
        "entities",
        entity,
        json!({"name":name,"manufacturer":"","prefix":prefix,"gates":{gate.to_string():{"uuid":gate,"name":"A","unit":unit,"symbol":symbol}}}),
    );
    Device {
        symbol,
        entity,
        gate,
        package,
    }
}
fn part(model: &mut DesignModel, root: &Path, device: &Device, value: &str) -> Uuid {
    let id = Uuid::new_v4();
    library(
        model,
        root,
        "parts",
        id,
        json!({"manufacturer":"Native example","mpn":value,"description":"Native common-emitter amplifier example","datasheet":"","value":value,"entity":device.entity,"package":device.package}),
    );
    id
}
fn place(
    model: &mut DesignModel,
    root: &Path,
    sheet: Uuid,
    path: &[Uuid],
    device: &Device,
    part: Uuid,
    reference: &str,
    value: &str,
    at: (i64, i64),
    rotation: i32,
    mirrored: bool,
) -> Uuid {
    let choice = PartGateSelection::capture(model, part, device.gate, device.symbol).unwrap();
    write(
        model,
        root,
        "datum.schematic.place_symbol",
        json!({"sheet":sheet,"instance_path":path,"library_symbol":device.symbol,"explicit_part":choice,"reference":reference,"value":value,"x_nm":at.0*1000000,"y_nm":at.1*1000000,"rotation_deg":rotation,"mirrored":mirrored}),
    )
}
fn wire(model: &mut DesignModel, root: &Path, sheet: Uuid, path: &[Uuid], points: &[(i64, i64)]) {
    let points: Vec<_> = points
        .iter()
        .map(|(x, y)| json!({"x":x*1000000,"y":y*1000000}))
        .collect();
    write(
        model,
        root,
        "datum.schematic.draw_wire",
        json!({"sheet":sheet,"instance_path":path,"points":points}),
    );
}
fn label(
    model: &mut DesignModel,
    root: &Path,
    sheet: Uuid,
    path: &[Uuid],
    name: &str,
    kind: &str,
    at: (i64, i64),
) {
    write(
        model,
        root,
        "datum.schematic.place_label",
        json!({"sheet":sheet,"instance_path":path,"name":name,"kind":kind,"x_nm":at.0*1000000,"y_nm":at.1*1000000}),
    );
}

pub(super) fn setup(root: &Path) -> Amplifier {
    bootstrap_native_project(
        root,
        GenesisSpec {
            project_name: "Native common-emitter amplifier".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    let eda_engine::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        eda_engine::ir::units::migrate_pre_feature_project_units(None).unwrap()
    else {
        panic!("Fresh project requires explicit unit initialization")
    };
    let prepared = eda_engine::api::native_write::project::build_initialize_project_display_units(
        &model,
        provenance(),
        profile,
        &receipt,
    )
    .unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let parent = Uuid::new_v4();
    write(
        &mut model,
        root,
        "datum.schematic.create_sheet",
        json!({"sheet":parent,"name":"Root"}),
    );
    let child = ChildSheetRequest {
        proposal_id: Uuid::new_v4(),
        parent,
        instance_path: vec![],
        sheet: Uuid::new_v4(),
        definition: Uuid::new_v4(),
        instance: Uuid::new_v4(),
        name: "Amplifier".into(),
        position: Point::new(15000000, 12000000),
    };
    proposal(
        &mut model,
        root,
        NativeProposalAction::CreateChildSheet(child.clone()),
    );
    proposal(
        &mut model,
        root,
        NativeProposalAction::AcceptApply {
            proposal_id: child.proposal_id,
        },
    );
    let path = [child.instance];
    let (r1, r2) = (Uuid::new_v4(), Uuid::new_v4());
    let resistor = device(
        &mut model,
        root,
        "Resistor",
        "R",
        &[
            (r1, "1", -2000000, 0, "Right"),
            (r2, "2", 2000000, 0, "Left"),
        ],
        json!([{"Rect":{"min":{"x":-1000000,"y":-500000},"max":{"x":1000000,"y":500000},"width":100000}}]),
    );
    let (c1, c2) = (Uuid::new_v4(), Uuid::new_v4());
    let capacitor = device(
        &mut model,
        root,
        "Coupling capacitor",
        "C",
        &[
            (c1, "1", -2000000, 0, "Right"),
            (c2, "2", 2000000, 0, "Left"),
        ],
        json!([{"Line":{"from":{"x":-1000000,"y":0},"to":{"x":-500000,"y":0},"width":100000}},{"Line":{"from":{"x":500000,"y":0},"to":{"x":1000000,"y":0},"width":100000}},{"Line":{"from":{"x":-500000,"y":-1000000},"to":{"x":-500000,"y":1000000},"width":100000}},{"Line":{"from":{"x":500000,"y":-1000000},"to":{"x":500000,"y":1000000},"width":100000}}]),
    );
    let (base, collector, emitter) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let transistor = device(
        &mut model,
        root,
        "NPN transistor",
        "Q",
        &[
            (base, "B", -2000000, 0, "Right"),
            (collector, "C", 0, -2000000, "Down"),
            (emitter, "E", 0, 2000000, "Up"),
        ],
        json!([
            {"Circle":{"center":{"x":0,"y":0},"radius":1000000,"width":100000}},
            {"Line":{"from":{"x":-1000000,"y":0},"to":{"x":-500000,"y":0},"width":100000}},
            {"Line":{"from":{"x":-500000,"y":-750000},"to":{"x":-500000,"y":750000},"width":100000}},
            {"Line":{"from":{"x":-500000,"y":-300000},"to":{"x":0,"y":-1000000},"width":100000}},
            {"Line":{"from":{"x":-500000,"y":300000},"to":{"x":0,"y":1000000},"width":100000}},
            {"Line":{"from":{"x":0,"y":1000000},"to":{"x":-400000,"y":750000},"width":100000}},
            {"Line":{"from":{"x":0,"y":1000000},"to":{"x":-50000,"y":550000},"width":100000}}
        ]),
    );
    let header = device(
        &mut model,
        root,
        "Two-pin connector",
        "J",
        &[
            (Uuid::new_v4(), "1", 2000000, 0, "Left"),
            (Uuid::new_v4(), "2", 0, 2000000, "Up"),
        ],
        json!([{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":1000000,"y":1000000},"width":100000}}]),
    );
    let (p100, p22, p47, p1) = (
        part(&mut model, root, &resistor, "100k"),
        part(&mut model, root, &resistor, "22k"),
        part(&mut model, root, &resistor, "4.7k"),
        part(&mut model, root, &resistor, "1k"),
    );
    let (pc1, pc10, pq, pj) = (
        part(&mut model, root, &capacitor, "1u"),
        part(&mut model, root, &capacitor, "10u"),
        part(&mut model, root, &transistor, "NPN"),
        part(&mut model, root, &header, "Header 2"),
    );
    let bias1 = place(
        &mut model,
        root,
        child.sheet,
        &path,
        &resistor,
        p100,
        "R1",
        "100k",
        (10, 10),
        90,
        false,
    );
    let bias2 = place(
        &mut model,
        root,
        child.sheet,
        &path,
        &resistor,
        p22,
        "R2",
        "22k",
        (11, 30),
        90,
        false,
    );
    place(
        &mut model,
        root,
        child.sheet,
        &path,
        &resistor,
        p47,
        "R3",
        "4.7k",
        (20, 10),
        90,
        false,
    );
    let re = place(
        &mut model,
        root,
        child.sheet,
        &path,
        &resistor,
        p1,
        "R4",
        "1k",
        (20, 30),
        90,
        false,
    );
    place(
        &mut model,
        root,
        child.sheet,
        &path,
        &transistor,
        pq,
        "Q1",
        "NPN",
        (20, 20),
        0,
        false,
    );
    place(
        &mut model,
        root,
        child.sheet,
        &path,
        &capacitor,
        pc1,
        "C1",
        "1u",
        (4, 20),
        0,
        false,
    );
    place(
        &mut model,
        root,
        child.sheet,
        &path,
        &capacitor,
        pc10,
        "C2",
        "10u",
        (28, 16),
        0,
        false,
    );
    // R5 lives in the parent: child copy must reserve references project-wide.
    place(
        &mut model,
        root,
        parent,
        &[],
        &resistor,
        p100,
        "R5",
        "100k",
        (34, 4),
        90,
        false,
    );
    place(
        &mut model,
        root,
        parent,
        &[],
        &header,
        pj,
        "J1",
        "IN",
        (0, 20),
        0,
        false,
    );
    place(
        &mut model,
        root,
        parent,
        &[],
        &header,
        pj,
        "J2",
        "OUT",
        (40, 20),
        0,
        true,
    );
    place(
        &mut model,
        root,
        parent,
        &[],
        &header,
        pj,
        "J3",
        "VCC",
        (20, 0),
        0,
        false,
    );
    for points in [
        vec![(10, 4), (10, 8)],
        vec![(10, 12), (10, 20)],
        vec![(10, 20), (11, 28)],
        vec![(11, 32), (10, 38)],
        vec![(20, 4), (20, 8)],
        vec![(20, 12), (20, 18)],
        vec![(20, 22), (20, 28)],
        vec![(18, 20), (10, 20)],
        vec![(0, 20), (2, 20)],
        vec![(6, 20), (10, 20)],
        vec![(20, 16), (26, 16)],
        vec![(30, 16), (34, 16)],
    ] {
        wire(&mut model, root, child.sheet, &path, &points);
    }
    for (name, kind, at) in [
        ("VCC", "global", (10, 4)),
        ("VCC", "global", (20, 4)),
        ("GND", "global", (10, 38)),
        ("AMP_IN", "hierarchical", (0, 20)),
        ("AMP_OUT", "hierarchical", (34, 16)),
    ] {
        label(&mut model, root, child.sheet, &path, name, kind, at);
    }
    for points in [
        vec![(2, 20), (10, 20)],
        vec![(30, 20), (38, 20)],
        vec![(0, 22), (0, 30)],
        vec![(40, 22), (40, 30)],
        vec![(22, 0), (26, 0)],
        vec![(20, 2), (20, 6)],
        vec![(34, 0), (34, 2)],
        vec![(34, 6), (34, 8)],
    ] {
        wire(&mut model, root, parent, &[], &points);
    }
    for (name, at) in [
        ("GND", (0, 30)),
        ("GND", (40, 30)),
        ("VCC", (26, 0)),
        ("GND", (20, 6)),
        ("VCC", (34, 0)),
        ("GND", (34, 8)),
    ] {
        label(&mut model, root, parent, &[], name, "global", at);
    }
    for (name, at) in [("AMP_IN", (10, 20)), ("AMP_OUT", (30, 20))] {
        let port = HierarchicalPort {
            uuid: Uuid::new_v4(),
            name: name.into(),
            direction: PortDirection::Passive,
            position: Point::new(at.0 * 1000000, at.1 * 1000000),
        };
        let prepared = build_create_schematic_port(&model, provenance(), parent, &port).unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
        model = ProjectResolver::new(root).resolve().unwrap();
        let binding = InterfaceBindingRequest {
            proposal_id: Uuid::new_v4(),
            instance: child.instance,
            instance_path: path.to_vec(),
            parent_port: port.uuid,
            bind: true,
        };
        proposal(
            &mut model,
            root,
            NativeProposalAction::CreateInterfaceBinding(binding.clone()),
        );
        proposal(
            &mut model,
            root,
            NativeProposalAction::AcceptApply {
                proposal_id: binding.proposal_id,
            },
        );
    }
    let source = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let fault_pin = source.schematic_source().unwrap().sheets[&child.sheet].symbols[&re]
        .pins
        .iter()
        .find(|pin| pin.library_pin == Some(r2))
        .unwrap()
        .uuid;
    assert_eq!(model.component_instances.len(), 11);
    let source_file = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&re].source_shard_id)
        .unwrap()
        .path
        .clone();
    Amplifier {
        root: root.into(),
        parent,
        child: child.sheet,
        instance: child.instance,
        bias: [bias1, bias2],
        emitter: re,
        fault_pin,
        source_file,
    }
}
