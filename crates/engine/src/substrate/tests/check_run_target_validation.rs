use super::super::check_run::persist_check_run;
use super::*;

#[test]
fn resolver_rejects_check_run_evidence_with_legacy_null_target() {
    let root = temp_project_root("invalid_check_run_null_target");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let check_run_id = Uuid::new_v5(&project_id, b"invalid-check-run-null-target");
    let check_run = CheckRun {
        schema_version: CHECK_RUN_SCHEMA_VERSION,
        check_run_id,
        project_id,
        model_revision: before.model_revision,
        profile_id: "native-combined".to_string(),
        status: "warning".to_string(),
        summary: serde_json::json!({ "status": "warning" }),
        finding_count: 1,
        findings: vec![CheckFinding {
            finding_id: Uuid::new_v5(&project_id, b"invalid-check-finding-null-target"),
            index: 0,
            source: "drc".to_string(),
            code: "track_width_below_min".to_string(),
            severity: "warning".to_string(),
            fingerprint: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
            domain: "drc".to_string(),
            rule_id: "track_width_below_min".to_string(),
            standards_basis: None,
            standards_basis_detail: None,
            rule_revision: None,
            import_key: None,
            status: "active".to_string(),
            primary_target: serde_json::Value::Null,
            related_targets: Vec::new(),
            message: "track width below minimum".to_string(),
            explanation: "track width below minimum".to_string(),
            suggested_next_action: None,
            evidence: Vec::new(),
            payload: serde_json::json!({ "objects": [] }),
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            waiver_refs: Vec::new(),
            deviation_refs: Vec::new(),
        }],
        proposal_refs: Vec::new(),
        proposal_links: Vec::new(),
        profile_basis: Default::default(),
        coverage: Vec::new(),
        raw_report: serde_json::json!({ "domain": "drc" }),
    };
    let error = persist_check_run(&root, &check_run)
        .expect_err("invalid generated evidence helper should reject null target");
    assert!(
        error
            .to_string()
            .contains("primary_target must be a typed target object")
    );
    let check_run_dir = root.join(".datum/check_runs");
    std::fs::create_dir_all(&check_run_dir).expect("check run dir should create");
    std::fs::write(
        check_run_dir.join(format!("{check_run_id}.json")),
        format!(
            "{}\n",
            to_json_deterministic(&check_run).expect("check run should serialize")
        ),
    )
    .expect("invalid check run should write");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("resolver should reject invalid check run without failing project resolution");

    assert!(!resolved.check_runs.contains_key(&check_run_id));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_check_run"
            && diagnostic
                .message
                .contains("primary_target must be a typed target object")
    }));
}

