use serde_json::{Value, json};

pub(super) fn daemon_erc_check_run_view(findings: &[eda_engine::erc::ErcFinding]) -> Value {
    let normalized = findings
        .iter()
        .enumerate()
        .map(|(index, finding)| daemon_erc_finding_view(index, finding))
        .collect::<Vec<_>>();
    let summary = daemon_check_summary(&normalized);
    json!({
        "contract": "check_run_v1",
        "persisted": false,
        "check_run_id": deterministic_daemon_uuid("check-run:erc", &serde_json::to_value(findings).unwrap_or(Value::Null)).to_string(),
        "project_id": "engine-daemon-session",
        "model_revision": "engine-daemon-live",
        "profile_id": "erc",
        "status": summary["status"],
        "summary": summary,
        "finding_count": normalized.len(),
        "findings": normalized,
        "proposal_refs": [],
        "proposal_links": [],
        "profile_basis": {
            "profile_id": "erc",
            "standards_basis": [],
            "inputs": [],
        },
        "coverage": [{
            "domain": "erc",
            "rule_id": "schematic_connectivity",
            "status": "evaluated",
            "message": "Engine-daemon live ERC precheck evaluation."
        }],
        "raw_report": {
            "erc": findings,
        },
    })
}

pub(super) fn daemon_drc_check_run_view(report: &eda_engine::drc::DrcReport) -> Value {
    let normalized = report
        .violations
        .iter()
        .enumerate()
        .map(|(index, violation)| daemon_drc_finding_view(index, violation))
        .collect::<Vec<_>>();
    let summary = daemon_check_summary(&normalized);
    json!({
        "contract": "check_run_v1",
        "persisted": false,
        "check_run_id": deterministic_daemon_uuid("check-run:drc", &serde_json::to_value(report).unwrap_or(Value::Null)).to_string(),
        "project_id": "engine-daemon-session",
        "model_revision": "engine-daemon-live",
        "profile_id": "drc",
        "status": summary["status"],
        "summary": summary,
        "finding_count": normalized.len(),
        "findings": normalized,
        "proposal_refs": [],
        "proposal_links": [],
        "profile_basis": {
            "profile_id": "drc",
            "standards_basis": [],
            "inputs": [],
        },
        "coverage": [{
            "domain": "drc",
            "rule_id": "board_design_rules",
            "status": "evaluated",
            "message": "Engine-daemon live DRC evaluation."
        }],
        "raw_report": {
            "drc": report,
        },
    })
}

pub(super) fn explain_erc_finding_by_fingerprint(
    findings: &[eda_engine::erc::ErcFinding],
    fingerprint: &str,
) -> Option<Value> {
    explain_finding_by_fingerprint(&daemon_erc_check_run_view(findings), fingerprint)
}

pub(super) fn explain_drc_finding_by_fingerprint(
    report: &eda_engine::drc::DrcReport,
    fingerprint: &str,
) -> Option<Value> {
    explain_finding_by_fingerprint(&daemon_drc_check_run_view(report), fingerprint)
}

fn explain_finding_by_fingerprint(check_run: &Value, fingerprint: &str) -> Option<Value> {
    let finding = check_run
        .get("findings")
        .and_then(Value::as_array)?
        .iter()
        .find(|finding| finding.get("fingerprint").and_then(Value::as_str) == Some(fingerprint))?;
    let objects_involved = std::iter::once(finding.get("primary_target"))
        .chain(
            finding
                .get("related_targets")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(Some),
        )
        .flatten()
        .filter_map(target_to_object_info)
        .collect::<Vec<_>>();
    Some(json!({
        "explanation": finding
            .get("explanation")
            .or_else(|| finding.get("message"))
            .cloned()
            .unwrap_or_else(|| json!("")),
        "rule_detail": format!(
            "{} {} ({})",
            finding.get("domain").and_then(Value::as_str).unwrap_or("check"),
            finding.get("rule_id").and_then(Value::as_str).unwrap_or("unknown"),
            finding.get("severity").and_then(Value::as_str).unwrap_or("info")
        ),
        "objects_involved": objects_involved,
        "suggestion": finding
            .get("suggested_next_action")
            .cloned()
            .filter(|value| !value.is_null())
            .unwrap_or_else(|| json!("Inspect the referenced check finding and apply an explicit repair, waiver, or accepted deviation.")),
        "fingerprint": fingerprint,
    }))
}

fn target_to_object_info(target: &Value) -> Option<Value> {
    let kind = target.get("kind").and_then(Value::as_str)?;
    let id = target.get("id")?;
    Some(json!({
        "type": kind,
        "uuid": id,
        "description": id.as_str().map(ToOwned::to_owned).unwrap_or_else(|| id.to_string()),
    }))
}

fn daemon_erc_finding_view(index: usize, finding: &eda_engine::erc::ErcFinding) -> Value {
    let payload = serde_json::to_value(finding).unwrap_or(Value::Null);
    let primary_target = daemon_primary_target(&payload, "erc");
    let status = if finding.waived { "waived" } else { "active" };
    json!({
        "finding_id": deterministic_daemon_uuid(&format!("erc:{}:{index}", finding.code), &payload).to_string(),
        "index": index,
        "source": "erc",
        "code": finding.code,
        "severity": daemon_erc_severity(&finding.severity),
        "fingerprint": daemon_fingerprint("erc", finding.code, &primary_target, &payload),
        "domain": "erc",
        "rule_id": finding.code,
        "standards_basis": null,
        "rule_revision": null,
        "import_key": null,
        "status": status,
        "primary_target": primary_target,
        "related_targets": daemon_related_targets(&payload, &primary_target),
        "message": finding.message,
        "explanation": finding.message,
        "suggested_next_action": null,
        "evidence": [payload.clone()],
        "payload": payload,
        "proposal_refs": [],
        "proposal_links": [],
        "waiver_refs": [],
        "deviation_refs": [],
    })
}

