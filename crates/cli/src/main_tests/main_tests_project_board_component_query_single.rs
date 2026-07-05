use super::*;
use crate::main_tests::main_tests_project_forward_annotation_support::write_board_packages;
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;

#[test]
fn project_query_board_component_reports_single_component_view() {
    let root = std::env::temp_dir().join(format!(
        "datum-eda-cli-project-query-board-component-{}",
        Uuid::new_v4()
    ));
    create_native_project(&root, Some("Single Component Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let component_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();
    write_board_packages(
        &root,
        "Single Component Query Demo Board",
        vec![PlacedPackage {
            uuid: component_uuid,
            part: part_uuid,
            package: package_uuid,
            reference: "U1".into(),
            value: "MCU".into(),
            position: Point::new(1234, 5678),
            rotation: 90,
            layer: 1,
            locked: false,
        }],
    );

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "board-component",
            "--component",
            &component_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("single component query should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("single component query JSON should parse");
    assert_eq!(report["uuid"], component_uuid.to_string());
    assert_eq!(report["part"], part_uuid.to_string());
    assert_eq!(report["package"], package_uuid.to_string());
    assert_eq!(report["reference"], "U1");
    assert_eq!(report["value"], "MCU");
    assert_eq!(report["position"]["x"], 1234);
    assert_eq!(report["position"]["y"], 5678);
    assert_eq!(report["rotation"], 90);
    assert_eq!(report["layer"], 1);
    assert_eq!(report["locked"], false);

    let _ = std::fs::remove_dir_all(&root);
}
