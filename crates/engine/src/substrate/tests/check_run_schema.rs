use super::super::check_run::persist_check_run;
use super::*;

fn minimal_check_run(
    project_id: Uuid,
    model_revision: ModelRevision,
    check_run_id: Uuid,
) -> CheckRun {
    CheckRun {
        schema_version: CHECK_RUN_SCHEMA_VERSION,
        check_run_id,
        project_id,
        model_revision,
        profile_id: "native-combined".to_string(),
        status: "ok".to_string(),
        summary: serde_json::json!({ "status": "ok" }),
        finding_count: 0,
        findings: Vec::new(),
        proposal_refs: Vec::new(),
        proposal_links: Vec::new(),
        profile_basis: CheckRunProfileBasis::default(),
        coverage: Vec::new(),
        raw_report: serde_json::json!({ "domain": "combined" }),
    }
}

#[test]
fn standards_basis_registry_resolves_only_known_v1_basis_ids() {
    let process_basis = standards_basis_for_id(PROCESS_APERTURE_STANDARDS_BASIS_ID)
        .expect("process aperture basis should be registered");
    assert_eq!(process_basis.basis_id, PROCESS_APERTURE_STANDARDS_BASIS_ID);
    assert_eq!(
        process_basis.registry_entry_ref,
        "datum.registry.standards.process_aperture_and_geometry"
    );
    assert_eq!(process_basis.basis_kind, "process_aperture_geometry");
    assert_eq!(
        standards_basis_id_for_check_code("pad_mask_expansion_missing"),
        Some(PROCESS_APERTURE_STANDARDS_BASIS_ID)
    );

    let zone_basis = standards_basis_for_id(ZONE_FILL_HONESTY_STANDARDS_BASIS_ID)
        .expect("zone fill honesty basis should be registered");
    assert_eq!(zone_basis.basis_id, ZONE_FILL_HONESTY_STANDARDS_BASIS_ID);
    assert_eq!(zone_basis.selection_scope, "board_zones");
    assert_eq!(
        standards_basis_id_for_check_code("zone_fill_unfilled"),
        Some(ZONE_FILL_HONESTY_STANDARDS_BASIS_ID)
    );

    assert!(standards_basis_for_id("datum.unknown.current").is_none());
    assert!(standards_basis_id_for_check_code("unknown_rule").is_none());
}

#[test]
fn check_run_helper_rejects_registered_standards_basis_registry_mismatch() {
    let root = temp_project_root("check_run_registered_standards_basis_mismatch");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let check_run_id = Uuid::new_v5(&project_id, b"registered-standards-basis-mismatch");
    let mut check_run = minimal_check_run(project_id, model.model_revision, check_run_id);
    let mut detail = standards_basis_for_id(PROCESS_APERTURE_STANDARDS_BASIS_ID)
        .expect("process aperture basis should be registered");
    detail.selected_by = "fixture.override".to_string();
    check_run.profile_basis.standards_basis_detail = Some(detail);

    let error = persist_check_run(&root, &check_run)
        .expect_err("registered standards basis mismatch should be rejected");

    assert!(
        error
            .to_string()
            .contains("selected_by must match registry value datum.check.profile")
    );
    assert!(
        !root
            .join(format!(".datum/check_runs/{check_run_id}.json"))
            .exists()
    );
}

#[test]
fn check_run_helper_accepts_unknown_structurally_valid_standards_basis() {
    let root = temp_project_root("check_run_unknown_structurally_valid_standards_basis");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let check_run_id = Uuid::new_v5(&project_id, b"unknown-standards-basis-valid");
    let mut check_run = minimal_check_run(project_id, model.model_revision, check_run_id);
    check_run.profile_basis.standards_basis_detail = Some(StandardsBasis {
        basis_id: "project.custom.standard.current".to_string(),
        registry_entry_ref: "project.registry.standards.custom".to_string(),
        revision_or_profile: Some("current".to_string()),
        selected_by: "project.check.profile".to_string(),
        selection_scope: "project_scope".to_string(),
        basis_kind: "project_standard".to_string(),
        disposition: "declared".to_string(),
        evidence_refs: Vec::new(),
        uncertainty: None,
        provenance: "project registry fixture".to_string(),
    });

    persist_check_run(&root, &check_run)
        .expect("unknown structurally valid standards basis should persist");

    assert!(
        root.join(format!(".datum/check_runs/{check_run_id}.json"))
            .exists()
    );
}

#[test]
fn check_run_helper_rejects_unsupported_payload_schema_version() {
    let root = temp_project_root("check_run_helper_rejects_payload_schema");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let check_run_id = Uuid::new_v5(&project_id, b"unsupported-check-run-payload-schema");
    let mut check_run = minimal_check_run(project_id, model.model_revision, check_run_id);
    check_run.schema_version = CHECK_RUN_SCHEMA_VERSION + 1;

    let error = persist_check_run(&root, &check_run)
        .expect_err("unsupported check run schema should be rejected");

    assert!(
        error
            .to_string()
            .contains("unsupported check run schema_version 2")
    );
    assert!(
        !root
            .join(format!(".datum/check_runs/{check_run_id}.json"))
            .exists()
    );
}

#[test]
fn resolver_defaults_legacy_check_run_payload_schema_version() {
    let root = temp_project_root("legacy_check_run_payload_schema");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let check_run_id = Uuid::new_v5(&project_id, b"legacy-check-run-payload-schema");

    write_json(
        &root.join(format!(".datum/check_runs/{check_run_id}.json")),
        serde_json::json!({
            "check_run_id": check_run_id,
            "project_id": project_id,
            "model_revision": model.model_revision.0,
            "profile_id": "native-combined",
            "status": "ok",
            "summary": { "status": "ok" },
            "finding_count": 0,
            "findings": [],
            "proposal_refs": [],
            "proposal_links": [],
            "profile_basis": {},
            "coverage": [],
            "raw_report": { "domain": "combined" }
        }),
    );

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve legacy check run");

    assert_eq!(
        resolved.check_runs[&check_run_id].schema_version,
        CHECK_RUN_SCHEMA_VERSION
    );
}
