use super::*;
use serde_json::json;
const PROJECT: &str = "f5c84dbb-dafc-4caf-ba86-b3b6c894651b";
const CURRENT: &str = "cc1b83c4-3106-57de-9e0f-74349dde78a0";
const OLD: &str = "2f808923-15a2-5910-b40e-b79052754aff";

fn history() -> Value {
    json!({"contract":"check_run_list_v1", "project_id":PROJECT,"model_revision":"current",
        "latest_check_run_id":OLD,"check_runs":[
            {"check_run_id":CURRENT,"project_id":PROJECT,"model_revision":"current"},
            {"check_run_id":OLD,"project_id":PROJECT,"model_revision":"stale"}]})
}
fn record() -> Value {
    json!({"contract":"check_run_record_v1","project_id":PROJECT,"model_revision":"current",
        "check_run":{"check_run_id":CURRENT,"project_id":PROJECT,"model_revision":"current",
        "status":"ok","findings":[],"finding_count":0}})
}
#[test]
fn fallback_uses_current_summary_and_checks_exact_shown_record() {
    let (empty, id) = history_candidate(&history());
    assert_eq!(
        id.as_ref().map(|candidate| candidate.id.as_str()),
        Some(CURRENT)
    );
    let candidate = id.unwrap();
    let state = admit_record(empty.clone(), &candidate, &record()).unwrap();
    assert_eq!(state.check_run_id.as_deref(), Some(CURRENT));
    assert_eq!(state.status.as_deref(), Some("ok"));
    assert!(state.findings.is_empty());
    for (field, value) in [
        ("project_id", OLD),
        ("model_revision", "stale"),
        ("check_run_id", OLD),
    ] {
        let mut wrong = record();
        wrong["check_run"][field] = json!(value);
        assert_eq!(
            admit_record(empty.clone(), &candidate, &wrong).unwrap(),
            empty
        );
    }
    let mut wrong = record();
    wrong["model_revision"] = json!("stale");
    assert_eq!(
        admit_record(empty.clone(), &candidate, &wrong).unwrap(),
        empty
    );
}
#[test]
fn fallback_retains_actual_history_without_granting_current_authority() {
    let mut absent = history();
    absent["check_runs"][0]["model_revision"] = json!("stale");
    let (empty, id) = history_candidate(&absent);
    let candidate = id.unwrap();
    assert_eq!(candidate.id, OLD);
    assert_eq!(candidate.revision, "stale");
    let mut historical = record();
    historical["check_run"]["check_run_id"] = json!(OLD);
    historical["check_run"]["model_revision"] = json!("stale");
    historical["check_run"]["finding_count"] = json!(13);
    let retained = admit_record(empty.clone(), &candidate, &historical).unwrap();
    assert_eq!(retained.model_revision.as_deref(), Some("stale"));
    assert_eq!(retained.finding_count, 13);
    assert!(retained.persisted);
    assert_eq!(empty.project_id.as_deref(), Some(PROJECT));
    assert_eq!(empty.model_revision.as_deref(), Some("current"));
    assert!(empty.check_run_id.is_none() && empty.status.is_none() && empty.findings.is_empty());
    for field in ["project_id", "model_revision", "contract"] {
        let mut malformed = history();
        malformed.as_object_mut().unwrap().remove(field);
        assert_eq!(
            history_candidate(&malformed),
            (CheckRunReviewState::default(), None)
        );
    }
    let mut foreign = history();
    foreign["check_runs"][0]["project_id"] = json!(OLD);
    let (_, candidate) = history_candidate(&foreign);
    assert_eq!(candidate.unwrap().id, OLD);
    foreign["check_runs"][1]["project_id"] = json!(OLD);
    assert!(history_candidate(&foreign).1.is_none());
}

#[test]
fn malformed_history_basis_is_entirely_unknown() {
    for (field, value) in [
        ("contract", json!("wrong")),
        ("project_id", json!("invalid")),
        ("model_revision", json!(null)),
        ("model_revision", json!(5)),
        ("model_revision", json!("")),
    ] {
        let mut malformed = history();
        malformed[field] = value;
        assert_eq!(
            history_candidate(&malformed),
            (CheckRunReviewState::default(), None)
        );
    }
}

