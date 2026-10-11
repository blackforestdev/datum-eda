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
    assert_eq!(id.as_deref(), Some(CURRENT));
    let state = admit_record(empty.clone(), CURRENT, &record()).unwrap();
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
        assert_eq!(admit_record(empty.clone(), CURRENT, &wrong).unwrap(), empty);
    }
    let mut wrong = record();
    wrong["model_revision"] = json!("stale");
    assert_eq!(admit_record(empty.clone(), CURRENT, &wrong).unwrap(), empty);
}
#[test]
fn fallback_no_current_or_missing_basis_cannot_expose_historical_run() {
    let mut absent = history();
    absent["check_runs"][0]["model_revision"] = json!("stale");
    let (empty, id) = history_candidate(&absent);
    assert!(id.is_none());
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
