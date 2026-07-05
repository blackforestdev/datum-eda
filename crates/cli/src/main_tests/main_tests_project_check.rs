use super::main_tests_project_proposals::write_legacy_proposal_sidecar;
use super::*;
use eda_engine::ir::geometry::Point;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::schematic::{
    CheckDomain, HiddenPowerBehavior, LabelKind, NetLabel, PinElectricalType, PlacedSymbol,
    SymbolDisplayMode, SymbolPin, WaiverTarget,
};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, OperationBatch, ProjectResolver, Proposal, ProposalSource,
    ProposalStatus,
};
use std::collections::BTreeMap;

pub(super) fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_native_sheet(
    root: &Path,
    sheet_uuid: Uuid,
    sheet_name: &str,
    symbols: BTreeMap<String, serde_json::Value>,
    labels: BTreeMap<String, serde_json::Value>,
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
                "labels": labels,
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

pub(super) fn write_native_waivers(root: &Path, waivers: &[serde_json::Value]) {
    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    schematic_value["waivers"] = serde_json::Value::Array(waivers.to_vec());
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

pub(super) fn build_native_check_fixture(root: &Path) -> (Uuid, Uuid) {
    let sheet_uuid = Uuid::new_v4();
    let passive_pin_uuid = Uuid::new_v4();
    let power_pin_uuid = Uuid::new_v4();
    let power_label_uuid = Uuid::new_v4();

    write_native_sheet(
        root,
        sheet_uuid,
        "Root",
        BTreeMap::from([
            (
                Uuid::new_v4().to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: Uuid::new_v4(),
                    part: None,
                    entity: None,
                    gate: None,
                    lib_id: Some("Device:R".into()),
                    reference: "R1".into(),
                    value: "10k".into(),
                    fields: Vec::new(),
                    pins: vec![SymbolPin {
                        uuid: passive_pin_uuid,
                        number: "1".into(),
                        name: "~".into(),
                        electrical_type: PinElectricalType::Passive,
                        position: Point::new(5, 5),
                    }],
                    position: Point::new(0, 0),
                    rotation: 0,
                    mirrored: false,
                    unit_selection: None,
                    display_mode: SymbolDisplayMode::LibraryDefault,
                    pin_overrides: Vec::new(),
                    hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
                })
                .expect("symbol should serialize"),
            ),
            (
                Uuid::new_v4().to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: Uuid::new_v4(),
                    part: None,
                    entity: None,
                    gate: None,
                    lib_id: Some("power:VCC".into()),
                    reference: "PWR1".into(),
                    value: "VCC".into(),
                    fields: Vec::new(),
                    pins: vec![SymbolPin {
                        uuid: power_pin_uuid,
                        number: "1".into(),
                        name: "VCC".into(),
                        electrical_type: PinElectricalType::PowerIn,
                        position: Point::new(20, 20),
                    }],
                    position: Point::new(0, 0),
                    rotation: 0,
                    mirrored: false,
                    unit_selection: None,
                    display_mode: SymbolDisplayMode::LibraryDefault,
                    pin_overrides: Vec::new(),
                    hidden_power_behavior: HiddenPowerBehavior::ExplicitPowerObject,
                })
                .expect("symbol should serialize"),
            ),
        ]),
        BTreeMap::from([(
            power_label_uuid.to_string(),
            serde_json::to_value(NetLabel {
                uuid: power_label_uuid,
                kind: LabelKind::Power,
                name: "VCC".into(),
                position: Point::new(20, 20),
            })
            .expect("label should serialize"),
        )]),
    );

    (passive_pin_uuid, power_pin_uuid)
}

