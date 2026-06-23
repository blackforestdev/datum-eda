use std::collections::BTreeMap;

use anyhow::Result;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{ArtifactMetadata, ArtifactValidationState, ZoneFill, ZoneFillState};
use uuid::Uuid;

use super::command_project_check_finding_identity::check_finding_fingerprint;
use super::command_project_check_run_view::{
    NativeProjectCheckFindingView, check_finding_explanation, check_finding_message,
    check_finding_suggested_next_action, check_primary_target, finding_domain,
};

pub(crate) fn append_artifact_finding_values(
    project_id: &Uuid,
    model_revision: &str,
    artifact_metadata: &BTreeMap<Uuid, ArtifactMetadata>,
    findings: &mut Vec<NativeProjectCheckFindingView>,
) -> Result<()> {
    for metadata in artifact_metadata.values() {
        if metadata.validation_state != ArtifactValidationState::Invalid {
            continue;
        }
        let index = findings.len();
        let payload = serde_json::json!({
            "artifact_id": metadata.artifact_id,
            "kind": metadata.kind,
            "model_revision": metadata.model_revision,
            "validation_state": metadata.validation_state,
            "files": metadata.files,
            "production_projections": metadata.production_projections,
        });
        let material = format!(
            "datum-eda:check-finding:{model_revision}:artifact:{}:{}",
            metadata.artifact_id,
            to_json_deterministic(&payload)?
        );
        let finding_id = Uuid::new_v5(project_id, material.as_bytes());
        let code = "artifact_validation_invalid";
        let message = check_finding_message(code, &payload);
        findings.push(NativeProjectCheckFindingView {
            finding_id,
            index,
            source: "artifact",
            code: code.to_string(),
            severity: "error".to_string(),
            domain: finding_domain("artifact", code),
            rule_id: code.to_string(),
            standards_basis: None,
            rule_revision: None,
            import_key: None,
            status: "active".to_string(),
            primary_target: check_primary_target("artifact", &payload),
            related_targets: Vec::new(),
            message: message.clone(),
            explanation: check_finding_explanation(code, &message, &payload),
            suggested_next_action: check_finding_suggested_next_action(code, &payload),
            evidence: vec![payload.clone()],
            fingerprint: check_finding_fingerprint(
                &finding_domain("artifact", code),
                code,
                None,
                None,
                None,
                &check_primary_target("artifact", &payload),
                &payload,
            )?,
            payload,
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            waiver_refs: Vec::new(),
            deviation_refs: Vec::new(),
        });
    }
    Ok(())
}

pub(crate) fn append_zone_fill_finding_values(
    project_id: &Uuid,
    model_revision: &str,
    zone_fills: &BTreeMap<Uuid, ZoneFill>,
    findings: &mut Vec<NativeProjectCheckFindingView>,
) -> Result<()> {
    for fill in zone_fills.values() {
        if fill.state == ZoneFillState::Filled {
            continue;
        }
        let index = findings.len();
        let payload = serde_json::json!({
            "zone_id": fill.zone_id,
            "state": fill.state,
            "source_zone_revision": fill.source_zone_revision,
            "model_revision": fill.model_revision,
            "island_count": fill.islands.len(),
            "provenance": fill.provenance,
        });
        let material = format!(
            "datum-eda:check-finding:{model_revision}:zone-fill:{}:{}",
            fill.zone_id,
            to_json_deterministic(&payload)?
        );
        let finding_id = Uuid::new_v5(project_id, material.as_bytes());
        let code = match fill.state {
            ZoneFillState::Unfilled => "zone_fill_unfilled",
            ZoneFillState::Stale => "zone_fill_stale",
            ZoneFillState::Unsupported => "zone_fill_unsupported",
            ZoneFillState::Filled => unreachable!("filled zone fills are renderable"),
        };
        let message = check_finding_message(code, &payload);
        findings.push(NativeProjectCheckFindingView {
            finding_id,
            index,
            source: "zone_fill",
            code: code.to_string(),
            severity: "error".to_string(),
            domain: finding_domain("zone_fill", code),
            rule_id: code.to_string(),
            standards_basis: Some("datum.zone_fill_honesty.current".to_string()),
            rule_revision: Some("v1".to_string()),
            import_key: None,
            status: "active".to_string(),
            primary_target: check_primary_target("zone_fill", &payload),
            related_targets: Vec::new(),
            message: message.clone(),
            explanation: check_finding_explanation(code, &message, &payload),
            suggested_next_action: check_finding_suggested_next_action(code, &payload),
            evidence: vec![payload.clone()],
            fingerprint: check_finding_fingerprint(
                &finding_domain("zone_fill", code),
                code,
                Some("datum.zone_fill_honesty.current"),
                Some("v1"),
                None,
                &check_primary_target("zone_fill", &payload),
                &payload,
            )?,
            payload,
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            waiver_refs: Vec::new(),
            deviation_refs: Vec::new(),
        });
    }
    Ok(())
}