fn daemon_drc_finding_view(index: usize, violation: &eda_engine::drc::DrcViolation) -> Value {
    let payload = serde_json::to_value(violation).unwrap_or(Value::Null);
    let primary_target = daemon_primary_target(&payload, "drc");
    let status = if violation.waived { "waived" } else { "active" };
    json!({
        "finding_id": deterministic_daemon_uuid(&format!("drc:{}:{index}", violation.code), &payload).to_string(),
        "index": index,
        "source": "drc",
        "code": violation.code,
        "severity": daemon_drc_severity(&violation.severity),
        "fingerprint": violation
            .fingerprint
            .clone()
            .unwrap_or_else(|| daemon_fingerprint("drc", &violation.code, &primary_target, &payload)),
        "domain": "drc",
        "rule_id": violation.code,
        "standards_basis": violation.standards_basis,
        "rule_revision": violation.rule_revision,
        "import_key": violation.import_key,
        "status": status,
        "primary_target": primary_target,
        "related_targets": daemon_related_targets(&payload, &primary_target),
        "message": violation.message,
        "explanation": violation.message,
        "suggested_next_action": null,
        "evidence": [payload.clone()],
        "payload": payload,
        "proposal_refs": [],
        "proposal_links": [],
        "waiver_refs": [],
        "deviation_refs": [],
    })
}

fn daemon_check_summary(findings: &[Value]) -> Value {
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut infos = 0usize;
    let mut waived = 0usize;
    let mut by_code = std::collections::BTreeMap::<String, usize>::new();
    for finding in findings {
        let code = finding
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        *by_code.entry(code).or_insert(0) += 1;
        if finding.get("status").and_then(Value::as_str) == Some("waived") {
            waived += 1;
            continue;
        }
        match finding.get("severity").and_then(Value::as_str) {
            Some("error") => errors += 1,
            Some("warning") => warnings += 1,
            _ => infos += 1,
        }
    }
    let status = if errors > 0 {
        "error"
    } else if warnings > 0 {
        "warning"
    } else if infos > 0 {
        "info"
    } else {
        "ok"
    };
    json!({
        "status": status,
        "errors": errors,
        "warnings": warnings,
        "infos": infos,
        "waived": waived,
        "by_code": by_code
            .into_iter()
            .map(|(code, count)| json!({"code": code, "count": count}))
            .collect::<Vec<_>>(),
    })
}

fn daemon_primary_target(payload: &Value, fallback_kind: &str) -> Value {
    if let Some(id) = daemon_object_id_values(payload).next() {
        return json!({"kind": "object_uuid", "id": id});
    }
    for key in [
        "object_id",
        "object_uuid",
        "component_uuid",
        "symbol_uuid",
        "pin_uuid",
        "pad_uuid",
        "pad_id",
        "zone_id",
        "artifact_id",
        "net_uuid",
        "uuid",
        "id",
    ] {
        if let Some(value) = payload.get(key) {
            return json!({"kind": key, "id": value});
        }
    }
    json!({"kind": fallback_kind, "id": "unknown"})
}

fn daemon_related_targets(payload: &Value, primary_target: &Value) -> Vec<Value> {
    daemon_object_id_values(payload)
        .map(|id| json!({"kind": "object_uuid", "id": id}))
        .filter(|target| target != primary_target)
        .collect()
}

fn daemon_object_id_values(payload: &Value) -> impl Iterator<Item = String> + '_ {
    ["objects", "object_uuids"]
        .into_iter()
        .filter_map(|key| payload.get(key).and_then(Value::as_array))
        .flatten()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
}

fn daemon_erc_severity(severity: &eda_engine::erc::ErcSeverity) -> &'static str {
    match severity {
        eda_engine::erc::ErcSeverity::Error => "error",
        eda_engine::erc::ErcSeverity::Warning => "warning",
        eda_engine::erc::ErcSeverity::Info => "info",
    }
}

fn daemon_drc_severity(severity: &eda_engine::drc::DrcSeverity) -> &'static str {
    match severity {
        eda_engine::drc::DrcSeverity::Error => "error",
        eda_engine::drc::DrcSeverity::Warning => "warning",
    }
}

fn daemon_fingerprint(
    domain: &str,
    rule_id: &str,
    primary_target: &Value,
    payload: &Value,
) -> String {
    let material = json!({
        "domain": domain,
        "rule_id": rule_id,
        "primary_target": primary_target,
        "payload": payload,
    });
    let encoded = serde_json::to_vec(&material).unwrap_or_default();
    let digest = <sha2::Sha256 as sha2::Digest>::digest(&encoded);
    format!("sha256:{digest:x}")
}

fn deterministic_daemon_uuid(prefix: &str, payload: &Value) -> uuid::Uuid {
    let material = format!(
        "datum-eda:engine-daemon:{prefix}:{}",
        serde_json::to_string(payload).unwrap_or_default()
    );
    uuid::Uuid::new_v5(&uuid::Uuid::nil(), material.as_bytes())
}
