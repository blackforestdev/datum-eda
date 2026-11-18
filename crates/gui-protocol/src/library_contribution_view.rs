//! Contribution facts in existing qualified Library Details, without approval.
use serde_json::Value;

use super::flatten_detail;

pub(in crate::library_browser) fn overview(record: &Value) -> Vec<String> {
    let report = &record["contribution"];
    if !record["revision"].is_object() || report["subject"] != record["revision"] {
        return vec!["Contribution evidence: unavailable for this exact revision".into()];
    }
    if report["inspection_status"] != "available" || !report["closure"]["assets"].is_array() {
        let mut lines = vec!["Contribution evidence: unavailable for this exact revision".into()];
        flatten_detail(
            "Inspection reason",
            &report["reason"],
            &mut lines,
            "unknown",
        );
        lines.extend([
            "Redistribution: not verified".into(),
            "Engineering approval: not established".into(),
            "Publication policy: unavailable".into(),
        ]);
        return lines;
    }
    let assets = report["closure"]["assets"].as_array().unwrap();
    let (
        mut passed,
        mut failed,
        mut needs_review,
        mut stale,
        mut missing,
        mut unknown,
        mut licenses,
    ) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    for asset in assets {
        licenses += usize::from(
            asset["provenance"]["rights"]["license_expression"]
                .as_str()
                .is_some_and(|value| !value.trim().is_empty()),
        );
        if asset["review_presence"] == "missing" {
            missing += 1;
        }
        let Some(reviews) = asset["reviews"].as_array() else {
            unknown += 1;
            continue;
        };
        for review in reviews {
            match (
                review["basis_status"].as_str(),
                review["record"]["result"].as_str(),
            ) {
                (Some("current"), Some("passed")) => passed += 1,
                (Some("current"), Some("failed")) => failed += 1,
                (Some("current"), Some("needs_review")) => needs_review += 1,
                (Some("stale"), _) => stale += 1,
                _ => unknown += 1,
            }
        }
    }
    let mut lines = vec![
        format!(
            "Dependency graph: {}",
            match report["closure"]["graph_valid"].as_bool() {
                Some(true) => "valid within supported checks",
                Some(false) => "invalid; inspect diagnostics",
                None => "unavailable",
            }
        ),
        format!("Closure members: {} exact assets", assets.len()),
        format!("Passed reviews: {passed} current"),
        format!("Failed reviews: {failed} current"),
        format!("Needs review: {needs_review} current"),
        format!("Stale reviews: {stale}"),
        format!("Missing reviews: {missing} assets"),
        format!("Declared licenses: {licenses} of {} assets", assets.len()),
        "Redistribution: not verified".into(),
        "Engineering approval: not established".into(),
        "Publication policy: unavailable".into(),
    ];
    if unknown > 0 {
        lines.push(format!("Unknown review facts: {unknown}"));
    }
    lines
}

