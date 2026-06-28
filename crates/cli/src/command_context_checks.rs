use eda_engine::substrate::{CheckFinding, CheckRun, DesignModel};
use serde_json::{Map, Value};

const MAX_CONTEXT_CHECK_RUNS: usize = 4;
const MAX_CONTEXT_FINDINGS_PER_RUN: usize = 8;

pub(super) fn check_context_summary(model: &DesignModel) -> Value {
    let mut check_runs = model.check_runs.values().collect::<Vec<_>>();
    check_runs.sort_by(compare_check_runs);
    let latest_check_run_id = check_runs
        .last()
        .map(|run| Value::String(run.check_run_id.to_string()))
        .unwrap_or(Value::Null);
    let latest_profile_id = check_runs
        .last()
        .map(|run| Value::String(run.profile_id.clone()))
        .unwrap_or(Value::Null);
    let mut latest_by_profile = std::collections::BTreeMap::<String, &CheckRun>::new();
    for run in &check_runs {
        latest_by_profile.insert(run.profile_id.clone(), run);
    }
    let profile_latest_check_runs = latest_by_profile
        .values()
        .map(|run| {
            serde_json::json!({
                "profile_id": run.profile_id,
                "check_run_id": run.check_run_id,
                "model_revision": run.model_revision.0,
                "status": run.status,
                "finding_count": run.finding_count
            })
        })
        .collect::<Vec<_>>();
    let visible = check_runs
        .into_iter()
        .rev()
        .take(MAX_CONTEXT_CHECK_RUNS)
        .map(check_run_context_summary)
        .collect::<Vec<_>>();
    serde_json::json!({
        "contract": "datum_check_context_v1",
        "check_run_count": model.check_runs.len(),
        "visible_check_run_count": visible.len(),
        "latest_check_run_id": latest_check_run_id,
        "latest_profile_id": latest_profile_id,
        "profile_latest_check_runs": profile_latest_check_runs,
        "visible_check_runs": visible,
        "agent_commands": {
            "run_checks": "datum-eda check run \"$DATUM_PROJECT_ROOT\"",
            "list_check_runs": "datum-eda check list \"$DATUM_PROJECT_ROOT\"",
            "show_check_run": "datum-eda check show \"$DATUM_PROJECT_ROOT\" --check-run <check_run_id>",
            "repair_standards": "datum-eda check repair-standards \"$DATUM_PROJECT_ROOT\"",
            "waive_finding": "datum-eda check waive \"$DATUM_PROJECT_ROOT\" --fingerprint <fingerprint> --rationale <rationale>",
            "accept_deviation": "datum-eda check accept-deviation \"$DATUM_PROJECT_ROOT\" --fingerprint <fingerprint> --rationale <rationale>"
        }
    })
}

fn compare_check_runs(left: &&CheckRun, right: &&CheckRun) -> std::cmp::Ordering {
    left.model_revision
        .0
        .cmp(&right.model_revision.0)
        .then_with(|| left.profile_id.cmp(&right.profile_id))
        .then_with(|| left.check_run_id.cmp(&right.check_run_id))
}

pub(super) fn visible_check_run_ids_from_context(check_context: &Value) -> Value {
    Value::Array(
        check_context
            .get("visible_check_runs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|run| run.get("check_run_id").and_then(Value::as_str))
            .map(|id| Value::String(id.to_string()))
            .collect(),
    )
}

pub(super) fn visible_finding_fingerprints_from_context(check_context: &Value) -> Value {
    Value::Array(
        check_context
            .get("visible_check_runs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .flat_map(|run| {
                run.get("active_findings")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
            })
            .filter_map(|finding| finding.get("fingerprint").and_then(Value::as_str))
            .filter(|fingerprint| !fingerprint.is_empty())
            .map(|fingerprint| Value::String(fingerprint.to_string()))
            .collect(),
    )
}

pub(super) fn check_status_from_context(check_context: &Value, model: &DesignModel) -> Value {
    let Some(run) = check_context
        .get("visible_check_runs")
        .and_then(Value::as_array)
        .and_then(|runs| runs.first())
    else {
        return Value::Object(Map::new());
    };
    serde_json::json!({
        "check_run_id": run.get("check_run_id").cloned().unwrap_or(Value::Null),
        "project_id": model.project.project_id.to_string(),
        "model_revision": run.get("model_revision").cloned().unwrap_or(Value::Null),
        "profile_id": run.get("profile_id").cloned().unwrap_or(Value::Null),
        "status": run.get("status").cloned().unwrap_or(Value::Null),
        "persisted": true,
        "summary": Value::Null,
        "finding_count": run.get("finding_count").cloned().unwrap_or(Value::Null),
        "findings": run.get("active_findings").cloned().unwrap_or_else(|| Value::Array(Vec::new())),
        "proposal_refs": run.get("proposal_refs").cloned().unwrap_or_else(|| Value::Array(Vec::new())),
        "proposal_links": Value::Array(Vec::new()),
        "profile_basis": run.get("profile_basis").cloned().unwrap_or(Value::Null),
        "coverage": run.get("coverage").cloned().unwrap_or_else(|| Value::Array(Vec::new()))
    })
}

fn check_run_context_summary(run: &CheckRun) -> Value {
    let active_findings = run
        .findings
        .iter()
        .filter(|finding| finding.status != "waived" && finding.status != "accepted_deviation")
        .take(MAX_CONTEXT_FINDINGS_PER_RUN)
        .map(check_finding_context_summary)
        .collect::<Vec<_>>();
    serde_json::json!({
        "check_run_id": run.check_run_id,
        "model_revision": run.model_revision.0,
        "profile_id": run.profile_id,
        "status": run.status,
        "finding_count": run.finding_count,
        "active_context_finding_count": active_findings.len(),
        "proposal_refs": run.proposal_refs,
        "profile_basis": run.profile_basis,
        "coverage": run.coverage,
        "active_findings": active_findings
    })
}

fn check_finding_context_summary(finding: &CheckFinding) -> Value {
    serde_json::json!({
        "finding_id": finding.finding_id,
        "index": finding.index,
        "source": finding.source,
        "code": finding.code,
        "severity": finding.severity,
        "fingerprint": finding.fingerprint,
        "domain": finding.domain,
        "rule_id": finding.rule_id,
        "standards_basis": finding.standards_basis,
        "standards_basis_detail": finding.standards_basis_detail,
        "rule_revision": finding.rule_revision,
        "import_key": finding.import_key,
        "status": finding.status,
        "primary_target": finding.primary_target,
        "related_targets": finding.related_targets,
        "message": finding.message,
        "suggested_next_action": finding.suggested_next_action,
        "proposal_refs": finding.proposal_refs,
        "proposal_links": finding.proposal_links,
        "waiver_refs": finding.waiver_refs,
        "deviation_refs": finding.deviation_refs
    })
}
