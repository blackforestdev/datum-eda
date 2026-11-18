//! Designer-facing text for the service's exact prepared revision review.
use serde_json::Value;

pub(super) fn lines(summary: &Value) -> Vec<String> {
    render_lines(summary, true)
}

pub(super) fn overview(summary: &Value) -> Vec<String> {
    render_lines(summary, false)
}

fn render_lines(summary: &Value, records: bool) -> Vec<String> {
    let mut lines = Vec::new();
    if summary["action"] == "adopt_profile" {
        lines.push(format!(
            "Project library format version: {}",
            summary
                .get("project_schema_version")
                .map(text)
                .unwrap_or_else(|| "unknown".into())
        ));
        lines.push(match summary["library_lock_baseline"].as_str() {
            Some("empty") => {
                "Initial managed library: empty; existing native definitions are retained.".into()
            }
            Some(other) => format!("Initial managed library: {other}"),
            None => "Initial managed library: unknown".into(),
        });
        lines.push(
            match summary["undo_to_previous_format"].as_bool() {
                Some(false) => {
                    "Format change is forward-only; Undo cannot restore the previous format."
                }
                Some(true) => "Service reports that Undo can restore the previous format.",
                None => "Undo to the previous format: unknown.",
            }
            .into(),
        );
        lines.push(match summary["legacy_common_metadata_coverage"].as_str() {
            Some("not_inferred") => {
                "Historical authorship, approval and rights are not inferred.".into()
            }
            Some(other) => format!("Historical metadata coverage: {other}"),
            None => "Historical metadata coverage: unknown.".into(),
        });
        return lines;
    }
    if matches!(
        summary["action"].as_str(),
        Some("adopt" | "adopt_selection")
    ) {
        lines.push("Review affected PCB placements and pad changes before applying. Unsupported physical changes are refused.".into());
    }
    if let Some(updates) = summary["physical_geometry_updates"].as_array() {
        if updates.is_empty() {
            lines.push("PCB geometry updates: none.".into());
        } else {
            lines.push("PCB pad geometry: positions and lengths in nm; corner ratios in ppm; layers are identifiers.".into());
            for user in updates {
                lines.push(format!(
                    "{}: PCB placement {}; component {}",
                    text(&user["reference"]),
                    text(&user["placed_package"]),
                    text(&user["component_instance"])
                ));
                if let Some(pads) = user["pads"].as_array() {
                    for pad in pads {
                        lines.push(format!(
                            "Pad {}: placed {}; library {}",
                            text(&pad["after"]["name"]),
                            text(&pad["placed_pad"]),
                            text(&pad["library_pad"])
                        ));
                        if pad["before"] == pad["after"] {
                            lines.push(
                                "Pad geometry unchanged; source binding will be refreshed.".into(),
                            );
                        } else {
                            native_values("Pad", pad.get("before"), pad.get("after"), &mut lines);
                        }
                    }
                }
            }
        }
    }
    let Some(assets) = summary.get("asset_changes").and_then(Value::as_array) else {
        super::revision_view::flatten_detail("Preview", summary, &mut lines, "unknown");
        return lines;
    };
    for asset in assets {
        let review = &asset["review"];
        lines.push(format!(
            "{} ({})",
            asset["name"].as_str().unwrap_or("Library definition"),
            text(&asset["asset_id"])
        ));
        lines.push(format!(
            "Exact managed revision: {} → {}",
            reference(&review["before"]),
            reference(&review["after"])
        ));
        if summary["action"] == "save_candidate"
            && let Some(candidate) = asset.get("candidate_contribution")
        {
            lines.push("Proposed Part contribution evidence".into());
            if candidate["kind"] == "part"
                && candidate["revision"].is_object()
                && candidate["revision"] == review["after"]
                && candidate["revision"]["asset_id"] == asset["asset_id"]
            {
                use super::revision_view::contribution;
                lines.extend(if records {
                    contribution::lines(candidate)
                } else {
                    contribution::overview(candidate)
                });
            } else {
                lines
                    .push("Contribution evidence: unavailable for this exact proposed Part".into());
            }
        }
        if review["comparison"] == "first_managed_record" {
            lines.push(
                "First managed record: no prior managed revision is available for comparison."
                    .into(),
            );
            lines.push(match asset["native_baseline"]["exists"].as_bool() {
                Some(true) => "This definition already exists natively; the fields below record its managed content.",
                Some(false) => "No existing native definition was found in the captured project.",
                None => "Native baseline unavailable; absence is not inferred.",
            }.into());
        }
        lines.push(format!("Payload bytes changed: {}; engineering values changed: {}; dependency or engineering basis changed: {}",
            yes(&review["payload_bytes_changed"]), yes(&review["payload_semantics_changed"]),
            yes(&review["engineering_basis_changed"])));
        for (key, heading) in [
            ("fields", "Engineering content"),
            ("provenance", "Recorded provenance claims"),
            ("revision_metadata", "Revision record metadata"),
        ] {
            if key == "revision_metadata" && !records {
                continue;
            }
            if let Some(fields) = review[key].as_array().filter(|v| !v.is_empty()) {
                lines.push(heading.into());
                for field in fields {
                    if key == "provenance"
                        && !records
                        && !has_claim(&field["before"])
                        && !has_claim(&field["after"])
                    {
                        continue;
                    }
                    let unit = match field["unit"].as_str() {
                        Some("nanometres") => " nm",
                        Some("parts_per_million") => " ppm",
                        _ => "",
                    };
                    lines.push(format!(
                        "{}: {}{} → {}{}",
                        field["label"].as_str().unwrap_or("Authored field"),
                        field_value(&field["before"]),
                        unit,
                        field_value(&field["after"]),
                        unit
                    ));
                }
                if key == "provenance" && !records {
                    lines.push("Empty or absent provenance fields remain available in the full recorded details; no authorship, approval or rights are inferred.".into());
                }
            }
        }
        if let Some(deps) = review["dependencies"].as_array().filter(|v| !v.is_empty()) {
            lines.push("Exact dependency changes".into());
            for dep in deps {
                lines.push(format!(
                    "{}: {} → {}",
                    text(&dep["role"]),
                    reference(&dep["before"]),
                    reference(&dep["after"])
                ));
            }
        }
        lines.push("Engineering checks: not evaluated by this comparison. Recorded claims do not establish approval or publication rights.".into());
    }
    if assets.is_empty() {
        lines.push("No managed asset revision changes in this prepared action.".into());
    }
    for (key, label) in [
        ("affected_components", "Affected placed component"),
        ("changed_materializations", "Changed native definition"),
        ("selected_roots", "Included exact root"),
    ] {
        if let Some(values) = summary[key].as_array() {
            if values.is_empty() {
                lines.push(format!("{label}s: none"));
            }
            for value in values {
                lines.push(format!(
                    "{label}: {}",
                    if value.is_object() {
                        reference(value)
                    } else {
                        text(value)
                    }
                ));
            }
        }
    }
    if let Some(changes) = summary["native_materialization_changes"].as_array() {
        for change in changes {
            lines.push(format!(
                "Native {} {}",
                text(&change["asset_id"]),
                if change["created"] == true {
                    "will be created"
                } else {
                    "will be updated"
                }
            ));
            native_values(
                "Native",
                change.get("before").filter(|v| !v.is_null()),
                change.get("after"),
                &mut lines,
            );
        }
    }
    if summary["active_lock_unchanged"] == true {
        lines.push("Active project definitions and placed bindings remain unchanged by this candidate save.".into());
    }
    lines
}