fn seed_board_drc_fixture(root: &Path) -> Uuid {
    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let package_a_uuid = Uuid::new_v4();
    let package_b_uuid = Uuid::new_v4();
    let pad_a_uuid = Uuid::new_v4();
    let pad_b_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Check Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    package_a_uuid.to_string(): {
                        "uuid": package_a_uuid,
                        "part": Uuid::new_v4(),
                        "package": Uuid::new_v4(),
                        "reference": "R1",
                        "value": "10k",
                        "position": { "x": 0, "y": 0 },
                        "rotation": 0,
                        "layer": 1,
                        "locked": false
                    },
                    package_b_uuid.to_string(): {
                        "uuid": package_b_uuid,
                        "part": Uuid::new_v4(),
                        "package": Uuid::new_v4(),
                        "reference": "R2",
                        "value": "10k",
                        "position": { "x": 5000000, "y": 0 },
                        "rotation": 0,
                        "layer": 1,
                        "locked": false
                    }
                },
                "pads": {
                    pad_a_uuid.to_string(): {
                        "uuid": pad_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": net_uuid,
                        "position": { "x": 0, "y": 0 },
                        "layer": 1
                    },
                    pad_b_uuid.to_string(): {
                        "uuid": pad_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": net_uuid,
                        "position": { "x": 5000000, "y": 0 },
                        "layer": 1
                    }
                },
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");
    net_uuid
}

pub(super) fn seed_board_process_aperture_fixture(root: &Path) {
    let _ = seed_board_drc_fixture(root);
    let board_json = root.join("board/board.json");
    let mut board: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&board_json).expect("board file should read"),
    )
    .expect("board JSON should parse");
    board["stackup"] = serde_json::json!({
        "layers": [
            { "id": 1, "name": "F.Cu", "layer_type": "Copper", "thickness_nm": 35000 },
            { "id": 2, "name": "F.Mask", "layer_type": "SolderMask", "thickness_nm": 0 },
            { "id": 4, "name": "F.Paste", "layer_type": "Paste", "thickness_nm": 0 }
        ]
    });
    board["pad_expansion_setup"] = serde_json::json!({
        "pad_to_mask_clearance_nm": 127000,
        "pad_to_paste_clearance_nm": -127000,
        "pad_to_paste_ratio_ppm": 0,
        "solder_mask_min_width_nm": 0
    });
    for pad in board["pads"].as_object_mut().unwrap().values_mut() {
        pad["copper_layers"] = serde_json::json!([1]);
        pad["shape"] = serde_json::json!("rect");
        pad["width"] = serde_json::json!(1_000_000);
        pad["height"] = serde_json::json!(500_000);
        pad["mask_layers"] = serde_json::json!([2]);
        pad["paste_layers"] = serde_json::json!([4]);
        pad["solder_mask_margin_nm"] = serde_json::json!(0);
        pad["solder_paste_margin_nm"] = serde_json::json!(0);
        pad["solder_paste_margin_ratio_ppm"] = serde_json::json!(0);
    }
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&board).expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");
}

