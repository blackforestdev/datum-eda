//! Real public CLI routes share native placement/transform geometry and history.
use super::*;
use eda_engine::{
    ir::geometry::Point,
    schematic::Schematic,
    substrate::{DesignModel, ElectricalSelectionSnapshot, ProjectResolver},
};
use std::collections::BTreeMap;

fn command(root: &Path, verb: &str, args: &[&str]) -> Result<serde_json::Value> {
    let mut argv = vec![
        "datum-eda",
        "--format",
        "json",
        "project",
        verb,
        root.to_str().unwrap(),
    ];
    argv.extend_from_slice(args);
    Ok(serde_json::from_str(&execute(Cli::try_parse_from(argv)?)?)?)
}
fn resolved(root: &Path) -> (DesignModel, Schematic) {
    let model = ProjectResolver::new(root).resolve().unwrap();
    let source = ElectricalSelectionSnapshot::capture(&model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .clone();
    (model, source)
}
fn project(label: &str) -> (PathBuf, Uuid, Uuid, [Uuid; 2]) {
    let root = development_workspace::path("cli-symbol-geometry")
        .join(format!("{label}-{}", Uuid::new_v4()));
    create_native_project(&root, Some(label.into())).unwrap();
    let sheet = Uuid::new_v4();
    let unit = Uuid::new_v4();
    let library = Uuid::new_v4();
    let pins = [Uuid::new_v4(), Uuid::new_v4()];
    command(
        &root,
        "create-sheet",
        &["--sheet", &sheet.to_string(), "--name", "Main"],
    )
    .unwrap();
    command(
        &root,
        "create-pool-unit",
        &["--unit", &unit.to_string(), "--name", "Asymmetric"],
    )
    .unwrap();
    command(
        &root,
        "create-pool-symbol",
        &[
            "--symbol",
            &library.to_string(),
            "--unit",
            &unit.to_string(),
            "--name",
            "Asymmetric",
        ],
    )
    .unwrap();
    for (pin, name, x, y) in [(pins[0], "1", "100", "20"), (pins[1], "2", "-40", "80")] {
        command(
            &root,
            "set-pool-unit-pin",
            &[
                "--unit",
                &unit.to_string(),
                "--pin",
                &pin.to_string(),
                "--name",
                name,
                "--electrical-type",
                "Passive",
            ],
        )
        .unwrap();
        command(
            &root,
            "set-pool-symbol-pin-anchor",
            &[
                "--symbol",
                &library.to_string(),
                "--pin",
                &pin.to_string(),
                &format!("--x-nm={x}"),
                &format!("--y-nm={y}"),
            ],
        )
        .unwrap();
    }
    (root, sheet, library, pins)
}
fn place(root: &Path, sheet: Uuid, library: Uuid) -> serde_json::Value {
    command(
        root,
        "place-symbol",
        &[
            "--sheet",
            &sheet.to_string(),
            "--lib-id",
            &library.to_string(),
            "--reference",
            "U1",
            "--value",
            "test",
            "--x-nm",
            "1000",
            "--y-nm",
            "1000",
            "--rotation-deg",
            "90",
            "--mirrored",
        ],
    )
    .unwrap()
}
fn pin_points(source: &Schematic, sheet: Uuid, symbol: Uuid) -> BTreeMap<Uuid, Point> {
    source.sheets[&sheet].symbols[&symbol]
        .pins
        .iter()
        .map(|pin| (pin.library_pin.unwrap(), pin.position))
        .collect()
}
fn pin_groups(source: &Schematic) -> Vec<Vec<Uuid>> {
    let mut result: Vec<_> = eda_engine::connectivity::schematic_net_info(source)
        .into_iter()
        .map(|net| {
            let mut pins: Vec<_> = net.pins.into_iter().map(|pin| pin.uuid).collect();
            pins.sort();
            pins
        })
        .collect();
    result.sort();
    result
}
fn persisted(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(path: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for item in std::fs::read_dir(path).unwrap() {
            let path = item.unwrap().path();
            if path.is_dir() {
                walk(&path, result);
            } else {
                result.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    walk(root, &mut result);
    result
}

#[test]
fn public_native_library_place_and_connected_transforms_preserve_geometry_and_history() {
    let (root, sheet, library, pins) = project("public-transform");
    let placed = place(&root, sheet, library);
    let id: Uuid = placed["symbol_uuid"].as_str().unwrap().parse().unwrap();
    assert_eq!(placed["action"], "place_symbol");
    assert_eq!(placed["rotation_deg"], 90);
    assert_eq!(placed["mirrored"], true);
    let (_, source) = resolved(&root);
    assert_eq!(
        pin_points(&source, sheet, id),
        BTreeMap::from([
            (pins[0], Point::new(980, 900)),
            (pins[1], Point::new(920, 1040))
        ])
    );
    let mut wire_ids = Vec::new();
    for (from, to) in [
        (Point::new(980, 900), Point::new(2000, 900)),
        (Point::new(920, 1040), Point::new(0, 1040)),
    ] {
        let wire = command(
            &root,
            "draw-wire",
            &[
                "--sheet",
                &sheet.to_string(),
                "--from-x-nm",
                &from.x.to_string(),
                "--from-y-nm",
                &from.y.to_string(),
                "--to-x-nm",
                &to.x.to_string(),
                "--to-y-nm",
                &to.y.to_string(),
            ],
        )
        .unwrap();
        wire_ids.push(wire["wire_uuid"].as_str().unwrap().parse::<Uuid>().unwrap());
    }
    let (initial, source) = resolved(&root);
    let identities = initial.electrical_identities.clone();
    let groups = pin_groups(&source);
    for (verb, args, expected) in [
        (
            "move-symbol",
            vec!["--x-nm", "1200", "--y-nm", "1100"],
            [Point::new(1180, 1000), Point::new(1120, 1140)],
        ),
        (
            "rotate-symbol",
            vec!["--rotation-deg", "0"],
            [Point::new(1100, 1120), Point::new(1240, 1180)],
        ),
        (
            "mirror-symbol",
            vec![],
            [Point::new(1300, 1120), Point::new(1160, 1180)],
        ),
    ] {
        let (_, before) = resolved(&root);
        let symbol_id = id.to_string();
        let mut flags = vec!["--symbol", &symbol_id];
        flags.extend(args);
        let report = command(&root, verb, &flags).unwrap();
        assert_eq!(report["action"], verb.replace('-', "_"));
        let (model, after) = resolved(&root);
        assert_eq!(model.electrical_identities, identities);
        assert_eq!(pin_groups(&after), groups);
        assert_eq!(
            pin_points(&after, sheet, id),
            BTreeMap::from([(pins[0], expected[0]), (pins[1], expected[1])])
        );
        for (wire, from) in wire_ids.iter().zip(expected) {
            assert_eq!(after.sheets[&sheet].wires[wire].from, from);
        }
        assert_eq!(
            after.sheets[&sheet].wires[&wire_ids[0]].to,
            Point::new(2000, 900)
        );
        assert_eq!(
            after.sheets[&sheet].wires[&wire_ids[1]].to,
            Point::new(0, 1040)
        );
        command(&root, "undo", &[]).unwrap();
        assert_eq!(resolved(&root).1, before);
        command(&root, "redo", &[]).unwrap();
        let (redone, reopened) = resolved(&root);
        assert_eq!(reopened, after);
        assert_eq!(redone.electrical_identities, identities);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn public_transform_refuses_invalid_pose_and_repeated_occurrence_without_writes() {
    let (root, sheet, library, _) = project("public-transform-refusal");
    let placed = place(&root, sheet, library);
    let id = placed["symbol_uuid"].as_str().unwrap();
    let before = persisted(&root);
    assert!(
        command(
            &root,
            "rotate-symbol",
            &["--symbol", id, "--rotation-deg", "45"]
        )
        .is_err()
    );
    assert_eq!(persisted(&root), before);
    let definition = Uuid::new_v4();
    command(
        &root,
        "create-sheet-definition",
        &[
            "--definition",
            &definition.to_string(),
            "--root-sheet",
            &sheet.to_string(),
            "--name",
            "Repeated",
        ],
    )
    .unwrap();
    // A unique instantiated occurrence is resolved rather than assuming root [].
    command(
        &root,
        "create-sheet-instance",
        &[
            "--definition",
            &definition.to_string(),
            "--name",
            "First",
            "--x-nm",
            "0",
            "--y-nm",
            "0",
        ],
    )
    .unwrap();
    command(
        &root,
        "move-symbol",
        &["--symbol", id, "--x-nm", "1100", "--y-nm", "1100"],
    )
    .unwrap();
    command(
        &root,
        "create-sheet-instance",
        &[
            "--definition",
            &definition.to_string(),
            "--name",
            "Second",
            "--x-nm",
            "0",
            "--y-nm",
            "0",
        ],
    )
    .unwrap();
    let before = persisted(&root);
    for (verb, args) in [
        ("move-symbol", vec!["--x-nm", "1200", "--y-nm", "1200"]),
        ("rotate-symbol", vec!["--rotation-deg", "0"]),
        ("mirror-symbol", vec![]),
    ] {
        let mut flags = vec!["--symbol", id];
        flags.extend(args);
        assert!(
            command(&root, verb, &flags)
                .unwrap_err()
                .to_string()
                .contains("exactly one")
        );
    }
    assert_eq!(persisted(&root), before);
    std::fs::remove_dir_all(root).unwrap();
}
