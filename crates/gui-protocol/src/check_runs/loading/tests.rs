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
