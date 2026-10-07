use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn seed_native_sheet(root: &Path) -> Uuid {
    let sheet_uuid = Uuid::new_v4();
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
                "name": "Main",
                "frame": null,
                "symbols": {},
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

    sheet_uuid
}

pub(super) fn add_local_pool_ref(root: &Path) {
    let project_json = root.join("project.json");
    let mut project_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&project_json).expect("project.json should read"),
    )
    .expect("project.json should parse");
    project_value["pools"] = serde_json::json!([{ "path": "pool", "priority": 0 }]);
    std::fs::write(
        &project_json,
        format!(
            "{}\n",
            to_json_deterministic(&project_value).expect("canonical serialization should succeed")
        ),
    )
    .expect("project.json should write");
}

pub(super) fn write_pool_part_with_default_footprint(
    root: &Path,
    part_uuid: Uuid,
    package_uuid: Uuid,
    footprint_uuid: Uuid,
) {
    std::fs::create_dir_all(root.join("pool/parts")).expect("parts dir should create");
    std::fs::create_dir_all(root.join("pool/footprints")).expect("footprints dir should create");
    std::fs::write(
        root.join("pool/parts").join(format!("{part_uuid}.json")),
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "uuid": part_uuid,
                "entity": Uuid::new_v4(),
                "package": Uuid::new_v4(),
                "default_footprint": footprint_uuid,
                "default_pin_pad_map": null,
                "pad_map": {},
                "mpn": "TEST-1",
                "manufacturer": "Datum",
                "manufacturer_jep106": null,
                "value": "10k",
                "description": "test part",
                "datasheet": "",
                "parametric": {},
                "orderable_mpns": [],
                "packaging_options": [],
                "tags": [],
                "lifecycle": "Active",
                "base": null,
                "behavioural_models": [],
                "thermal": null,
                "supply_chain_offers": null,
                "last_supply_chain_check": null
            }))
            .expect("part JSON should serialize")
        ),
    )
    .expect("part should write");
    std::fs::write(
        root.join("pool/footprints")
            .join(format!("{footprint_uuid}.json")),
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "uuid": footprint_uuid,
                "name": "FP",
                "package": package_uuid,
                "pads": {},
                "courtyard": { "vertices": [], "closed": true },
                "silkscreen": [],
                "fab": [],
                "assembly": [],
                "mechanical": [],
                "models_3d": [],
                "standards_basis": null,
                "process_aperture_policy": null,
                "tags": []
            }))
            .expect("footprint JSON should serialize")
        ),
    )
    .expect("footprint should write");
}

// Extend this historical handoff recipe with real source-backed copper Pads.
pub(super) fn install_supported_handoff_pads(root: &Path, package: Uuid, footprint: Uuid) -> Uuid {
    let pad = Uuid::new_v4();
    let stack = Uuid::new_v4();
    let write = |path: PathBuf, value: serde_json::Value| {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            path,
            format!("{}\n", to_json_deterministic(&value).unwrap()),
        )
        .unwrap();
    };
    write(
        root.join(format!("pool/packages/{package}.json")),
        serde_json::json!({
            "schema_version":1, "uuid":package, "name":"HANDOFF", "pads":{},
            "courtyard":{"vertices":[],"closed":true},"silkscreen":[],"models_3d":[],"tags":[]
        }),
    );
    write(
        root.join(format!("pool/padstacks/{stack}.json")),
        serde_json::json!({
            "schema_version":1, "uuid":stack, "name":"PAD",
            "aperture":{"circle":{"diameter_nm":600000}},"drill_nm":null,"plated":false
        }),
    );
    let path = root.join(format!("pool/footprints/{footprint}.json"));
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value["schema_version"] = 1.into();
    value["pads"] = serde_json::json!({pad.to_string():{
        "uuid":pad,"name":"1","position":{"x":100000,"y":200000},"padstack":stack,"layer":1
    }});
    write(path, value);
    let path = root.join("board/board.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value["stackup"] = serde_json::json!({"layers":[
        {"id":1,"name":"F.Cu","layer_type":"Copper","thickness_nm":35000},
        {"id":2,"name":"B.Cu","layer_type":"Copper","thickness_nm":35000},
        {"id":21,"name":"F.SilkS","layer_type":"Silkscreen","thickness_nm":0},
        {"id":22,"name":"B.SilkS","layer_type":"Silkscreen","thickness_nm":0}
    ]});
    write(path, value);
    pad
}

