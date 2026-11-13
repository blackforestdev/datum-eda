//! Public native authoring, saved checks and accepted occurrence-specific waivers.
use super::*;
use eda_engine::substrate::{CheckFinding, CheckRun, ElectricalSelectionSnapshot};

fn command(root: &Path, verb: &str, args: &[&str]) -> serde_json::Value {
    let mut argv = vec![
        "datum-eda",
        "--format",
        "json",
        "project",
        verb,
        root.to_str().unwrap(),
    ];
    argv.extend_from_slice(args);
    serde_json::from_str(&execute(Cli::try_parse_from(argv).unwrap()).unwrap()).unwrap()
}

fn saved_conflicts(root: &Path) -> Vec<CheckFinding> {
    // Each invocation starts by cold resolving, then executes the persisted public command.
    drop(ProjectResolver::new(root).resolve().unwrap());
    let report: serde_json::Value = serde_json::from_str(
        &execute(
            Cli::try_parse_from([
                "datum-eda",
                "--format",
                "json",
                "check",
                "run",
                root.to_str().unwrap(),
                "--profile",
                "erc",
            ])
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(report["persisted"], true);
    let run_id = Uuid::parse_str(report["check_run_id"].as_str().unwrap()).unwrap();
    let saved: CheckRun = serde_json::from_str(
        &std::fs::read_to_string(root.join(format!(".datum/check_runs/{run_id}.json"))).unwrap(),
    )
    .unwrap();
    let cold = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(cold.check_runs.get(&run_id), Some(&saved));
    let conflicts: Vec<_> = saved
        .findings
        .into_iter()
        .filter(|f| f.source == "erc" && f.code == "output_to_output_conflict")
        .collect();
    assert_eq!(conflicts.len(), 2);
    for finding in &conflicts {
        let emitted = report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["fingerprint"] == finding.fingerprint)
            .unwrap();
        assert_eq!(emitted["primary_target"], finding.primary_target);
        assert_eq!(
            emitted["related_targets"],
            serde_json::to_value(&finding.related_targets).unwrap()
        );
        assert_eq!(emitted["status"], finding.status);
        assert_eq!(
            emitted["waiver_refs"],
            serde_json::to_value(&finding.waiver_refs).unwrap()
        );
    }
    conflicts
}

#[test]
fn public_repeated_definition_check_waives_one_occurrence_after_accept_and_cold_reopen() {
    let root = development_workspace::path("cli-erc-occurrences").join(Uuid::new_v4().to_string());
    create_native_project(&root, Some("Repeated definition ERC".into())).unwrap();
    let sheet = Uuid::new_v4();
    let unit = Uuid::new_v4();
    let symbol = Uuid::new_v4();
    let library_pin = Uuid::new_v4();
    command(
        &root,
        "create-sheet",
        &["--sheet", &sheet.to_string(), "--name", "Child"],
    );
    command(
        &root,
        "create-pool-unit",
        &["--unit", &unit.to_string(), "--name", "Driver"],
    );
    command(
        &root,
        "create-pool-symbol",
        &[
            "--symbol",
            &symbol.to_string(),
            "--unit",
            &unit.to_string(),
            "--name",
            "Driver",
        ],
    );
    command(
        &root,
        "set-pool-unit-pin",
        &[
            "--unit",
            &unit.to_string(),
            "--pin",
            &library_pin.to_string(),
            "--name",
            "OUT",
            "--electrical-type",
            "Output",
        ],
    );
    command(
        &root,
        "set-pool-symbol-pin-anchor",
        &[
            "--symbol",
            &symbol.to_string(),
            "--pin",
            &library_pin.to_string(),
            "--x-nm",
            "0",
            "--y-nm",
            "0",
        ],
    );
    // All child contents are authored through ordinary commands before repetition.
    // Both placed symbols have one output at the local SIG node.
    for reference in ["U1", "U2"] {
        command(
            &root,
            "place-symbol",
            &[
                "--sheet",
                &sheet.to_string(),
                "--lib-id",
                &symbol.to_string(),
                "--reference",
                reference,
                "--value",
                "Driver",
                "--x-nm",
                "0",
                "--y-nm",
                "0",
            ],
        );
    }
    command(
        &root,
        "place-label",
        &[
            "--sheet",
            &sheet.to_string(),
            "--name",
            "SIG",
            "--kind",
            "local",
            "--x-nm",
            "0",
            "--y-nm",
            "0",
        ],
    );
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
            "One shared child",
        ],
    );
    let instances = [Uuid::new_v4(), Uuid::new_v4()];
    for (index, instance) in instances.iter().enumerate() {
        command(
            &root,
            "create-sheet-instance",
            &[
                "--definition",
                &definition.to_string(),
                "--instance",
                &instance.to_string(),
                "--name",
                &format!("Child {index}"),
                "--x-nm",
                "0",
                "--y-nm",
                "0",
            ],
        );
    }
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let source = snapshot.schematic_source().unwrap();
    assert_eq!(source.sheet_definitions.len(), 1);
    assert_eq!(source.sheet_instances.len(), 2);
    assert!(
        source
            .sheet_instances
            .values()
            .all(|i| i.definition == definition)
    );
    let source_pins: std::collections::BTreeSet<_> = source.sheets[&sheet]
        .symbols
        .values()
        .flat_map(|s| s.pins.iter().map(|p| p.uuid))
        .collect();
    assert_eq!(source_pins.len(), 2);
    drop(snapshot);
    drop(model);

    let before = saved_conflicts(&root);
    assert_ne!(before[0].fingerprint, before[1].fingerprint);
    assert_ne!(before[0].primary_target, before[1].primary_target);
    assert_eq!(
        before[0].payload["object_uuids"],
        before[1].payload["object_uuids"]
    );
    assert_ne!(before[0].payload["net_uuid"], before[1].payload["net_uuid"]);
    let paths: std::collections::BTreeSet<Vec<Uuid>> = before
        .iter()
        .map(|f| {
            assert_eq!(f.status, "active");
            assert!(f.waiver_refs.is_empty());
            assert_eq!(f.payload["net_name"], "SIG");
            let path: Vec<Uuid> =
                serde_json::from_value(f.primary_target["instance_path"].clone()).unwrap();
            assert_eq!(f.primary_target["kind"], "object_uuid");
            let pin = Uuid::parse_str(f.primary_target["id"].as_str().unwrap()).unwrap();
            assert!(source_pins.contains(&pin));
            for evidence in f.payload["pin_evidence"].as_array().unwrap() {
                assert_eq!(
                    evidence["instance_path"],
                    serde_json::to_value(&path).unwrap()
                );
            }
            assert_eq!(
                f.related_targets
                    .iter()
                    .filter(|t| t["kind"] == "object_uuid")
                    .count(),
                1
            );
            assert!(
                f.related_targets
                    .iter()
                    .any(|t| t["kind"] == "net" && t["id"] == f.payload["net_uuid"])
            );
            path
        })
        .collect();
    assert_eq!(paths, instances.into_iter().map(|id| vec![id]).collect());

    let selected = &before[0];
    let draft = command(
        &root,
        "waive-finding",
        &[
            "--fingerprint",
            &selected.fingerprint,
            "--rationale",
            "Only this repeated occurrence",
            "--created-by",
            "occurrence-test",
        ],
    );
    // A draft cannot waive either occurrence until public proposal acceptance.
    assert!(saved_conflicts(&root).iter().all(|f| f.status == "active"));
    super::super::main_tests_project_check_run_drc_lifecycle::accept_disposition_draft(
        &root, &draft,
    );
    let waiver = Uuid::parse_str(draft["waiver_id"].as_str().unwrap()).unwrap();
    for _ in 0..2 {
        let after = saved_conflicts(&root);
        assert_eq!(after.iter().filter(|f| f.status == "waived").count(), 1);
        assert_eq!(after.iter().filter(|f| f.status == "active").count(), 1);
        for finding in after {
            let original = before
                .iter()
                .find(|f| f.fingerprint == finding.fingerprint)
                .unwrap();
            assert_eq!(finding.primary_target, original.primary_target);
            assert_eq!(finding.related_targets, original.related_targets);
            if finding.fingerprint == selected.fingerprint {
                assert_eq!(finding.status, "waived");
                assert_eq!(finding.waiver_refs, vec![waiver]);
            } else {
                assert_eq!(finding.status, "active");
                assert!(finding.waiver_refs.is_empty());
            }
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