fn has_claim(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(text) => !matches!(text.as_str(), "" | "null" | "[]" | "{}" | "\"\""),
        _ => true,
    }
}

fn field_value(value: &Value) -> String {
    // Option<String>: JSON null means absent; string "null" is an authored null.
    match value {
        Value::Null => "[absent]".into(),
        Value::String(text) => text.clone(),
        other => format!("[unexpected review value: {other}]"),
    }
}

fn native_values(
    path: &str,
    before: Option<&Value>,
    after: Option<&Value>,
    lines: &mut Vec<String>,
) {
    if before == after {
        return;
    }
    if let (Some(Value::Object(before)), Some(Value::Object(after))) = (before, after) {
        let keys: std::collections::BTreeSet<_> = before.keys().chain(after.keys()).collect();
        for key in keys {
            native_values(
                &format!("{path}.{key}"),
                before.get(key),
                after.get(key),
                lines,
            );
        }
    } else {
        let display = |v: Option<&Value>| v.map(text).unwrap_or_else(|| "[absent]".into());
        lines.push(format!("{path}: {} → {}", display(before), display(after)));
    }
}

fn yes(value: &Value) -> &'static str {
    match value.as_bool() {
        Some(true) => "yes",
        Some(false) => "no",
        None => "unknown",
    }
}