fn place_symbol_with_part(root: &Path, sheet_uuid: Uuid, part_uuid: Uuid) -> String {
    let place_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-symbol",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--reference",
            "R1",
            "--value",
            "10k",
            "--x-nm",
            "10",
            "--y-nm",
            "20",
        ])
        .expect("CLI should parse"),
    )
    .expect("place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol output should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "set-symbol-part",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_uuid,
            "--part",
            &part_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("set-symbol-part should succeed");
    symbol_uuid
}

#[test]
fn project_generate_board_components_reports_and_applies_initial_packages() {
    let root = unique_project_root("datum-eda-cli-project-board-handoff");
    create_native_project(&root, Some("Board Handoff".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    add_local_pool_ref(&root);
    let part_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();
    let footprint_uuid = Uuid::new_v4();
    write_pool_part_with_default_footprint(&root, part_uuid, package_uuid, footprint_uuid);
    let library_pad = install_supported_handoff_pads(&root, package_uuid, footprint_uuid);
    let initial_board = std::fs::read(root.join("board/board.json")).unwrap();
    let symbol_uuid = place_symbol_with_part(&root, sheet_uuid, part_uuid);

    let dry_run = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "generate-board-components",
            root.to_str().unwrap(),
            "--origin-x-nm",
            "1000",
            "--origin-y-nm",
            "2000",
            "--pitch-nm",
            "3000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("generate-board-components dry-run should succeed");
    let dry_report: serde_json::Value =
        serde_json::from_str(&dry_run).expect("dry-run output should parse");
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        initial_board
    );
    assert!(dry_report["source_profile_adoption_transaction"].is_null());
    assert_eq!(dry_report["applied"], false);
    assert_eq!(dry_report["generated_count"], 1);
    assert_eq!(
        dry_report["generated_packages"][0]["symbol_uuid"],
        symbol_uuid
    );
    assert_eq!(
        dry_report["generated_packages"][0]["package_ref_uuid"],
        package_uuid.to_string()
    );
    assert_eq!(
        dry_report["generated_packages"][0]["default_footprint_uuid"],
        footprint_uuid.to_string()
    );
    assert_eq!(
        dry_report["relationship_diagnostics"][0]["code"],
        "component_instance_unmatched_symbol"
    );

    let apply = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "generate-board-components",
            root.to_str().unwrap(),
            "--apply",
            "--origin-x-nm",
            "1000",
            "--origin-y-nm",
            "2000",
        ])
        .expect("CLI should parse"),
    )
    .expect("generate-board-components apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply).expect("apply output should parse");
    assert_eq!(apply_report["applied"], true);
    assert_eq!(apply_report["generated_count"], 1);
    assert!(apply_report["source_profile_adoption_transaction"].is_string());
    let source: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    let key = apply_report["generated_packages"][0]["package_uuid"]
        .as_str()
        .unwrap();
    let placed_pad = &source["component_pads"][key][0];
    assert_ne!(placed_pad["uuid"], library_pad.to_string());
    assert_eq!(
        placed_pad["source_binding"]["land_pattern"]["object_id"],
        footprint_uuid.to_string()
    );

    let components_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-components",
        ])
        .expect("CLI should parse"),
    )
    .expect("board-components query should succeed");
    let components: serde_json::Value =
        serde_json::from_str(&components_output).expect("components output should parse");
    assert_eq!(components.as_array().unwrap().len(), 1);
    assert_eq!(components[0]["reference"], "R1");
    assert_eq!(components[0]["part"], part_uuid.to_string());
    assert_eq!(components[0]["package"], package_uuid.to_string());
    assert_eq!(components[0]["position"]["x"], 1000);
    assert_eq!(components[0]["position"]["y"], 2000);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_generate_board_components_can_create_and_apply_proposal() {
    let root = unique_project_root("datum-eda-cli-project-board-handoff-proposal");
    create_native_project(&root, Some("Board Handoff Proposal".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    add_local_pool_ref(&root);
    let part_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();
    let footprint_uuid = Uuid::new_v4();
    write_pool_part_with_default_footprint(&root, part_uuid, package_uuid, footprint_uuid);
    let library_pad = install_supported_handoff_pads(&root, package_uuid, footprint_uuid);
    let initial_board = std::fs::read(root.join("board/board.json")).unwrap();
    let symbol_uuid = place_symbol_with_part(&root, sheet_uuid, part_uuid);
    let proposal_id = Uuid::new_v4();

    let proposal_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "generate-board-components",
            root.to_str().unwrap(),
            "--as-proposal",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "Review schematic handoff package generation",
            "--origin-x-nm",
            "4000",
            "--origin-y-nm",
            "5000",
        ])
        .expect("CLI should parse"),
    )
    .expect("generate-board-components proposal should succeed");
    let proposal_report: serde_json::Value =
        serde_json::from_str(&proposal_output).expect("proposal output should parse");
    assert_eq!(proposal_report["applied"], false);
    assert_eq!(proposal_report["proposed"], true);
    assert_eq!(proposal_report["proposal_id"], proposal_id.to_string());
    assert_eq!(proposal_report["generated_count"], 1);
    assert!(proposal_report["source_profile_adoption_transaction"].is_string());
    let adopted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    let mut expected: serde_json::Value = serde_json::from_slice(&initial_board).unwrap();
    expected["schema_version"] = 4.into();
    assert_eq!(
        adopted, expected,
        "draft writes only the separate profile adoption, not Pad materialization"
    );
    let proposed_pad =
        &proposal_report["proposal"]["batch"]["operations"][0]["materialized"]["component_pads"][0];
    assert_ne!(proposed_pad["uuid"], library_pad.to_string());
    assert_eq!(
        proposed_pad["source_binding"]["land_pattern"]["object_id"],
        footprint_uuid.to_string()
    );
    assert_eq!(
        proposal_report["generated_packages"][0]["symbol_uuid"],
        symbol_uuid
    );
    assert_eq!(
        proposal_report["proposal"]["batch"]["operations"][0]["kind"],
        "create_board_package"
    );
    assert_eq!(
        proposal_report["proposal"]["batch"]["operations"][0]["package_id"],
        proposal_report["generated_packages"][0]["package_uuid"]
    );

    let components_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-components",
        ])
        .expect("CLI should parse"),
    )
    .expect("board-components query should succeed");
    let components: serde_json::Value =
        serde_json::from_str(&components_output).expect("components output should parse");
    assert_eq!(
        components.as_array().unwrap().len(),
        0,
        "proposal creation must not mutate board packages"
    );

    let preview_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "preview",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal preview should succeed");
    let preview: serde_json::Value =
        serde_json::from_str(&preview_output).expect("preview output should parse");
    assert_eq!(preview["proposal_id"], proposal_id.to_string());
    assert_eq!(preview["validation"]["can_apply"], false);
    assert_eq!(preview["validation"]["acceptance_required"], true);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal accept-apply should succeed");
    let source: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    let key = proposal_report["generated_packages"][0]["package_uuid"]
        .as_str()
        .unwrap();
    assert_eq!(
        source["component_pads"][key][0]["uuid"],
        proposed_pad["uuid"]
    );

    let components_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-components",
        ])
        .expect("CLI should parse"),
    )
    .expect("board-components query should succeed");
    let components: serde_json::Value =
        serde_json::from_str(&components_output).expect("components output should parse");
    assert_eq!(components.as_array().unwrap().len(), 1);
    assert_eq!(components[0]["reference"], "R1");
    assert_eq!(components[0]["part"], part_uuid.to_string());
    assert_eq!(components[0]["package"], package_uuid.to_string());
    assert_eq!(components[0]["position"]["x"], 4000);
    assert_eq!(components[0]["position"]["y"], 5000);

    let _ = std::fs::remove_dir_all(&root);
}
