//! Ordinary fabrication graphics use the existing primitive and journal contract.
use super::*;
use eda_engine::substrate::{Operation, ProjectResolver};

fn command(root: &Path, verb: &str, args: &[&str]) -> Result<String> {
    let mut argv = vec![
        "datum-eda",
        "--format",
        "json",
        "project",
        verb,
        root.to_str().unwrap(),
    ];
    argv.extend_from_slice(args);
    execute(Cli::try_parse_from(argv)?)
}
fn payload(root: &Path, id: Uuid) -> serde_json::Value {
    super::main_tests_project_pool_library::query_pool_object_payload(root, "footprints", id)
}
fn journal(root: &Path) -> Vec<u8> {
    std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap_or_default()
}
struct Fixture {
    root: PathBuf,
    footprint: Uuid,
    package: Uuid,
    stack: Uuid,
}
impl Fixture {
    fn new(label: &str) -> Self {
        let root = crate::main_tests::development_workspace::path("footprint-reference-audit")
            .join(format!("{label}-{}", Uuid::new_v4()));
        create_native_project(&root, Some(label.to_string())).unwrap();
        let package = Uuid::new_v4();
        let footprint = Uuid::new_v4();
        let stack = Uuid::new_v4();
        let pad = Uuid::new_v4();
        command(
            &root,
            "create-pool-package",
            &["--package", &package.to_string(), "--name", "Manual body"],
        )
        .unwrap();
        command(
            &root,
            "create-pool-footprint",
            &[
                "--footprint",
                &footprint.to_string(),
                "--package",
                &package.to_string(),
                "--name",
                "Manual graphics",
            ],
        )
        .unwrap();
        command(
            &root,
            "create-pool-padstack",
            &[
                "--padstack",
                &stack.to_string(),
                "--name",
                "Manual pad",
                "--aperture",
                "circle",
                "--diameter-nm",
                "1000",
            ],
        )
        .unwrap();
        command(
            &root,
            "set-pool-footprint-pad",
            &[
                "--footprint",
                &footprint.to_string(),
                "--pad",
                &pad.to_string(),
                "--padstack",
                &stack.to_string(),
            ],
        )
        .unwrap();
        let mut object = payload(&root, footprint);
        let primitive = serde_json::json!({"Line": {"from": {"x": 1, "y": 2}, "to": {"x": 3, "y": 4}, "width": 1}});
        for field in ["silkscreen", "fab", "assembly", "mechanical"] {
            object[field] = serde_json::json!([primitive.clone()]);
        }
        object["extension"] = serde_json::json!({"uuid": Uuid::new_v4(), "retained": [1,2]});
        let file = root.join("footprint-input.json");
        std::fs::write(&file, serde_json::to_vec(&object).unwrap()).unwrap();
        command(
            &root,
            "set-pool-library-object",
            &[
                "--kind",
                "footprints",
                "--object",
                &footprint.to_string(),
                "--from-json",
                file.to_str().unwrap(),
            ],
        )
        .unwrap();
        Self {
            root,
            footprint,
            package,
            stack,
        }
    }
}
fn shapes() -> Vec<(&'static str, Vec<&'static str>, serde_json::Value)> {
    vec![
        (
            "line",
            vec![
                "--from-x-nm",
                "0",
                "--from-y-nm",
                "0",
                "--to-x-nm",
                "10",
                "--to-y-nm",
                "20",
                "--width-nm",
                "2",
            ],
            serde_json::json!({"Line": {"from": {"x": 0, "y": 0}, "to": {"x": 10, "y": 20}, "width": 2}}),
        ),
        (
            "rect",
            vec![
                "--min-x-nm",
                "0",
                "--min-y-nm",
                "0",
                "--max-x-nm",
                "10",
                "--max-y-nm",
                "20",
                "--width-nm",
                "2",
            ],
            serde_json::json!({"Rect": {"min": {"x": 0, "y": 0}, "max": {"x": 10, "y": 20}, "width": 2}}),
        ),
        (
            "circle",
            vec![
                "--center-x-nm",
                "10",
                "--center-y-nm",
                "20",
                "--radius-nm",
                "30",
                "--width-nm",
                "2",
            ],
            serde_json::json!({"Circle": {"center": {"x": 10, "y": 20}, "radius": 30, "width": 2}}),
        ),
        (
            "polygon",
            vec!["--vertices", "0,0;10,0;0,20", "--width-nm", "2"],
            serde_json::json!({"Polygon": {"polygon": {"vertices": [{"x": 0, "y": 0},{"x": 10, "y": 0},{"x": 0, "y": 20}], "closed": true}, "width": 2}}),
        ),
    ]
}