fn text(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
}

fn reference(value: &Value) -> String {
    if value.is_null() {
        "[no managed revision]".into()
    } else {
        format!(
            "{} / {}",
            text(&value["asset_id"]),
            text(&value["revision_id"])
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn candidate_evidence_is_visible_before_save_only_for_the_exact_proposed_part() {
        let reference = json!({"asset_id":"part", "revision_id":"proposed-exact-revision"});
        let mut summary = json!({"action":"save_candidate", "active_lock_unchanged":true,
            "asset_changes":[{"asset_id":"part", "name":"Part", "review":{
                "before":{"asset_id":"part","revision_id":"active-revision"},
                "after":reference}, "candidate_contribution":{"kind":"part",
                "revision":reference, "contribution":{"subject":reference,
                    "inspection_status":"available", "closure":{"graph_valid":true,
                    "digest":"proposed-closure", "assets":[{"revision":reference,
                        "review_presence":"recorded", "provenance":{"rights":{}},
                        "reviews":[{"record_ref":"current-failed-evidence", "basis_status":"current",
                            "record":{"result":"failed"}},
                            {"record_ref":"stale-passed-evidence", "basis_status":"stale",
                            "record":{"result":"passed"}}]}]}}}}]});
        let compact = overview(&summary).join("\n");
        for fact in [
            "Proposed Part contribution evidence",
            "Failed reviews: 1 current",
            "Stale reviews: 1",
            "Redistribution: not verified",
            "Publication policy: unavailable",
        ] {
            assert!(compact.contains(fact), "{fact}");
        }
        let full = lines(&summary).join("\n");
        for fact in [
            "proposed-closure",
            "current-failed-evidence",
            "stale-passed-evidence",
            "Review results and basis currency do not establish engineering approval",
            "placed bindings remain unchanged",
        ] {
            assert!(full.contains(fact), "{fact}");
        }
        // A response qualified for another proposed revision must not donate
        // rights or review facts to the current visible preview.
        summary["asset_changes"][0]["review"]["after"]["revision_id"] = json!("other-proposal");
        for rendered in [overview(&summary), lines(&summary)] {
            let text = rendered.join("\n");
            assert!(text.contains("unavailable for this exact proposed Part"));
            assert!(
                !text.contains("proposed-closure") && !text.contains("current-failed-evidence")
            );
        }
        summary["action"] = json!("adopt_selection");
        assert!(
            !lines(&summary)
                .join("\n")
                .contains("Proposed Part contribution evidence")
        );
    }

    #[test]
    fn physical_update_overview_keeps_every_user_and_exact_pad_geometry() {
        let pad = json!({"placed_pad":"placed-pad","library_pad":"library-pad",
            "before":{"name":"1","position":{"x":9007199254740992_i64,"y":0},
                "aperture":{"circle":{"diameter_nm":1000000}}},
            "after":{"name":"1","position":{"x":9007199254740993_i64,"y":0},
                "aperture":{"circle":{"diameter_nm":1200000}}}});
        let summary = json!({"action":"adopt_selection","asset_changes":[],
            "physical_geometry_updates":[
                {"reference":"R1","placed_package":"pcb-one","component_instance":"ci-one","pads":[pad]},
                {"reference":"R2","placed_package":"pcb-two","component_instance":"ci-two","pads":[pad]}]});
        for rendered in [overview(&summary), lines(&summary)] {
            for required in [
                "R1: PCB placement pcb-one; component ci-one",
                "R2: PCB placement pcb-two; component ci-two",
                "placed-pad",
                "library-pad",
                "Pad.position.x: 9007199254740992 → 9007199254740993",
                "Pad.aperture.circle.diameter_nm: 1000000 → 1200000",
                "positions and lengths in nm",
                "Unsupported physical changes are refused",
            ] {
                assert!(
                    rendered.iter().any(|line| line.contains(required)),
                    "{required}"
                );
            }
            assert!(
                !rendered
                    .iter()
                    .any(|line| line.contains("retain their saved geometry"))
            );
        }
    }

    #[test]
    fn overview_keeps_engineering_and_claim_removals_and_details_restore_empty_records() {
        let summary = json!({"action":"adopt_selection", "asset_changes":[{
            "asset_id":"symbol", "name":"Shared resistor", "review":{
                "before":{"asset_id":"symbol","revision_id":"old"},
                "after":{"asset_id":"symbol","revision_id":"chosen"},
                "fields":[{"label":"Geometry width", "before":"9007199254740992",
                    "after":"9007199254740993", "unit":"nanometres"},
                    {"label":"Unclassified authored value","before":null,"after":"null"}],
                "provenance":[{"label":"License claim","before":"\"Recorded claim\"","after":null},
                    {"label":"Authors","before":null,"after":"[]"}],
                "revision_metadata":[{"label":"Recorded payload","before":"old-payload","after":"exact-payload"}],
                "dependencies":[{"role":"unit","before":{"asset_id":"unit","revision_id":"old-unit"},
                    "after":{"asset_id":"unit","revision_id":"chosen-unit"}}]
            }}], "affected_components":["placed-one","placed-two"],
            "native_materialization_changes":[{"asset_id":"symbol","created":false,
                "before":{"unknown":1},"after":{"unknown":2}}]});
        let compact = overview(&summary);
        let full = lines(&summary);
        for required in [
            "9007199254740993 nm",
            "Unclassified authored value: [absent] → null",
            "License claim: \"Recorded claim\" → [absent]",
            "chosen-unit",
            "placed-one",
            "placed-two",
            "Native.unknown: 1 → 2",
            "not evaluated",
            "no authorship, approval or rights are inferred",
        ] {
            assert!(
                compact.iter().any(|line| line.contains(required)),
                "{required}"
            );
        }
        assert!(
            !compact
                .iter()
                .any(|line| line.contains("exact-payload") || line.starts_with("Authors:"))
        );
        assert!(full.iter().any(|line| line.contains("exact-payload")));
        assert!(full.iter().any(|line| line == "Authors: [absent] → []"));
    }

    #[test]
    fn exact_authored_values_absence_native_baseline_and_check_limits_remain_readable() {
        let result = lines(
            &json!({"action":"save_candidate", "active_lock_unchanged":true,
            "asset_changes":[{"asset_id":"asset", "name":"Resistor",
                "native_baseline":{"exists":true}, "review":{
                    "comparison":"first_managed_record", "before":null,
                    "after":{"asset_id":"asset","revision_id":"exact-revision"},
                    "payload_bytes_changed":true,"payload_semantics_changed":true,
                    "engineering_basis_changed":true,
                    "fields":[{"label":"Width", "before":"9007199254740992",
                        "after":"9007199254740993", "unit":"nanometres"},
                        {"label":"Unknown authored field", "before":null,"after":"null"}],
                    "provenance":[{"label":"Approval claim","before":null,"after":"\"unverified\""}],
                    "dependencies":[{"role":"physical map","before":null,
                        "after":{"asset_id":"map","revision_id":"exact-map"}}],
                    "checks":"not_evaluated"}}],"native_materialization_changes":[]}),
        );
        assert!(
            result
                .iter()
                .any(|s| s == "Width: 9007199254740992 nm → 9007199254740993 nm")
        );
        assert!(
            result
                .iter()
                .any(|s| s == "Unknown authored field: [absent] → null")
        );
        assert!(result.iter().any(|s| s.contains("already exists natively")));
        assert!(
            result
                .iter()
                .any(|s| s.contains("physical map") && s.contains("exact-map"))
        );
        assert!(
            result
                .iter()
                .any(|s| s.contains("not evaluated") && s.contains("approval"))
        );
        assert!(
            result
                .iter()
                .any(|s| s.contains("bindings remain unchanged"))
        );
        let unknown = lines(&json!({"asset_changes":[{"review":{
            "comparison":"first_managed_record"}}]}));
        assert!(
            unknown
                .iter()
                .any(|s| s == "Native baseline unavailable; absence is not inferred.")
        );
        assert!(
            !unknown
                .iter()
                .any(|s| s.contains("No existing native definition"))
        );
    }
}
