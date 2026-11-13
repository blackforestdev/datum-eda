//! Persisted public check/disposition workflows; fixture seeding precedes writes.
use super::*;
use eda_engine::api::native_write::{BatchComposer, WriteProvenance, commit_prepared};
use eda_engine::substrate::{CheckRun, Operation, SourceShardKind};

fn command(root: &Path, args: &[&str]) -> serde_json::Value {
    let mut words = vec![
        "datum-eda",
        "--format",
        "json",
        "project",
        args[0],
        root.to_str().unwrap(),
    ];
    words.extend_from_slice(&args[1..]);
    serde_json::from_str(&execute(Cli::try_parse_from(words).unwrap()).unwrap()).unwrap()
}
fn finding(root: &Path, source: &str, code: &str) -> serde_json::Value {
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
                "native-combined",
            ])
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(report["persisted"], true);
    let run_id = Uuid::parse_str(report["check_run_id"].as_str().unwrap()).unwrap();
    let saved: CheckRun = serde_json::from_str(
        &std::fs::read_to_string(root.join(format!(".datum/check_runs/{run_id}.json")))
            .expect("public check run must save its evidence"),
    )
    .unwrap();
    // Resolve after the command and disk read; inspect the actual saved CheckRun,
    // rather than treating a freshly computed read-only view as durable evidence.
    let cold = ProjectResolver::new(root).resolve().unwrap();
    assert_eq!(cold.check_runs.get(&run_id), Some(&saved));
    assert!(
        cold.source_shards
            .iter()
            .any(|s| s.kind == SourceShardKind::CheckRun)
    );
    let emitted = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["source"] == source && f["code"] == code)
        .unwrap();
    let stored = saved
        .findings
        .iter()
        .find(|f| f.source == source && f.code == code)
        .unwrap();
    assert_eq!(stored.fingerprint, emitted["fingerprint"].as_str().unwrap());
    assert_eq!(stored.status, emitted["status"].as_str().unwrap());
    assert_eq!(
        serde_json::to_value(&stored.waiver_refs).unwrap(),
        emitted["waiver_refs"]
    );
    serde_json::to_value(stored).unwrap()
}
fn waive(root: &Path, fingerprint: &str) -> Uuid {
    let draft = command(
        root,
        &[
            "waive-finding",
            "--fingerprint",
            fingerprint,
            "--rationale",
            "Reviewed intentional finding",
            "--created-by",
            "rename-test",
        ],
    );
    super::super::main_tests_project_check_run_drc_lifecycle::accept_disposition_draft(
        root, &draft,
    );
    Uuid::parse_str(draft["waiver_id"].as_str().unwrap()).unwrap()
}
fn apply(root: &Path, operation: Operation) {
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    let prepared = BatchComposer::compose(
        &model,
        WriteProvenance::new(
            "rename-test",
            CommitSource::Test,
            "Relevant observation control",
        ),
    )
    .push_ops([operation])
    .finish()
    .unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    drop(model);
    ProjectResolver::new(root)
        .resolve()
        .expect("saved mutation must cold reopen");
}