#[test]
fn project_query_check_reports_native_schematic_check_json() {
    let root = unique_project_root("datum-eda-cli-project-query-check-json");
    create_native_project(&root, Some("Check Demo".to_string()))
        .expect("initial scaffold should succeed");
    let (passive_pin_uuid, power_pin_uuid) = build_native_check_fixture(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "check",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query check should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    assert_eq!(report["domain"], "combined");
    assert_eq!(report["summary"]["status"], "warning");
    assert!(
        report["summary"]["by_code"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["code"] == "dangling_component_pin" && entry["count"] == 1)
    );
    assert!(
        report["summary"]["by_code"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["code"] == "unconnected_component_pin" && entry["count"] == 1)
    );
    assert!(
        report["summary"]["by_code"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["code"] == "power_in_without_source" && entry["count"] == 1)
    );
    assert!(
        report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| {
                entry["kind"] == "dangling_component_pin"
                    && entry["objects"] == serde_json::json!([passive_pin_uuid.to_string()])
            })
    );
    assert!(report["erc"].as_array().unwrap().iter().any(|entry| {
        entry["code"] == "unconnected_component_pin"
            && entry["object_uuids"] == serde_json::json!([passive_pin_uuid.to_string()])
    }));
    assert!(report["erc"].as_array().unwrap().iter().any(|entry| {
        entry["code"] == "power_in_without_source"
            && entry["object_uuids"] == serde_json::json!([power_pin_uuid.to_string()])
    }));
    assert_eq!(report["drc"], serde_json::json!([]));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_check_run_reports_stable_process_aperture_findings() {
    let root = unique_project_root("datum-eda-cli-project-query-check-run-process-aperture");
    create_native_project(&root, Some("Process Aperture Check Demo".to_string()))
        .expect("initial scaffold should succeed");
    let _ = build_native_check_fixture(&root);
    seed_board_process_aperture_fixture(&root);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "check-run",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query check-run should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("check-run JSON should parse");
    assert!(report["findings"].as_array().unwrap().iter().any(|entry| {
        entry["source"] == "drc" && entry["code"] == "pad_mask_expansion_missing"
    }));
    assert!(report["findings"].as_array().unwrap().iter().any(|entry| {
        entry["source"] == "drc" && entry["code"] == "pad_paste_reduction_missing"
    }));
    assert!(report["findings"].as_array().unwrap().iter().any(|entry| {
        entry["source"] == "drc" && entry["code"] == "pad_process_aperture_inherited_from_copper"
    }));
    let mask_finding = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["source"] == "drc" && entry["code"] == "pad_mask_expansion_missing")
        .expect("pad mask expansion finding should exist");
    assert!(
        mask_finding["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| {
                entry["evidence_kind"] == "standards_basis"
                    && entry["basis_id"] == "datum.process_aperture_and_geometry.current"
                    && entry["rule_family"] == "process_aperture_policy"
                    && entry["policy"]["pad_process_aperture_must_not_inherit_from_copper"] == true
            })
    );

    let repeat_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "check-run",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query check-run repeat should succeed");
    assert_eq!(output, repeat_output);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_check_run_links_findings_to_matching_proposals() {
    let root = unique_project_root("datum-eda-cli-project-query-check-run-proposal-ref");
    create_native_project(&root, Some("Check Run Proposal Ref Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "check-run",
        ])
        .expect("CLI should parse"),
    )
    .expect("initial check-run should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("check-run JSON should parse");
    let finding_id = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["source"] == "erc")
        .expect("erc finding should exist")["finding_id"]
        .as_str()
        .unwrap()
        .to_string();
    let finding_fingerprint = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["finding_id"] == finding_id)
        .expect("erc finding should exist")["fingerprint"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(finding_fingerprint.starts_with("sha256:"));
    let check_run_id = Uuid::parse_str(report["check_run_id"].as_str().unwrap()).unwrap();
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"check-run-proposal-ref");
    write_legacy_proposal_sidecar(
        &root,
        &Proposal {
            schema_version: 1,
            proposal_id,
            project_id: model.project.project_id,
            prepared_against: model.model_revision.clone(),
            batch: OperationBatch {
                batch_id: Uuid::new_v5(&model.project.project_id, b"check-run-proposal-ref-batch"),
                expected_model_revision: Some(model.model_revision),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "link proposal to check finding".to_string(),
                },
                operations: Vec::new(),
            },
            rationale: "test proposal ref linkage".to_string(),
            affected_objects: Vec::new(),
            checks_run: vec![check_run_id],
            finding_fingerprints: vec![finding_fingerprint],
            source: ProposalSource::Check,
            status: ProposalStatus::Draft,
            applied_transaction_id: None,
        },
    )
    .expect("proposal should write");

    let linked_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "check-run",
        ])
        .expect("CLI should parse"),
    )
    .expect("linked check-run should succeed");
    let linked: serde_json::Value =
        serde_json::from_str(&linked_output).expect("check-run JSON should parse");
    assert_eq!(
        linked["proposal_refs"],
        serde_json::json!([proposal_id.to_string()])
    );
    assert_eq!(
        linked["proposal_links"][0]["proposal_id"],
        serde_json::json!(proposal_id.to_string())
    );
    assert_eq!(linked["proposal_links"][0]["status"], "draft");
    assert_eq!(linked["proposal_links"][0]["source"], "check");
    assert_eq!(
        linked["proposal_links"][0]["rationale"],
        "test proposal ref linkage"
    );
    assert_eq!(linked["proposal_links"][0]["can_apply"], false);
    assert_eq!(
        linked["proposal_links"][0]["blocker_codes"],
        serde_json::json!(["missing_acceptance"])
    );
    assert!(
        linked["proposal_links"][0]["command_templates"]["accept_apply"]
            .as_str()
            .unwrap()
            .contains("datum-eda proposal accept-apply")
    );
    assert_eq!(
        linked["proposal_links"][0]["command_templates"]["preview"],
        format!(
            "datum-eda proposal preview {} --proposal {}",
            root.display(),
            proposal_id
        )
    );
    let linked_finding = linked["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["finding_id"] == finding_id)
        .expect("linked finding should exist");
    assert_eq!(
        linked_finding["proposal_refs"],
        serde_json::json!([proposal_id.to_string()])
    );
    assert_eq!(
        linked_finding["proposal_links"][0]["proposal_id"],
        serde_json::json!(proposal_id.to_string())
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_check_reports_native_schematic_check_text() {
    let root = unique_project_root("datum-eda-cli-project-query-check-text");
    create_native_project(&root, Some("Check Demo".to_string()))
        .expect("initial scaffold should succeed");
    let _ = build_native_check_fixture(&root);

    let cli = Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "check"])
        .expect("CLI should parse");

    let output = execute(cli).expect("project query check should succeed");
    assert!(output.contains("combined check: status=warning"));
    assert!(output.contains("counts:"));
    assert!(output.contains("dangling_component_pin x1"));
    assert!(output.contains("unconnected_component_pin x1"));
    assert!(output.contains("power_in_without_source x1"));
    assert!(output.contains("diagnostics:"));
    assert!(output.contains("erc:"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_check_counts_matching_native_waiver_without_failing_summary() {
    let root = unique_project_root("datum-eda-cli-project-query-check-waiver");
    create_native_project(&root, Some("Check Demo".to_string()))
        .expect("initial scaffold should succeed");
    let (passive_pin_uuid, _) = build_native_check_fixture(&root);

    write_native_waivers(
        &root,
        &[serde_json::to_value(serde_json::json!({
            "uuid": Uuid::new_v4(),
            "domain": CheckDomain::ERC,
            "target": WaiverTarget::Object(passive_pin_uuid),
            "rationale": "Intentional dangling passive pin",
            "created_by": "cli-test"
        }))
        .expect("waiver should serialize")],
    );

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "check",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query check should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    assert_eq!(report["summary"]["status"], "warning");
    assert_eq!(report["summary"]["errors"], 0);
    assert_eq!(report["summary"]["warnings"], 2);
    assert_eq!(report["summary"]["waived"], 1);
    let waived = report["erc"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["code"] == "unconnected_component_pin")
        .expect("waived finding should exist");
    assert_eq!(
        waived["object_uuids"],
        serde_json::json!([passive_pin_uuid.to_string()])
    );
    assert_eq!(waived["waived"], true);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_check_includes_waived_native_drc_results() {
    let root = unique_project_root("datum-eda-cli-project-query-check-drc");
    create_native_project(&root, Some("Check Demo".to_string()))
        .expect("initial scaffold should succeed");
    let _ = build_native_check_fixture(&root);
    let net_uuid = seed_board_drc_fixture(&root);

    write_native_waivers(
        &root,
        &[serde_json::to_value(serde_json::json!({
            "uuid": Uuid::new_v4(),
            "domain": CheckDomain::DRC,
            "target": WaiverTarget::Object(net_uuid),
            "rationale": "Intentional unrouted fixture net",
            "created_by": "cli-test"
        }))
        .expect("waiver should serialize")],
    );

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "check",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query check should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    assert_eq!(report["domain"], "combined");
    assert_eq!(report["summary"]["status"], "warning");
    assert_eq!(report["summary"]["errors"], 0);
    assert_eq!(report["summary"]["warnings"], 3);
    assert_eq!(report["summary"]["waived"], 2);
    assert!(
        report["drc"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["waived"] == true)
    );

    let _ = std::fs::remove_dir_all(&root);
}
