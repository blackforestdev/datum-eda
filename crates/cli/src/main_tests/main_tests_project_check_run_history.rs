use super::main_tests_project_check::{
    build_native_check_fixture, seed_board_process_aperture_fixture, unique_project_root,
};
use super::*;

fn check_source_bytes(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, out: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, out);
            } else if !path.ends_with(".datum/revision/writer.lock") {
                out.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    visit(root, root, &mut out);
    out
}

#[test]
fn guarded_check_run_refuses_foreign_and_stale_basis_before_persisting() {
    let root = unique_project_root("datum-eda-cli-guarded-check");
    create_native_project(&root, Some("Guarded ERC".to_string())).unwrap();
    build_native_check_fixture(&root);
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let project = model.project.project_id.to_string();
    let revision = model.model_revision.0;
    let job = Uuid::new_v4().to_string();
    let before = check_source_bytes(&root);
    let run = |project: &str, revision: &str| {
        execute(
            Cli::try_parse_from([
                "eda",
                "--format",
                "json",
                "check",
                "run",
                root.to_str().unwrap(),
                "--profile",
                "erc",
                "--expected-project",
                project,
                "--expected-revision",
                revision,
                "--progress-job",
                &job,
            ])
            .unwrap(),
        )
    };
    let foreign = Uuid::new_v4().to_string();
    assert!(
        run(&foreign, &revision)
            .unwrap_err()
            .to_string()
            .contains("basis mismatch")
    );
    assert_eq!(check_source_bytes(&root), before);
    let unsupported = execute(
        Cli::try_parse_from([
            "eda",
            "check",
            "run",
            root.to_str().unwrap(),
            "--profile",
            "drc",
            "--expected-project",
            &project,
            "--expected-revision",
            &revision,
            "--progress-job",
            &job,
        ])
        .unwrap(),
    )
    .unwrap_err();
    assert!(unsupported.to_string().contains("erc profile only"));
    assert_eq!(check_source_bytes(&root), before);
    assert!(
        run(&project, "stale-revision")
            .unwrap_err()
            .to_string()
            .contains("basis mismatch")
    );
    assert_eq!(check_source_bytes(&root), before);
    let output = run(&project, &revision).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["project_id"], project);
    assert_eq!(report["model_revision"], revision);
    assert_eq!(report["profile_id"], "erc");
    assert_eq!(report["persisted"], true);
    let saved = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let id = Uuid::parse_str(report["check_run_id"].as_str().unwrap()).unwrap();
    assert_eq!(saved.check_runs[&id].model_revision.0, revision);
    assert_eq!(saved.check_runs[&id].summary, report["summary"]);
    assert!(
        Cli::try_parse_from([
            "eda",
            "check",
            "run",
            root.to_str().unwrap(),
            "--progress-job",
            &job,
        ])
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn guarded_check_run_keeps_captured_inputs_and_refuses_changed_source_for_both_run_paths() {
    for existing_run in [false, true] {
        let root = unique_project_root("datum-eda-cli-check-capture-race");
        create_native_project(&root, Some("Captured ERC".into())).unwrap();
        build_native_check_fixture(&root);
        if existing_run {
            run_native_project_check_with_profile(&root, Some("erc")).unwrap();
        }
        let model = eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap();
        let revision = model.model_revision.clone();
        let captured = capture_native_erc_inputs(&root, &model).unwrap();
        let expected =
            serde_json::to_value(native_erc_report_from_schematic(&captured.schematic)).unwrap();
        assert!(!expected["erc"].as_array().unwrap().is_empty());

        // The actual canonical source changes after immutable input capture.
        // Use the existing typed writer, not a private-file race or test hook.
        let sheets: Vec<_> = captured.schematic.sheets.keys().copied().collect();
        for sheet in sheets {
            execute(
                Cli::try_parse_from([
                    "eda",
                    "project",
                    "delete-sheet",
                    root.to_str().unwrap(),
                    "--sheet",
                    &sheet.to_string(),
                ])
                .unwrap(),
            )
            .unwrap();
        }
        let changed = eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap();
        assert_ne!(changed.model_revision, revision);
        let current = query_native_project_check_run_with_profile(&root, Some("erc")).unwrap();
        assert_eq!(current.finding_count, 0);
        let before_finish = check_source_bytes(&root);

        // Computation after that change still uses the retained source only.
        let report = native_erc_report_from_schematic(&captured.schematic);
        assert_eq!(serde_json::to_value(&report).unwrap(), expected);
        let result = finish_check_run(&root, model, captured.project, report, "erc", true, true);
        assert!(
            result.is_err(),
            "changed basis must refuse even when the old run ID exists"
        );
        assert_eq!(check_source_bytes(&root), before_finish);
        let after = eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap();
        assert_eq!(after.check_runs.len(), usize::from(existing_run));
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn erc_summary_survives_check_run_persistence() {
    let root = unique_project_root("datum-eda-cli-erc-summary-history");
    create_native_project(&root, Some("ERC Summary".to_string())).unwrap();
    build_native_check_fixture(&root);
    let output = execute(
        Cli::try_parse_from([
            "eda",
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
    .unwrap();
    let run: serde_json::Value = serde_json::from_str(&output).unwrap();
    let raw = &run["raw_report"];
    assert!(raw["summary"]["warnings"].as_u64().unwrap() > 0);
    assert_eq!(run["summary"], raw["summary"]);
    assert_eq!(run["status"], raw["summary"]["status"]);
    let raw_findings = raw["erc"].as_array().unwrap();
    let findings = run["findings"].as_array().unwrap();
    assert_eq!(findings.len(), raw_findings.len());
    for (finding, original) in findings.iter().zip(raw_findings) {
        assert_eq!(&finding["payload"], original);
        assert_eq!(
            finding["severity"],
            original["severity"].as_str().unwrap().to_ascii_lowercase()
        );
    }
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "show",
            root.to_str().unwrap(),
            "--check-run",
            run["check_run_id"].as_str().unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    let show: serde_json::Value = serde_json::from_str(&output).unwrap();
    let saved = &show["check_run"];
    assert_eq!(saved["summary"], run["summary"]);
    assert_eq!(saved["raw_report"], run["raw_report"]);
    assert_eq!(saved["status"], run["status"]);
    for (saved, finding) in saved["findings"].as_array().unwrap().iter().zip(findings) {
        assert_eq!(saved["severity"], finding["severity"]);
        assert_eq!(saved["fingerprint"], finding["fingerprint"]);
        assert_eq!(saved["finding_id"], finding["finding_id"]);
        assert_eq!(saved["payload"], finding["payload"]);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn check_list_and_show_report_persisted_check_run_evidence() {
    let root = unique_project_root("datum-eda-cli-check-list-show");
    create_native_project(&root, Some("Check History Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);

    let run_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("check run should succeed");
    let run: serde_json::Value =
        serde_json::from_str(&run_output).expect("check-run JSON should parse");
    let check_run_id = run["check_run_id"].as_str().unwrap().to_string();

    let list_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "list",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("check list should succeed");
    let list: serde_json::Value = serde_json::from_str(&list_output).unwrap();
    assert_eq!(list["contract"], "check_run_list_v1");
    assert_eq!(list["check_run_count"], 1);
    assert_eq!(list["check_runs"][0]["check_run_id"], check_run_id);
    assert_eq!(list["check_runs"][0]["profile_id"], "native-combined");
    assert_eq!(list["check_runs"][0]["status"], run["status"]);
    assert_eq!(list["check_runs"][0]["finding_count"], run["finding_count"]);

    let show_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "show",
            root.to_str().unwrap(),
            "--check-run",
            &check_run_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("check show should succeed");
    let show: serde_json::Value = serde_json::from_str(&show_output).unwrap();
    assert_eq!(show["contract"], "check_run_record_v1");
    assert_eq!(show["check_run"]["check_run_id"], check_run_id);
    assert_eq!(show["check_run"]["model_revision"], run["model_revision"]);
    assert_eq!(show["check_run"]["finding_count"], run["finding_count"]);
    assert_eq!(show["check_run"]["raw_report"]["domain"], "combined");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn check_list_reports_latest_profile_history() {
    let root = unique_project_root("datum-eda-cli-check-list-history");
    create_native_project(&root, Some("Check History Demo".to_string()))
        .expect("initial scaffold should succeed");
    build_native_check_fixture(&root);

    let combined_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
            "--profile",
            "native-combined",
        ])
        .expect("CLI should parse"),
    )
    .expect("combined check run should succeed");
    let combined: serde_json::Value =
        serde_json::from_str(&combined_output).expect("combined check run should parse");
    let standards_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
            "--profile",
            "standards",
        ])
        .expect("CLI should parse"),
    )
    .expect("standards check run should succeed");
    let standards: serde_json::Value =
        serde_json::from_str(&standards_output).expect("standards check run should parse");

    let list_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "list",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("check list should succeed");
    let list: serde_json::Value =
        serde_json::from_str(&list_output).expect("check list should parse");

    assert_eq!(list["contract"], "check_run_list_v1");
    assert_eq!(list["check_run_count"], 2);
    assert_eq!(list["latest_check_run_id"], standards["check_run_id"]);
    assert_eq!(list["latest_profile_id"], "standards");
    let latest_profiles = list["profile_latest_check_runs"].as_array().unwrap();
    assert!(latest_profiles.iter().any(|entry| {
        entry["profile_id"] == "native-combined"
            && entry["check_run_id"] == combined["check_run_id"]
            && entry["finding_count"].as_u64().is_some()
    }));
    assert!(latest_profiles.iter().any(|entry| {
        entry["profile_id"] == "standards"
            && entry["check_run_id"] == standards["check_run_id"]
            && entry["finding_count"].as_u64().is_some()
    }));
    assert!(list["check_runs"].as_array().unwrap().iter().all(|entry| {
        entry["latest_for_profile"] == true
            && entry["coverage_count"].as_u64().is_some()
            && entry["proposal_refs"].as_array().is_some()
    }));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn check_show_enriches_persisted_record_with_current_repair_proposal_links() {
    let root = unique_project_root("datum-eda-cli-check-show-repair-links");
    create_native_project(&root, Some("Check Show Repair Links Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);

    let run_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("check run should succeed");
    let run: serde_json::Value =
        serde_json::from_str(&run_output).expect("check-run JSON should parse");
    let check_run_id = run["check_run_id"].as_str().unwrap().to_string();
    assert_eq!(run["proposal_refs"], serde_json::json!([]));

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "repair-standards",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("standards repair proposals should generate");

    let show_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "show",
            root.to_str().unwrap(),
            "--check-run",
            &check_run_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("check show should succeed");
    let show: serde_json::Value = serde_json::from_str(&show_output).unwrap();
    let shown_run = &show["check_run"];
    assert_eq!(shown_run["check_run_id"], check_run_id);
    assert_eq!(shown_run["proposal_refs"].as_array().unwrap().len(), 2);
    assert_eq!(shown_run["proposal_links"].as_array().unwrap().len(), 2);
    assert!(
        shown_run["proposal_links"]
            .as_array()
            .unwrap()
            .iter()
            .all(|link| link["command_templates"]["preview"]
                .as_str()
                .unwrap()
                .contains("datum-eda proposal preview "))
    );
    assert!(
        shown_run["findings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|finding| {
                if finding["code"] == "pad_mask_expansion_missing"
                    || finding["code"] == "pad_paste_reduction_missing"
                {
                    !finding["proposal_refs"].as_array().unwrap().is_empty()
                        && !finding["proposal_links"].as_array().unwrap().is_empty()
                        && finding["proposal_links"][0]["command_templates"]["preview"]
                            .as_str()
                            .unwrap()
                            .contains("datum-eda proposal preview ")
                } else {
                    true
                }
            })
    );

    let persisted_path = root.join(format!(".datum/check_runs/{check_run_id}.json"));
    let persisted: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&persisted_path).expect("persisted check run should read"),
    )
    .expect("persisted check run should parse");
    assert_eq!(persisted["proposal_refs"], serde_json::json!([]));

    let _ = std::fs::remove_dir_all(&root);
}