pub(in crate::library_browser) fn lines(record: &Value) -> Vec<String> {
    let report = &record["contribution"];
    let mut lines = vec!["Contribution evidence: inspection only".into()];
    if !record["revision"].is_object() || report["subject"] != record["revision"] {
        lines.push("Contribution evidence: unavailable for this exact revision".into());
        return lines;
    }
    lines.push("Rights are recorded declarations; redistribution authority is not verified".into());
    lines.push("Review results and basis currency do not establish engineering approval".into());
    lines.push("Publication policy: unavailable; no publishing authority established".into());
    if report["inspection_status"] == "unavailable" {
        lines.push("Contribution evidence: unavailable for this exact revision".into());
        flatten_detail(
            "Contribution inspection reason",
            &report["reason"],
            &mut lines,
            "unknown",
        );
        return lines;
    }
    if report["inspection_status"] != "available" || !report["closure"]["assets"].is_array() {
        lines.push("Contribution evidence: unavailable for this exact revision".into());
        return lines;
    }
    flatten_detail(
        "Inspected subject",
        &report["subject"],
        &mut lines,
        "unknown",
    );
    flatten_detail(
        "Dependency closure digest",
        &report["closure"]["digest"],
        &mut lines,
        "unavailable",
    );
    lines.push(format!(
        "Dependency graph validation: {}",
        match report["closure"]["graph_valid"].as_bool() {
            Some(true) => "valid within supported checks",
            Some(false) => "invalid; inspect diagnostics",
            None => "unavailable",
        }
    ));
    flatten_detail(
        "Dependency diagnostics",
        &report["closure"]["diagnostics"],
        &mut lines,
        "unavailable",
    );
    for (index, asset) in report["closure"]["assets"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let member = format!("Dependency member {}", index + 1);
        flatten_detail(
            &format!("{member} revision"),
            &asset["revision"],
            &mut lines,
            "unknown",
        );
        flatten_detail(
            &format!("{member} kind"),
            &asset["kind"],
            &mut lines,
            "unknown",
        );
        for (label, field) in [
            ("payload digest", "payload_ref"),
            ("provenance digest", "provenance_ref"),
            ("exact dependencies", "dependencies"),
        ] {
            flatten_detail(
                &format!("{member} {label}"),
                &asset[field],
                &mut lines,
                "unknown",
            );
        }
        flatten_detail(
            &format!("{member} provenance / declared rights"),
            &asset["provenance"],
            &mut lines,
            "unknown",
        );
        lines.push(format!(
            "{member} review evidence: {}",
            match asset["review_presence"].as_str() {
                Some("missing") => "not recorded",
                Some("recorded") => "recorded; inspect result and basis separately",
                _ => "unavailable",
            }
        ));
        if let Some(reviews) = asset["reviews"].as_array() {
            for (review_index, review) in reviews.iter().enumerate() {
                let label = format!("{member} review {}", review_index + 1);
                lines.push(format!(
                    "{label} evidence basis: {}",
                    match review["basis_status"].as_str() {
                        Some("current") => "current for the exact subject and dependency closure",
                        Some("stale") => "stale for the inspected subject or dependency closure",
                        _ => "unavailable",
                    }
                ));
                flatten_detail(
                    &format!("{label} identity"),
                    &review["record_ref"],
                    &mut lines,
                    "unknown",
                );
                flatten_detail(
                    &format!("{label} record"),
                    &review["record"],
                    &mut lines,
                    "unknown",
                );
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn record() -> Value {
        let subject = json!({"asset_id":"part-id","revision_id":"exact-part-revision"});
        json!({"revision":subject,"contribution":{"subject":subject,"inspection_status":"available","closure":{
            "digest":"exact-closure-digest","graph_valid":true,"diagnostics":[],"assets":[
                {"revision":subject,"kind":"part","provenance":{"authors":[],"rights":{
                    "license_expression":"CC0-1.0","evidence":[]}},"review_presence":"recorded",
                    "reviews":[{"record_ref":"current-failed-record","basis_status":"current",
                        "record":{"result":"failed","reviewer":{"display_name":null}}},
                        {"record_ref":"stale-passed-record","basis_status":"stale",
                            "record":{"result":"passed","closure_digest":"old-closure"}}]},
                {"revision":{"asset_id":"pad-id","revision_id":"exact-pad-revision"},
                    "kind":"padstack","provenance":{"rights":{"license_expression":null}},
                    "review_presence":"missing","reviews":[]}]},
            "redistribution_authority":"not_verified","engineering_approval":"not_evaluated",
            "publication_policy":"unavailable"}})
    }

    #[test]
    fn contribution_details_preserve_failed_current_and_stale_passed_evidence_without_authority() {
        let details = lines(&record()).join("\n");
        for fact in [
            "exact-part-revision",
            "exact-pad-revision",
            "exact-closure-digest",
            "license_expression: CC0-1.0",
            "license_expression: unknown",
            "review 1 evidence basis: current",
            "review 1 record.result: failed",
            "review 2 evidence basis: stale",
            "review 2 record.result: passed",
            "review 1 record.reviewer.display_name: unknown",
            "review evidence: not recorded",
            "redistribution authority is not verified",
            "do not establish engineering approval",
            "Publication policy: unavailable",
        ] {
            assert!(details.contains(fact), "missing honest fact: {fact}");
        }
    }

    #[test]
    fn contribution_details_refuse_mismatched_or_missing_subject_instead_of_reusing_evidence() {
        let mut input = record();
        input["contribution"]["subject"]["revision_id"] = json!("other-revision");
        let details = lines(&input).join("\n");
        assert!(details.contains("unavailable for this exact revision"));
        assert!(!details.contains("CC0-1.0"));
        input.as_object_mut().unwrap().remove("contribution");
        assert!(
            lines(&input)
                .join("\n")
                .contains("unavailable for this exact revision")
        );
    }

    #[test]
    fn contribution_details_keep_unavailable_history_reason_separate_from_current_graph_checks() {
        let mut input = record();
        input["contribution"] = json!({"subject":input["revision"],
            "inspection_status":"unavailable","reason":"historical dependency slot differs",
            "redistribution_authority":"not_verified","engineering_approval":"not_evaluated",
            "publication_policy":"unavailable"});
        let details = lines(&input).join("\n");
        assert!(details.contains("unavailable for this exact revision"));
        assert!(details.contains("historical dependency slot differs"));
        assert!(details.contains("Publication policy: unavailable"));
        assert!(details.contains("redistribution authority is not verified"));
        assert!(details.contains("do not establish engineering approval"));
        assert!(!details.contains("Dependency graph validation:"));
        assert!(!details.contains("evidence basis: current"));
    }

    #[test]
    fn contribution_overview_exposes_failed_stale_missing_and_unknown_authority_before_raw_records()
    {
        let input = record();
        let summary = overview(&input);
        assert!(summary[0].contains("Dependency graph: valid within supported checks"));
        let text = summary.join("\n");
        for expected in [
            "Closure members: 2 exact assets",
            "Passed reviews: 0 current",
            "Failed reviews: 1 current",
            "Stale reviews: 1",
            "Missing reviews: 1 assets",
            "Declared licenses: 1 of 2 assets",
            "Redistribution: not verified",
            "Engineering approval: not established",
            "Publication policy: unavailable",
        ] {
            assert!(text.contains(expected), "missing summary fact: {expected}");
        }
        assert!(!text.contains("record.result"));
        let full = lines(&input).join("\n");
        assert!(full.contains("current-failed-record"));
        assert!(full.contains("stale-passed-record"));
        let mut changed = input;
        changed["contribution"]["subject"]["revision_id"] = json!("other");
        let text = overview(&changed).join("\n");
        assert!(text.contains("unavailable for this exact revision"));
        assert!(!text.contains("valid within supported checks"));
        assert!(!text.contains("Declared licenses"));
    }
}
