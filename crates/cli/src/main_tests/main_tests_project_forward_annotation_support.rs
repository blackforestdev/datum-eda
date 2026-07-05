use std::collections::BTreeMap;

use super::*;
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::schematic::{HiddenPowerBehavior, PlacedSymbol, SymbolDisplayMode};

pub(super) fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

pub(super) fn native_symbol(
    uuid: Uuid,
    reference: &str,
    value: &str,
    lib_id: &str,
    part: Option<Uuid>,
    entity: Option<Uuid>,
) -> PlacedSymbol {
    PlacedSymbol {
        uuid,
        part,
        entity,
        gate: None,
        lib_id: Some(lib_id.to_string()),
        reference: reference.to_string(),
        value: value.to_string(),
        fields: Vec::new(),
        pins: Vec::new(),
        position: Point::new(0, 0),
        rotation: 0,
        mirrored: false,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: Vec::new(),
        hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
    }
}

pub(super) fn write_native_sheet_symbols(
    root: &Path,
    sheet_uuid: Uuid,
    sheet_name: &str,
    symbols: Vec<PlacedSymbol>,
) {
    let symbols = symbols
        .into_iter()
        .map(|symbol| {
            (
                symbol.uuid.to_string(),
                serde_json::to_value(symbol).expect("symbol should serialize"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    write_native_sheet(root, sheet_uuid, sheet_name, symbols);
}

pub(super) fn write_native_sheet(
    root: &Path,
    sheet_uuid: Uuid,
    sheet_name: &str,
    symbols: BTreeMap<String, serde_json::Value>,
) {
    let sheet_path = root
        .join("schematic/sheets")
        .join(format!("{sheet_uuid}.json"));
    std::fs::write(
        &sheet_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": sheet_uuid,
                "name": sheet_name,
                "frame": null,
                "symbols": symbols,
                "wires": {},
                "junctions": {},
                "labels": {},
                "buses": {},
                "bus_entries": {},
                "ports": {},
                "noconnects": {},
                "texts": {},
                "drawings": {}
            }))
            .expect("sheet JSON should serialize")
        ),
    )
    .expect("sheet file should write");

    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    schematic_value["sheets"] = serde_json::json!({
        sheet_uuid.to_string(): format!("sheets/{sheet_uuid}.json")
    });
    std::fs::write(
        &schematic_json,
        format!(
            "{}\n",
            to_json_deterministic(&schematic_value)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("schematic.json should write");
}

pub(super) fn write_board_packages(root: &Path, board_name: &str, packages: Vec<PlacedPackage>) {
    let packages = packages
        .into_iter()
        .map(|package| {
            (
                package.uuid.to_string(),
                serde_json::to_value(package).expect("component should serialize"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": board_name,
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": packages,
                "component_silkscreen": {},
                "component_silkscreen_texts": {},
                "component_silkscreen_arcs": {},
                "component_silkscreen_circles": {},
                "component_silkscreen_polygons": {},
                "component_silkscreen_polylines": {},
                "component_mechanical_lines": {},
                "component_mechanical_texts": {},
                "component_mechanical_polygons": {},
                "component_mechanical_polylines": {},
                "component_mechanical_circles": {},
                "component_mechanical_arcs": {},
                "component_pads": {},
                "component_models_3d": {},
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");
}

pub(super) fn query_forward_annotation_proposal(root: &Path) -> serde_json::Value {
    let proposal_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "forward-annotation-proposal",
    ])
    .expect("CLI should parse");
    let proposal_output = execute(proposal_cli).expect("proposal should succeed");
    serde_json::from_str(&proposal_output).expect("proposal JSON")
}

pub(super) fn find_action_id(
    proposal: &serde_json::Value,
    action: &str,
    reason: Option<&str>,
) -> String {
    proposal["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            entry["action"] == action
                && reason
                    .map(|expected| entry["reason"] == expected)
                    .unwrap_or(true)
        })
        .unwrap()["action_id"]
        .as_str()
        .unwrap()
        .to_string()
}

pub(super) fn query_board_components(root: &Path) -> Vec<PlacedPackage> {
    let components_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-components",
    ])
    .expect("CLI should parse");
    let components_output = execute(components_cli).expect("components query should succeed");
    serde_json::from_str(&components_output).expect("components parse")
}

pub(super) fn write_component_instance_shard(
    root: &Path,
    component_instance_id: Uuid,
    symbol_id: Uuid,
    package_id: Uuid,
) {
    write_component_instance_shard_with_part_ref(
        root,
        component_instance_id,
        symbol_id,
        package_id,
        None,
    );
}

pub(super) fn write_component_instance_shard_with_part_ref(
    root: &Path,
    component_instance_id: Uuid,
    symbol_id: Uuid,
    package_id: Uuid,
    part_id: Option<Uuid>,
) {
    let path = root.join(format!(
        ".datum/component_instances/{component_instance_id}.json"
    ));
    std::fs::create_dir_all(path.parent().unwrap()).expect("component instance dir should create");
    let part_ref = part_id.map(|part_id| {
        serde_json::json!({
            "object_id": part_id,
            "object_revision": 0
        })
    });
    std::fs::write(
        &path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "component_instance": {
                    "uuid": component_instance_id,
                    "object_revision": 0,
                    "part_ref": part_ref,
                    "placed_symbol_refs": [{
                        "object_id": symbol_id,
                        "object_revision": 0
                    }],
                    "placed_package_refs": [{
                        "object_id": package_id,
                        "object_revision": 0
                    }]
                }
            }))
            .expect("component instance JSON should serialize")
        ),
    )
    .expect("component instance shard should write");
}

pub(super) fn write_pool_part_object(root: &Path, part_id: Uuid) {
    let project_path = root.join("project.json");
    let mut project: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&project_path).expect("project JSON should read"))
            .expect("project JSON should parse");
    project["pools"] = serde_json::json!([{ "path": "pool", "priority": 1 }]);
    std::fs::write(
        &project_path,
        format!(
            "{}\n",
            to_json_deterministic(&project).expect("project JSON should serialize")
        ),
    )
    .expect("project JSON should write");
    let part_path = root.join(format!("pool/parts/{part_id}.json"));
    std::fs::create_dir_all(part_path.parent().unwrap()).expect("pool part dir should create");
    std::fs::write(
        &part_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": part_id,
                "object_revision": 0,
                "entity": Uuid::new_v5(&part_id, b"entity"),
                "package": Uuid::new_v5(&part_id, b"package"),
                "mpn": "TEST-PART",
                "manufacturer": "Datum",
                "value": "TEST",
                "description": "",
                "datasheet": "",
                "lifecycle": "Active"
            }))
            .expect("pool part JSON should serialize")
        ),
    )
    .expect("pool part should write");
}
