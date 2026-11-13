use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_compare_pnp_reports_matched_missing_extra_and_drift() {
    let root = unique_project_root("datum-eda-cli-project-pnp-compare");
    create_native_project(&root, Some("PnP Compare Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let u2_uuid = Uuid::new_v4();
    let u1_uuid = Uuid::new_v4();
    let u1_part_uuid = Uuid::new_v4();
    let u2_part_uuid = Uuid::new_v4();
    let u1_package_uuid = Uuid::new_v4();
    let u2_package_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "PnP Compare Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    u2_uuid.to_string(): {
                        "uuid": u2_uuid,
                        "part": u2_part_uuid,
                        "package": u2_package_uuid,
                        "reference": "U2",
                        "value": "MCU",
                        "position": { "x": 2000, "y": 3000 },
                        "rotation": 180,
                        "layer": 31,
                        "locked": true
                    },
                    u1_uuid.to_string(): {
                        "uuid": u1_uuid,
                        "part": u1_part_uuid,
                        "package": u1_package_uuid,
                        "reference": "U1",
                        "value": "ADC",
                        "position": { "x": 1000, "y": 1500 },
                        "rotation": 90,
                        "layer": 1,
                        "locked": false
                    }
                },
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

    // All three historical headers retain their shared legacy matching mode.
    let legacy_path = root.join("legacy-pnp.csv");
    export_native_project_pnp(&root, &legacy_path, None).unwrap();
    let current_csv = std::fs::read_to_string(&legacy_path).unwrap();
    let legacy_line = |line: &str, leading_columns: usize| {
        let columns = parse_csv_line(line).unwrap();
        let end = columns.len() - 1;
        match leading_columns {
            3 => columns[..end].join(","),
            1 => [&columns[..1], &columns[3..end]].concat().join(","),
            0 => columns[3..end].join(","),
            _ => unreachable!(),
        }
    };
    for leading_columns in [3, 1, 0] {
        let legacy = current_csv
            .lines()
            .map(|line| legacy_line(line, leading_columns))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        std::fs::write(&legacy_path, legacy).unwrap();
        let report = compare_native_project_pnp(&root, &legacy_path, None).unwrap();
        assert_eq!(report.matched_count, 2);
        assert_eq!(
            (report.missing_count, report.extra_count, report.drift_count),
            (0, 0, 0)
        );
        // A header-only legacy file must retain legacy identities as well.
        let header_only = current_csv.lines().next().unwrap();
        std::fs::write(
            &legacy_path,
            legacy_line(header_only, leading_columns) + "\n",
        )
        .unwrap();
        let empty = compare_native_project_pnp(&root, &legacy_path, None).unwrap();
        let mut expected_missing = vec![u1_package_uuid.to_string(), u2_package_uuid.to_string()];
        expected_missing.sort();
        assert_eq!(empty.missing, expected_missing);
        assert_eq!(
            (
                empty.matched_count,
                empty.actual_count,
                empty.extra_count,
                empty.drift_count
            ),
            (0, 0, 0, 0)
        );
    }

    std::fs::write(
        &legacy_path,
        current_csv.lines().next().unwrap().to_string() + "\n",
    )
    .unwrap();
    let empty_current = compare_native_project_pnp(&root, &legacy_path, None).unwrap();
    let mut expected_missing = vec![u1_uuid.to_string(), u2_uuid.to_string()];
    expected_missing.sort();
    assert_eq!(empty_current.missing, expected_missing);

    let pnp_path = root.join("pnp.csv");
    std::fs::write(
        &pnp_path,
        format!(
            "reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked\nU1,1000,1500,90,1,top,{u1_package_uuid},{u1_part_uuid},ADC,false\nU2,2500,3000,180,31,bottom,{u2_package_uuid},{u2_part_uuid},MCU,true\nU3,4000,5000,0,1,top,{u1_package_uuid},{u1_part_uuid},SPARE,false\n"
        ),
    )
    .expect("pnp file should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-pnp",
        root.to_str().unwrap(),
        "--pnp",
        pnp_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("PnP compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "compare_pnp");
    assert_eq!(report["expected_count"], 2);
    assert_eq!(report["actual_count"], 3);
    assert_eq!(report["matched_count"], 1);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 1);
    assert_eq!(report["drift_count"], 1);
    assert_eq!(report["matched"][0], u1_package_uuid.to_string());
    assert_eq!(report["extra"][0], u1_package_uuid.to_string());
    assert_eq!(report["drift"][0]["identity"], u2_package_uuid.to_string());
    assert_eq!(report["drift"][0]["reference"], "U2");
    assert_eq!(report["drift"][0]["fields"][0], "position");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_compare_pnp_distinguishes_repeated_package_placements_after_journaled_move() {
    let root = unique_project_root("datum-eda-pnp-placement-identity");
    create_native_project(&root, Some("Repeated placements".to_string())).unwrap();
    let part = Uuid::new_v4().to_string();
    let package = Uuid::new_v4().to_string();
    let mut placements = Vec::new();
    for reference in ["R1", "R2"] {
        let output = execute(
            Cli::try_parse_from([
                "datum-eda",
                "--format",
                "json",
                "project",
                "place-board-component",
                root.to_str().unwrap(),
                "--part",
                &part,
                "--package",
                &package,
                "--reference",
                reference,
                "--value",
                "10k",
                "--x-nm",
                "1000",
                "--y-nm",
                "2000",
                "--layer",
                "1",
            ])
            .unwrap(),
        )
        .unwrap();
        let report: serde_json::Value = serde_json::from_str(&output).unwrap();
        placements.push(report["component_uuid"].as_str().unwrap().to_string());
    }
    let csv_path = root.join("placements.csv");
    let output = execute(
        Cli::try_parse_from([
            "datum-eda",
            "--format",
            "json",
            "project",
            "export-pnp",
            root.to_str().unwrap(),
            "--out",
            csv_path.to_str().unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    let export: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(export["rows"], 2);
    let csv = std::fs::read_to_string(&csv_path).unwrap();
    assert_eq!(csv.lines().count(), 3);
    assert!(csv.lines().next().unwrap().ends_with(",placement_uuid"));
    let compare = || {
        let output = execute(
            Cli::try_parse_from([
                "datum-eda",
                "--format",
                "json",
                "project",
                "compare-pnp",
                root.to_str().unwrap(),
                "--pnp",
                csv_path.to_str().unwrap(),
            ])
            .unwrap(),
        )
        .unwrap();
        serde_json::from_str::<serde_json::Value>(&output).unwrap()
    };
    let unchanged = compare();
    assert_eq!(unchanged["matched_count"], 2);
    assert_eq!(unchanged["missing_count"], 0);
    assert_eq!(unchanged["extra_count"], 0);
    assert_eq!(unchanged["drift_count"], 0);
    for placement in &placements {
        assert!(
            unchanged["matched"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!(placement))
        );
    }
    execute(
        Cli::try_parse_from([
            "datum-eda",
            "project",
            "move-board-component",
            root.to_str().unwrap(),
            "--component",
            &placements[1],
            "--x-nm",
            "3000",
            "--y-nm",
            "4000",
        ])
        .unwrap(),
    )
    .unwrap();
    let changed = compare();
    assert_eq!(changed["expected_count"], 2);
    assert_eq!(changed["actual_count"], 2);
    assert_eq!(changed["matched"], serde_json::json!([placements[0]]));
    assert_eq!(changed["missing_count"], 0);
    assert_eq!(changed["extra_count"], 0);
    assert_eq!(changed["drift_count"], 1);
    assert_eq!(changed["drift"][0]["identity"], placements[1]);
    assert_eq!(changed["drift"][0]["reference"], "R2");
    assert_eq!(
        changed["drift"][0]["fields"],
        serde_json::json!(["position"])
    );
    // New identity columns must not silently fall back to ambiguous legacy keys.
    std::fs::write(&csv_path, csv.replace(&placements[1], "invalid-placement")).unwrap();
    assert!(compare_native_project_pnp(&root, &csv_path, None).is_err());
    let _ = std::fs::remove_dir_all(&root);
}
