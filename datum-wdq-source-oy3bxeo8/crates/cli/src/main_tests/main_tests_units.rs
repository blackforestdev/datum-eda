use super::*;

fn run(args: &[&str]) -> (serde_json::Value, i32) {
    let mut argv = vec!["datum-eda", "--format", "json", "units", "resolve-length"];
    argv.extend_from_slice(args);
    let (output, code) = execute_with_exit_code(Cli::try_parse_from(argv).unwrap()).unwrap();
    (serde_json::from_str(&output).unwrap(), code)
}

#[test]
fn units_resolve_length_preserves_nm_and_matches_explicit_suffixes() {
    let (canonical, canonical_code) = run(&["--canonical-nm", "5080000"]);
    assert_eq!(canonical_code, 0);
    assert_eq!(canonical["canonical_nm"], 5_080_000);

    for expression in ["5.08mm", "200mil", "0.2in", "5080000nm"] {
        let (report, code) = run(&["--expression", expression]);
        assert_eq!(code, 0, "{expression}: {report}");
        assert_eq!(report["canonical_nm"], 5_080_000);
        assert_eq!(report["unit_source"], "explicit_suffix");
    }
}

#[test]
fn units_resolve_length_requires_complete_context_and_refuses_dual_input() {
    let (contextual, code) = run(&[
        "--expression",
        "200",
        "--quantity",
        "board",
        "--unit",
        "mil",
        "--system",
        "metric",
        "--field",
        "board.track.width",
        "--project-id",
        "sensor-node",
    ]);
    assert_eq!(code, 0);
    assert_eq!(contextual["canonical_nm"], 5_080_000);
    assert_eq!(contextual["unit_source"], "context");
    assert_eq!(contextual["context"]["project_id"], "sensor-node");

    let (missing, missing_code) = run(&["--expression", "200"]);
    assert_eq!(missing_code, 2);
    assert_eq!(missing["refusal"], "missing_bare_expression_context");

    let (dual, dual_code) = run(&["--canonical-nm", "5080000", "--expression", "5.08mm"]);
    assert_eq!(dual_code, 2);
    assert_eq!(dual["refusal"], "conflicting_canonical_nm_and_expression");
}
