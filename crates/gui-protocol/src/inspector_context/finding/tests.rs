use super::*;
use eda_engine::api::native_write::{
    commit_prepared,
    context::WriteProvenance,
    genesis::{GenesisSpec, bootstrap_native_project},
    waivers::{
        CreateSchematicDeviationRequest, CreateSchematicWaiverRequest,
        build_create_schematic_deviation, build_create_schematic_waiver,
    },
};
use eda_engine::substrate::{CommitSource, ProjectResolver};
use serde_json::json;
use uuid::Uuid;

fn fixture() -> (std::path::PathBuf, DesignModel, ReviewWorkspaceState, Uuid) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.datum/scratch/erc-readable-inspector-tests")
        .join(Uuid::new_v4().to_string());
    let genesis = bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Finding inspection".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let mut state = crate::load_fixture_workspace_state();
    state.scene.project_uuid = model.project.project_id.to_string();
    state.scene.source_revision = model.model_revision.0.clone();
    state.checks = CheckRunReviewState {
        project_id: Some(state.scene.project_uuid.clone()),
        model_revision: Some(state.scene.source_revision.clone()),
        check_run_id: Some(Uuid::new_v4().to_string()),
        persisted: true,
        profile_id: Some("erc".into()),
        finding_count: 1,
        findings: vec![CheckFindingSummary {
            fingerprint: "erc:pin".into(),
            domain: "erc".into(),
            rule_id: "unconnected_component_pin".into(),
            status: "active".into(),
            message: "Q1.3 has no connection".into(),
            evidence: vec![json!({"objects":[{"kind":"pin","key":"Q1.3"}]})],
            ..Default::default()
        }],
        ..Default::default()
    };
    let native = crate::selection_resolution::NativeSelectionResolution::capture(&model).unwrap();
    let view = crate::selection_artifacts::build_workspace_selection_view(
        &model,
        &native,
        SelectionSubject::Diagnostic("erc:pin".into()),
        &state,
    )
    .unwrap();
    state.selection = SelectionTarget::Native(Box::new(view));
    (root, model, state, genesis.schematic_uuid)
}
#[test]
fn erc_readable_inspector_qualifies_actual_diagnostic_and_report_currentness() {
    let (root, _, mut state, _) = fixture();
    let view = capture(&state).unwrap().unwrap();
    assert_eq!(view.target.as_deref(), Some("Q1.3"));
    assert_eq!(view.currentness, "Current persisted result");
    state.checks.persisted = false;
    assert!(
        capture(&state)
            .unwrap()
            .unwrap()
            .currentness
            .contains("read-only")
    );
    state.scene.source_revision.push_str("-new");
    assert!(
        capture(&state)
            .unwrap()
            .unwrap()
            .currentness
            .starts_with("Stale")
    );
    // The same fingerprint in another producing run cannot replace selected evidence.
    let before = state.checks.clone();
    state.checks.check_run_id = Some(Uuid::new_v4().to_string());
    assert!(capture(&state).unwrap().is_err());
    state.checks = before;
    state.checks.findings.push(state.checks.findings[0].clone());
    assert!(capture(&state).unwrap().is_err());
    state.checks.findings.pop();
    state.checks.project_id = Some(Uuid::new_v4().to_string());
    assert!(capture(&state).unwrap().is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn erc_readable_inspector_captures_only_exact_canonical_rationale_without_historical_overlay() {
    let (root, mut model, mut state, schematic) = fixture();
    let provenance = || WriteProvenance::new("test", CommitSource::Test, "finding rationale");
    let waiver = build_create_schematic_waiver(
        &model,
        provenance(),
        &CreateSchematicWaiverRequest {
            schematic_id: schematic,
            domain: CheckDomain::ERC,
            fingerprint: "erc:pin".into(),
            rationale: "Intentional test point".into(),
            created_by: None,
        },
    )
    .unwrap();
    let waiver_id = waiver.waiver_id;
    commit_prepared(&mut model, &root, waiver.write).unwrap();
    let deviation = build_create_schematic_deviation(
        &model,
        provenance(),
        &CreateSchematicDeviationRequest {
            schematic_id: schematic,
            domain: CheckDomain::ERC,
            fingerprint: "erc:deviation".into(),
            rationale: "Approved alternate population".into(),
            accepted_by: None,
        },
    )
    .unwrap();
    let deviation_id = deviation.deviation_id;
    commit_prepared(&mut model, &root, deviation.write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    state.checks.findings[0].waiver_refs = vec![waiver_id.to_string()];
    state.checks.findings[0].status = "waived".into();
    state.checks.findings.push(CheckFindingSummary {
        fingerprint: "erc:deviation".into(),
        domain: "erc".into(),
        status: "accepted_deviation".into(),
        deviation_refs: vec![deviation_id.to_string()],
        ..Default::default()
    });
    state.checks.finding_count = 2;
    // Old report references alone do not allow borrowing current root rationale.
    capture_dispositions(&mut state.checks, &model).unwrap();
    assert!(state.checks.findings[0].disposition_details.is_empty());
    state.checks.model_revision = Some(model.model_revision.0.clone());
    capture_dispositions(&mut state.checks, &model).unwrap();
    let details = &state.checks.findings[0].disposition_details;
    assert_eq!(details.len(), 1);
    assert_eq!(details[0].rationale, "Intentional test point");
    assert_eq!(
        state.checks.findings[1].disposition_details[0].rationale,
        "Approved alternate population"
    );
    // Report serialization cannot introduce these locally qualified details.
    let roundtrip: CheckRunReviewState =
        serde_json::from_value(serde_json::to_value(&state.checks).unwrap()).unwrap();
    assert!(roundtrip.findings[0].disposition_details.is_empty());
    state.checks.findings[0].fingerprint = "erc:other".into();
    capture_dispositions(&mut state.checks, &model).unwrap();
    assert!(state.checks.findings[0].disposition_details.is_empty());
    state.checks.findings[0].fingerprint = "erc:pin".into();
    state.checks.findings[0]
        .waiver_refs
        .push(waiver_id.to_string());
    capture_dispositions(&mut state.checks, &model).unwrap();
    assert_eq!(
        state.checks.findings[0].disposition_details.len(),
        0,
        "ambiguous duplicate waiver reference is not displayed"
    );
    std::fs::remove_dir_all(root).unwrap();
}