#[test]
fn footprint_graphics_fab_all_families_default_silk_and_history_preserve_siblings() {
    let f = Fixture::new("fab-history");
    let root = &f.root;
    for (shape, args, primitive) in shapes() {
        for target in ["fab", "silkscreen"] {
            let before = ProjectResolver::new(root).resolve().unwrap();
            let original = payload(root, f.footprint);
            let id = f.footprint.to_string();
            let mut argv = vec!["--footprint", &id];
            argv.extend_from_slice(&args);
            if target == "fab" {
                argv.extend_from_slice(&["--target", "fab"]);
            }
            let verb = if target == "fab" {
                format!("add-pool-footprint-{shape}")
            } else {
                format!("add-pool-footprint-silkscreen-{shape}")
            };
            let result: serde_json::Value =
                serde_json::from_str(&command(root, &verb, &argv).unwrap()).unwrap();
            assert_eq!(result["action"], format!("add_footprint_{target}_{shape}"));
            let mut expected = original.clone();
            expected[target]
                .as_array_mut()
                .unwrap()
                .push(primitive.clone());
            assert_eq!(payload(root, f.footprint), expected);
            let after = ProjectResolver::new(root).resolve().unwrap();
            assert_eq!(after.journal.len(), before.journal.len() + 1);
            assert_eq!(
                after.objects[&f.footprint].object_revision.0,
                before.objects[&f.footprint].object_revision.0 + 1
            );
            assert!(matches!(
                after.journal.last().unwrap().operations.as_slice(),
                [Operation::SetPoolLibraryObject { .. }]
            ));
            for id in [f.package, f.stack] {
                assert_eq!(
                    after.objects[&id].object_revision,
                    before.objects[&id].object_revision
                );
            }
            command(root, "undo", &[]).unwrap();
            assert_eq!(payload(root, f.footprint), original);
            command(root, "redo", &[]).unwrap();
            assert_eq!(payload(root, f.footprint), expected);
        }
    }
    // A generic alias with no target keeps silk; explicit open fabrication polygons stay open.
    let id = f.footprint.to_string();
    command(
        root,
        "add-pool-footprint-line",
        &[
            "--footprint",
            &id,
            "--from-x-nm",
            "0",
            "--from-y-nm",
            "0",
            "--to-x-nm",
            "1",
            "--to-y-nm",
            "1",
            "--width-nm",
            "1",
        ],
    )
    .unwrap();
    command(
        root,
        "add-pool-footprint-polygon",
        &[
            "--footprint",
            &id,
            "--target",
            "fab",
            "--vertices",
            "0,0;1,1",
            "--closed",
            "false",
            "--width-nm",
            "1",
        ],
    )
    .unwrap();
    let object = payload(root, f.footprint);
    assert_eq!(
        object["fab"].as_array().unwrap().last().unwrap()["Polygon"]["polygon"]["closed"],
        false
    );
    let valid: serde_json::Value =
        serde_json::from_str(&command(root, "validate", &[]).unwrap()).unwrap();
    assert_eq!(valid["valid"], true);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn footprint_graphics_invalid_geometry_target_and_missing_definition_do_not_mutate() {
    let f = Fixture::new("fab-refusals");
    let root = &f.root;
    let id = f.footprint.to_string();
    let before = ProjectResolver::new(root).resolve().unwrap();
    let original = payload(root, f.footprint);
    let bytes = std::fs::read(root.join(format!("pool/footprints/{id}.json"))).unwrap();
    let log = journal(root);
    let manifest = std::fs::read(root.join("project.json")).unwrap();
    for (shape, valid, _) in shapes() {
        for target in ["fab", "silkscreen"] {
            let mut invalid = valid.clone();
            *invalid.last_mut().unwrap() = "0";
            let mut argv = vec!["--footprint", &id, "--target", target];
            argv.extend_from_slice(&invalid);
            let error = command(root, &format!("add-pool-footprint-{shape}"), &argv).unwrap_err();
            assert_eq!(
                format!("{error:#}"),
                format!("footprint {target} {shape} width-nm must be positive")
            );
        }
        let mut argv = vec!["--footprint", &id, "--target", "invalid"];
        argv.extend_from_slice(&valid);
        assert!(
            format!(
                "{:#}",
                command(root, &format!("add-pool-footprint-{shape}"), &argv).unwrap_err()
            )
            .contains("invalid value 'invalid'")
        );
    }
    for (verb, args, expected) in [
        (
            "add-pool-footprint-line",
            vec![
                "--from-x-nm",
                "0",
                "--from-y-nm",
                "0",
                "--to-x-nm",
                "0",
                "--to-y-nm",
                "0",
                "--width-nm",
                "0",
            ],
            "footprint fab line must have distinct endpoints",
        ),
        (
            "add-pool-footprint-rect",
            vec![
                "--min-x-nm",
                "1",
                "--min-y-nm",
                "1",
                "--max-x-nm",
                "0",
                "--max-y-nm",
                "0",
                "--width-nm",
                "0",
            ],
            "footprint fab rect min-x-nm must be less than max-x-nm",
        ),
        (
            "add-pool-footprint-circle",
            vec![
                "--center-x-nm",
                "0",
                "--center-y-nm",
                "0",
                "--radius-nm",
                "0",
                "--width-nm",
                "0",
            ],
            "footprint fab circle radius-nm must be positive",
        ),
        (
            "add-pool-footprint-polygon",
            vec!["--vertices", "broken", "--width-nm", "0"],
            "footprint fab polygon vertex 'broken' must be formatted as x,y",
        ),
        (
            "add-pool-footprint-polygon",
            vec!["--vertices", "0,0;1,1", "--width-nm", "0"],
            "footprint fab closed polygon must have at least 3 vertices",
        ),
        (
            "add-pool-footprint-polygon",
            vec!["--vertices", "0,0", "--closed", "false", "--width-nm", "0"],
            "footprint fab polyline must have at least 2 vertices",
        ),
    ] {
        let mut argv = vec!["--footprint", &id, "--target", "fab"];
        argv.extend_from_slice(&args);
        assert_eq!(
            format!("{:#}", command(root, verb, &argv).unwrap_err()),
            expected
        );
    }
    let missing = Uuid::new_v4();
    let missing_id = missing.to_string();
    assert_eq!(
        format!(
            "{:#}",
            command(
                root,
                "add-pool-footprint-circle",
                &[
                    "--footprint",
                    &missing_id,
                    "--target",
                    "fab",
                    "--center-x-nm",
                    "0",
                    "--center-y-nm",
                    "0",
                    "--radius-nm",
                    "1",
                    "--width-nm",
                    "1"
                ]
            )
            .unwrap_err()
        ),
        format!("missing pool footprint {missing}")
    );
    assert_eq!(journal(root), log);
    assert_eq!(payload(root, f.footprint), original);
    assert_eq!(
        std::fs::read(root.join(format!("pool/footprints/{id}.json"))).unwrap(),
        bytes
    );
    assert_eq!(std::fs::read(root.join("project.json")).unwrap(), manifest);
    assert_eq!(
        ProjectResolver::new(root).resolve().unwrap().model_revision,
        before.model_revision
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn footprint_graphics_malformed_collection_is_rejected_without_replacing_data() {
    use eda_engine::api::native_write::library_footprint::{
        FootprintGraphicTarget, footprint_with_graphic,
    };
    let f = Fixture::new("fab-malformed");
    let original = payload(&f.root, f.footprint);
    let log = journal(&f.root);
    for (target, field) in [
        (FootprintGraphicTarget::Fab, "fab"),
        (FootprintGraphicTarget::Silkscreen, "silkscreen"),
    ] {
        let mut input = original.clone();
        input[field] = serde_json::Value::Null;
        let primitive =
            serde_json::json!({"Circle": {"center": {"x": 0, "y": 0}, "radius": 1, "width": 1}});
        let error = footprint_with_graphic(input, target, primitive).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("validation error: pool footprint {field} field must be an array")
        );
    }
    assert_eq!(payload(&f.root, f.footprint), original);
    assert_eq!(journal(&f.root), log);
    std::fs::remove_dir_all(f.root).unwrap();
}
