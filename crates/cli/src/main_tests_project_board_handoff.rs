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

fn add_local_pool_ref(root: &Path) {
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

fn write_pool_part_with_default_footprint(
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