#[test]
fn erc_refdes_rename_preserves_reviewed_conflict_across_check_and_cold_reopen() {
    let root = unique_project_root("erc-refdes-correlation");
    create_native_project(&root, Some("ERC correlation".into())).unwrap();
    build_native_check_fixture(&root);
    // Seed a coherent pair of output symbols before exercising ordinary writes.
    let path = std::fs::read_dir(root.join("schematic/sheets"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let sheet: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let sheet_id = Uuid::parse_str(sheet["uuid"].as_str().unwrap()).unwrap();
    let template = sheet["symbols"]
        .as_object()
        .unwrap()
        .values()
        .find(|s| s["reference"] == "R1")
        .unwrap();
    let mut symbols = BTreeMap::new();
    let first = Uuid::new_v4();
    for (id, reference) in [(first, "U1"), (Uuid::new_v4(), "U2")] {
        let mut symbol = template.clone();
        symbol["uuid"] = serde_json::json!(id);
        symbol["reference"] = serde_json::json!(reference);
        symbol["pins"][0]["uuid"] = serde_json::json!(Uuid::new_v4());
        symbol["pins"][0]["electrical_type"] =
            serde_json::to_value(PinElectricalType::Output).unwrap();
        symbols.insert(id.to_string(), symbol);
    }
    let label = Uuid::new_v4();
    write_native_sheet(
        &root,
        sheet_id,
        "Root",
        symbols,
        BTreeMap::from([(
            label.to_string(),
            serde_json::to_value(NetLabel {
                uuid: label,
                kind: LabelKind::Local,
                name: "DATA".into(),
                position: Point::new(5, 5),
            })
            .unwrap(),
        )]),
    );
    let before = finding(&root, "erc", "output_to_output_conflict");
    let fp = before["fingerprint"].as_str().unwrap();
    let waiver = waive(&root, fp);
    command(
        &root,
        &[
            "set-symbol-reference",
            "--symbol",
            &first.to_string(),
            "--reference",
            "U7",
        ],
    );
    for _ in 0..2 {
        let after = finding(&root, "erc", "output_to_output_conflict");
        assert_eq!(after["fingerprint"], fp);
        assert_eq!(after["status"], "waived");
        assert_eq!(after["waiver_refs"], serde_json::json!([waiver]));
        assert_ne!(after["message"], before["message"]);
    }
    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut symbol = saved["symbols"][first.to_string()].clone();
    symbol["pins"][0]["electrical_type"] =
        serde_json::to_value(PinElectricalType::PowerOut).unwrap();
    apply(
        &root,
        Operation::SetSchematicSymbol {
            sheet_id,
            symbol_id: first,
            symbol,
        },
    );
    let changed = finding(&root, "erc", "output_to_output_conflict");
    assert_ne!(changed["fingerprint"], fp);
    assert_eq!(changed["status"], "active");
    assert_eq!(changed["waiver_refs"], serde_json::json!([]));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn drc_net_rename_preserves_waiver_but_geometry_change_reopens() {
    let root = unique_project_root("drc-net-correlation");
    create_native_project(&root, Some("DRC correlation".into())).unwrap();
    build_native_check_fixture(&root);
    let net_id = seed_board_drc_fixture(&root);
    let before = finding(&root, "drc", "connectivity_unrouted_net");
    let fp = before["fingerprint"].as_str().unwrap();
    let waiver = waive(&root, fp);
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let mut net = board["nets"][net_id.to_string()].clone();
    net["name"] = serde_json::json!("RENAMED");
    drop(model);
    apply(&root, Operation::SetBoardNet { net_id, net });
    for _ in 0..2 {
        let after = finding(&root, "drc", "connectivity_unrouted_net");
        assert_eq!(after["fingerprint"], fp);
        assert_eq!(after["status"], "waived");
        assert_eq!(after["waiver_refs"], serde_json::json!([waiver]));
        assert_ne!(after["message"], before["message"]);
        // A fingerprint waiver for one rule does not waive another finding.
        assert_eq!(
            finding(&root, "drc", "connectivity_no_copper")["status"],
            "active"
        );
    }
    let mut pad = board["pads"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .clone();
    let pad_id = Uuid::parse_str(pad["uuid"].as_str().unwrap()).unwrap();
    pad["position"]["x"] = serde_json::json!(9_000_000);
    apply(&root, Operation::SetBoardPad { pad_id, pad });
    let changed = finding(&root, "drc", "connectivity_unrouted_net");
    assert_ne!(changed["fingerprint"], fp);
    assert_eq!(changed["status"], "active");
    assert_eq!(changed["waiver_refs"], serde_json::json!([]));
    std::fs::remove_dir_all(root).unwrap();
}