#[test]
fn resolver_rejects_check_run_evidence_with_invalid_fingerprint() {
    let root = temp_project_root("invalid_check_run_fingerprint");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let check_run_id = Uuid::new_v5(&project_id, b"invalid-check-run-fingerprint");
    let check_run = CheckRun {
        schema_version: CHECK_RUN_SCHEMA_VERSION,
        check_run_id,
        project_id,
        model_revision: before.model_revision,
        profile_id: "native-combined".to_string(),
        status: "warning".to_string(),
        summary: serde_json::json!({ "status": "warning" }),
        finding_count: 1,
        findings: vec![CheckFinding {
            finding_id: Uuid::new_v5(&project_id, b"invalid-check-finding-fingerprint"),
            index: 0,
            source: "drc".to_string(),
            code: "track_width_below_min".to_string(),
            severity: "warning".to_string(),
            fingerprint: "sha256:track-width".to_string(),
            domain: "drc".to_string(),
            rule_id: "track_width_below_min".to_string(),
            standards_basis: None,
            standards_basis_detail: None,
            rule_revision: None,
            import_key: None,
            status: "active".to_string(),
            primary_target: serde_json::json!({ "kind": "track", "id": "track-test" }),
            related_targets: Vec::new(),
            message: "track width below minimum".to_string(),
            explanation: "track width below minimum".to_string(),
            suggested_next_action: None,
            evidence: Vec::new(),
            payload: serde_json::json!({ "objects": [] }),
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            waiver_refs: Vec::new(),
            deviation_refs: Vec::new(),
        }],
        proposal_refs: Vec::new(),
        proposal_links: Vec::new(),
        profile_basis: Default::default(),
        coverage: Vec::new(),
        raw_report: serde_json::json!({ "domain": "drc" }),
    };
    let error = persist_check_run(&root, &check_run)
        .expect_err("invalid generated evidence helper should reject bad fingerprint");
    assert!(
        error
            .to_string()
            .contains("fingerprint must be a sha256:<64 lowercase hex> value")
    );
    let check_run_dir = root.join(".datum/check_runs");
    std::fs::create_dir_all(&check_run_dir).expect("check run dir should create");
    std::fs::write(
        check_run_dir.join(format!("{check_run_id}.json")),
        format!(
            "{}\n",
            to_json_deterministic(&check_run).expect("check run should serialize")
        ),
    )
    .expect("invalid check run should write");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("resolver should reject invalid check run without failing project resolution");

    assert!(!resolved.check_runs.contains_key(&check_run_id));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_check_run"
            && diagnostic
                .message
                .contains("fingerprint must be a sha256:<64 lowercase hex> value")
    }));
}

#[test]
fn resolver_rejects_check_run_evidence_with_invalid_standards_basis_detail() {
    let root = temp_project_root("invalid_check_run_standards_basis_detail");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let check_run_id = Uuid::new_v5(&project_id, b"invalid-check-run-standards-basis");
    let check_run = CheckRun {
        schema_version: CHECK_RUN_SCHEMA_VERSION,
        check_run_id,
        project_id,
        model_revision: before.model_revision,
        profile_id: "standards".to_string(),
        status: "warning".to_string(),
        summary: serde_json::json!({ "status": "warning" }),
        finding_count: 1,
        findings: vec![CheckFinding {
            finding_id: Uuid::new_v5(&project_id, b"invalid-check-finding-standards-basis"),
            index: 0,
            source: "drc".to_string(),
            code: "track_width_below_min".to_string(),
            severity: "warning".to_string(),
            fingerprint: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
            domain: "standards".to_string(),
            rule_id: "track_width_below_min".to_string(),
            standards_basis: Some("datum.process_aperture_and_geometry.current".to_string()),
            standards_basis_detail: Some(StandardsBasis {
                basis_id: String::new(),
                registry_entry_ref: "datum.registry.standards.process_aperture_and_geometry"
                    .to_string(),
                revision_or_profile: Some("current".to_string()),
                selected_by: "datum.check.profile".to_string(),
                selection_scope: "board_pads_tracks_vias".to_string(),
                basis_kind: "process_aperture_geometry".to_string(),
                disposition: "declared".to_string(),
                evidence_refs: Vec::new(),
                uncertainty: None,
                provenance: "fixture".to_string(),
            }),
            rule_revision: Some("v1".to_string()),
            import_key: None,
            status: "active".to_string(),
            primary_target: serde_json::json!({ "kind": "track", "id": "track-test" }),
            related_targets: Vec::new(),
            message: "track width below minimum".to_string(),
            explanation: "track width below minimum".to_string(),
            suggested_next_action: None,
            evidence: Vec::new(),
            payload: serde_json::json!({ "objects": [] }),
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            waiver_refs: Vec::new(),
            deviation_refs: Vec::new(),
        }],
        proposal_refs: Vec::new(),
        proposal_links: Vec::new(),
        profile_basis: Default::default(),
        coverage: Vec::new(),
        raw_report: serde_json::json!({ "domain": "standards" }),
    };
    let error = persist_check_run(&root, &check_run)
        .expect_err("invalid standards basis detail should be rejected");
    assert!(
        error
            .to_string()
            .contains("standards_basis_detail.basis_id must not be blank")
    );
}