#[test]
fn erc_disposition_report_never_upgrades_read_only_evidence_or_foreign_basis() {
    let mut state = crate::load_fixture_workspace_state();
    state.scene.project_uuid = PROJECT.into();
    state.scene.source_revision = "current".into();
    let good = CheckRunReviewState {
        check_run_id: Some(CURRENT.into()),
        project_id: Some(PROJECT.into()),
        model_revision: Some("current".into()),
        profile_id: Some("erc".into()),
        persisted: false,
        ..Default::default()
    };
    assert!(admit_projection(&state, &good).is_ok());
    for variant in 0..4 {
        let mut bad = good.clone();
        match variant {
            0 => bad.persisted = true,
            1 => bad.project_id = Some(OLD.into()),
            2 => bad.model_revision = Some("old".into()),
            _ => bad.profile_id = Some("drc".into()),
        }
        assert!(admit_projection(&state, &bad).is_err());
    }
    assert!(!good.persisted);
}

#[test]
fn erc_disposition_report_survives_save_but_not_source_change_or_current_persisted_run() {
    let mut state = crate::load_fixture_workspace_state();
    state.scene.project_uuid = PROJECT.into();
    state.scene.source_revision = "current".into();
    let report = CheckRunReviewState {
        check_run_id: Some(CURRENT.into()),
        project_id: Some(PROJECT.into()),
        model_revision: Some("current".into()),
        profile_id: Some("erc".into()),
        persisted: false,
        status: Some("waived".into()),
        finding_count: 1,
        ..Default::default()
    };
    state.checks = CheckRunReviewState::default();
    retain_current_check_projection(&mut state, &report);
    assert_eq!(state.checks, report);
    assert_eq!(state.supervision.checks.status, report.status);
    state.scene.source_revision = "geometry-changed".into();
    state.checks = CheckRunReviewState::default();
    retain_current_check_projection(&mut state, &report);
    assert_eq!(state.checks, CheckRunReviewState::default());
    state.scene.source_revision = "current".into();
    let persisted = CheckRunReviewState {
        persisted: true,
        status: Some("persisted".into()),
        ..report.clone()
    };
    state.checks = persisted.clone();
    retain_current_check_projection(&mut state, &report);
    assert_eq!(state.checks, persisted);
}

#[test]
fn erc_disposition_basis_is_frozen_before_canonical_root_change_and_history() {
    use eda_engine::api::native_write::{
        commit_prepared,
        context::WriteProvenance,
        genesis::{GenesisSpec, bootstrap_native_project},
        schematic_sheets::build_create_schematic_sheet,
        waivers::{CreateSchematicWaiverRequest, build_create_schematic_waiver},
    };
    use eda_engine::schematic::CheckDomain;
    use eda_engine::substrate::{CommitSource, ProjectResolver};
    use uuid::Uuid;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.datum/scratch/erc-disposition-refresh-tests")
        .join(Uuid::new_v4().to_string());
    let genesis = bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "ERC refresh basis".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let provenance = || WriteProvenance::new("test", CommitSource::Test, "ERC refresh basis");
    let sheet = Uuid::new_v4();
    let write = build_create_schematic_sheet(&model, provenance(), genesis.schematic_uuid, sheet,
        "sheets/main.json", json!({"schema_version":1,"uuid":sheet,"name":"Main","symbols":{},"wires":{},"labels":{},"buses":{},"bus_entries":{},"ports":{},"junctions":{},"noconnects":{},"texts":{},"drawings":{}})).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    // This separately resolved, unchanged model is the worker's prior workspace.
    let previous = ProjectResolver::new(&root).resolve().unwrap();
    let frozen = disposition_basis(&previous).unwrap();
    let prepared = build_create_schematic_waiver(
        &model,
        provenance(),
        &CreateSchematicWaiverRequest {
            schematic_id: genesis.schematic_uuid,
            domain: CheckDomain::ERC,
            fingerprint: "erc-refresh-qualified-fingerprint".into(),
            rationale: "Intentional test disposition".into(),
            created_by: Some("test".into()),
        },
    )
    .unwrap();
    let waiver = prepared.waiver_id;
    commit_prepared(&mut model, &root, prepared.write).unwrap();
    let mut current = ProjectResolver::new(&root).resolve().unwrap();
    let applied = disposition_basis(&current).unwrap();
    assert_ne!(frozen, applied);
    assert_eq!(applied["waivers"][0]["uuid"], json!(waiver));
    assert_ne!(previous.model_revision, current.model_revision);
    // Comparison uses the frozen value; refresh itself adds no history entry.
    let tail = current.journal.len();
    assert_ne!(frozen, disposition_basis(&current).unwrap());
    assert_eq!(current.journal.len(), tail);
    current
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    current = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(disposition_basis(&current).unwrap(), frozen);
    current
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    current = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(disposition_basis(&current).unwrap(), applied);
    std::fs::remove_dir_all(root).unwrap();
}
