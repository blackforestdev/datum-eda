use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::BoardReviewSceneV1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CheckRunReviewState {
    #[serde(default)]
    pub check_run_id: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub model_revision: Option<String>,
    #[serde(default)]
    pub profile_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub persisted: bool,
    #[serde(default)]
    pub summary: Value,
    #[serde(default)]
    pub finding_count: usize,
    #[serde(default)]
    pub findings: Vec<CheckFindingSummary>,
    #[serde(default)]
    pub proposal_refs: Vec<String>,
    #[serde(default)]
    pub proposal_links: Vec<Value>,
    #[serde(default)]
    pub profile_basis: CheckRunProfileBasisSummary,
    #[serde(default)]
    pub coverage: Vec<CheckRunCoverageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CheckRunProfileBasisSummary {
    #[serde(default)]
    pub profile_id: String,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub standards_basis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CheckRunCoverageSummary {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub rule_id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub target_scope: String,
    #[serde(default)]
    pub basis_id: Option<String>,
    #[serde(default)]
    pub rule_revision: Option<String>,
    #[serde(default)]
    pub standards_basis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CheckFindingSummary {
    #[serde(default, alias = "id")]
    pub finding_id: Option<String>,
    #[serde(default)]
    pub index: usize,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub fingerprint: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub rule_id: String,
    #[serde(default)]
    pub standards_basis: Option<String>,
    #[serde(default)]
    pub rule_revision: Option<String>,
    #[serde(default)]
    pub import_key: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub primary_target: Value,
    #[serde(default)]
    pub related_targets: Vec<Value>,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub explanation: String,
    #[serde(default)]
    pub suggested_next_action: Option<String>,
    #[serde(default)]
    pub evidence: Vec<Value>,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub proposal_refs: Vec<String>,
    #[serde(default)]
    pub proposal_links: Vec<Value>,
    #[serde(default)]
    pub waiver_refs: Vec<String>,
    #[serde(default)]
    pub deviation_refs: Vec<String>,
}

impl CheckFindingSummary {
    pub fn target_label(&self) -> Option<String> {
        let target = std::iter::once(&self.primary_target)
            .chain(self.related_targets.iter())
            .find(|target| target_id(target).is_some())?;
        let id = target_id(target)?;
        let kind = target
            .get("kind")
            .and_then(Value::as_str)
            .filter(|kind| !kind.is_empty())
            .unwrap_or("target");
        Some(format!("{kind}:{id}"))
    }

    pub fn standards_basis_label(&self) -> Option<String> {
        self.standards_basis
            .clone()
            .or_else(|| {
                self.evidence.iter().find_map(|entry| {
                    (entry.get("evidence_kind").and_then(Value::as_str) == Some("standards_basis"))
                        .then(|| entry.get("basis_id").and_then(Value::as_str))
                        .flatten()
                        .map(str::to_string)
                })
            })
            .filter(|basis| !basis.is_empty())
    }
}

pub fn check_run_review_state_from_json(payload: &str) -> Result<CheckRunReviewState> {
    let envelope: CheckRunEnvelope =
        serde_json::from_str(payload).context("failed to decode check-run JSON")?;
    Ok(match envelope.contract.as_deref() {
        Some("check_run_record_v1") => envelope
            .check_run
            .map(|check_run| check_run.into_state(envelope.project_id, envelope.model_revision))
            .unwrap_or_default(),
        _ => CheckRunPayload {
            check_run_id: envelope.check_run_id,
            project_id: envelope.project_id,
            model_revision: envelope.model_revision,
            profile_id: envelope.profile_id,
            status: envelope.status,
            persisted: envelope.persisted,
            summary: envelope.summary,
            finding_count: envelope.finding_count,
            findings: envelope.findings,
            proposal_refs: envelope.proposal_refs,
            proposal_links: envelope.proposal_links,
            profile_basis: envelope.profile_basis,
            coverage: envelope.coverage,
        }
        .into_state(None, None),
    })
}

pub(crate) fn check_run_review_state_from_context_value(
    value: &Value,
) -> Option<CheckRunReviewState> {
    let context = value.get("check_context")?;
    let context: CheckContextSummary = serde_json::from_value(context.clone()).ok()?;
    if context.contract.as_deref() != Some("datum_check_context_v1") {
        return None;
    }
    context
        .visible_check_runs
        .into_iter()
        .next()
        .map(|run| run.into_state())
}

pub fn check_finding_scene_target_object_id(
    scene: &BoardReviewSceneV1,
    finding: &CheckFindingSummary,
) -> Option<String> {
    std::iter::once(&finding.primary_target)
        .chain(finding.related_targets.iter())
        .filter_map(|target| target_id(target))
        .find_map(|id| scene_object_id_for_target_id(scene, id))
}

fn target_id(target: &Value) -> Option<&str> {
    target
        .get("id")
        .or_else(|| target.get("object_id"))
        .or_else(|| target.get("object_uuid"))
        .or_else(|| target.get("uuid"))
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty() && *id != "unknown")
}

fn scene_object_id_for_target_id(scene: &BoardReviewSceneV1, id: &str) -> Option<String> {
    scene
        .components
        .iter()
        .find(|item| {
            item.object_id == id || item.component_uuid == id || item.source_object_uuid == id
        })
        .map(|item| item.object_id.clone())
        .or_else(|| {
            scene
                .pads
                .iter()
                .find(|item| {
                    item.object_id == id || item.pad_uuid == id || item.source_object_uuid == id
                })
                .map(|item| item.object_id.clone())
        })
        .or_else(|| {
            scene
                .tracks
                .iter()
                .find(|item| {
                    item.object_id == id || item.track_uuid == id || item.source_object_uuid == id
                })
                .map(|item| item.object_id.clone())
        })
        .or_else(|| {
            scene
                .vias
                .iter()
                .find(|item| {
                    item.object_id == id || item.via_uuid == id || item.source_object_uuid == id
                })
                .map(|item| item.object_id.clone())
        })
        .or_else(|| {
            scene
                .zones
                .iter()
                .find(|item| {
                    item.object_id == id || item.zone_uuid == id || item.source_object_uuid == id
                })
                .map(|item| item.object_id.clone())
        })
        .or_else(|| {
            scene
                .board_texts
                .iter()
                .find(|item| item.object_id == id || item.text_uuid == id)
                .map(|item| item.object_id.clone())
        })
        .or_else(|| {
            scene
                .board_graphics
                .iter()
                .find(|item| item.object_id == id || item.source_object_uuid == id)
                .map(|item| item.object_id.clone())
        })
        .or_else(|| {
            scene
                .outline
                .iter()
                .find(|item| item.object_id == id || item.source_object_uuid == id)
                .map(|item| item.object_id.clone())
        })
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
struct CheckRunEnvelope {
    #[serde(default)]
    contract: Option<String>,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    model_revision: Option<String>,
    #[serde(default)]
    check_run: Option<CheckRunPayload>,
    #[serde(default)]
    check_run_id: Option<String>,
    #[serde(default)]
    profile_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    persisted: bool,
    #[serde(default)]
    summary: Value,
    #[serde(default)]
    finding_count: usize,
    #[serde(default)]
    findings: Vec<CheckFindingSummary>,
    #[serde(default)]
    proposal_refs: Vec<String>,
    #[serde(default)]
    proposal_links: Vec<Value>,
    #[serde(default)]
    profile_basis: CheckRunProfileBasisSummary,
    #[serde(default)]
    coverage: Vec<CheckRunCoverageSummary>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
struct CheckContextSummary {
    #[serde(default)]
    contract: Option<String>,
    #[serde(default)]
    visible_check_runs: Vec<CheckContextRunSummary>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
struct CheckContextRunSummary {
    #[serde(default)]
    check_run_id: Option<String>,
    #[serde(default)]
    model_revision: Option<String>,
    #[serde(default)]
    profile_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    finding_count: usize,
    #[serde(default)]
    proposal_refs: Vec<String>,
    #[serde(default)]
    profile_basis: CheckRunProfileBasisSummary,
    #[serde(default)]
    coverage: Vec<CheckRunCoverageSummary>,
    #[serde(default)]
    active_findings: Vec<CheckFindingSummary>,
}

impl CheckContextRunSummary {
    fn into_state(self) -> CheckRunReviewState {
        let finding_count = if self.finding_count == 0 {
            self.active_findings.len()
        } else {
            self.finding_count
        };
        CheckRunReviewState {
            check_run_id: self.check_run_id,
            project_id: None,
            model_revision: self.model_revision,
            profile_id: self.profile_id,
            status: self.status,
            persisted: true,
            summary: Value::Null,
            finding_count,
            findings: self.active_findings,
            proposal_refs: self.proposal_refs,
            proposal_links: Vec::new(),
            profile_basis: self.profile_basis,
            coverage: self.coverage,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
struct CheckRunPayload {
    #[serde(default)]
    check_run_id: Option<String>,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    model_revision: Option<String>,
    #[serde(default)]
    profile_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    persisted: bool,
    #[serde(default)]
    summary: Value,
    #[serde(default)]
    finding_count: usize,
    #[serde(default)]
    findings: Vec<CheckFindingSummary>,
    #[serde(default)]
    proposal_refs: Vec<String>,
    #[serde(default)]
    proposal_links: Vec<Value>,
    #[serde(default)]
    profile_basis: CheckRunProfileBasisSummary,
    #[serde(default)]
    coverage: Vec<CheckRunCoverageSummary>,
}

impl CheckRunPayload {
    fn into_state(
        self,
        fallback_project_id: Option<String>,
        fallback_model_revision: Option<String>,
    ) -> CheckRunReviewState {
        let finding_count = if self.finding_count == 0 {
            self.findings.len()
        } else {
            self.finding_count
        };
        CheckRunReviewState {
            check_run_id: self.check_run_id,
            project_id: self.project_id.or(fallback_project_id),
            model_revision: self.model_revision.or(fallback_model_revision),
            profile_id: self.profile_id,
            status: self.status,
            persisted: self.persisted,
            summary: self.summary,
            finding_count,
            findings: self.findings,
            proposal_refs: self.proposal_refs,
            proposal_links: self.proposal_links,
            profile_basis: self.profile_basis,
            coverage: self.coverage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_context_v1_maps_visible_run_to_review_state() {
        let context = serde_json::json!({
            "check_context": {
                "contract": "datum_check_context_v1",
                "visible_check_runs": [{
                    "check_run_id": "run-context",
                    "model_revision": "revision-context",
                    "profile_id": "standards",
                    "status": "error",
                    "finding_count": 3,
                    "proposal_refs": ["proposal-a"],
                    "profile_basis": {
                        "profile_id": "standards",
                        "domains": ["standards"],
                        "standards_basis": "datum.process_aperture_and_geometry.current"
                    },
                    "coverage": [{
                        "domain": "standards",
                        "rule_id": "process_aperture_policy",
                        "status": "evaluated",
                        "target_scope": "board_pads_tracks_vias",
                        "standards_basis": "datum.process_aperture_and_geometry.current"
                    }],
                    "active_findings": [{
                        "finding_id": "finding-context",
                        "index": 0,
                        "source": "drc",
                        "code": "pad_mask_expansion_missing",
                        "severity": "error",
                        "fingerprint": "sha256:context-finding",
                        "domain": "standards",
                        "rule_id": "process_aperture_policy",
                        "standards_basis": "datum.process_aperture_and_geometry.current",
                        "rule_revision": "v1",
                        "import_key": "kicad:pad:1",
                        "status": "active",
                        "message": "Pad mask expansion is missing.",
                        "suggested_next_action": "Generate a standards repair proposal.",
                        "proposal_refs": ["proposal-a"],
                        "waiver_refs": [],
                        "deviation_refs": []
                    }]
                }]
            }
        });

        let state = check_run_review_state_from_context_value(&context)
            .expect("context should map to review state");

        assert_eq!(state.check_run_id.as_deref(), Some("run-context"));
        assert_eq!(state.model_revision.as_deref(), Some("revision-context"));
        assert_eq!(state.profile_id.as_deref(), Some("standards"));
        assert_eq!(state.status.as_deref(), Some("error"));
        assert!(state.persisted);
        assert_eq!(state.finding_count, 3);
        assert_eq!(state.findings.len(), 1);
        assert_eq!(state.proposal_refs, vec!["proposal-a"]);
        assert_eq!(
            state.profile_basis.standards_basis.as_deref(),
            Some("datum.process_aperture_and_geometry.current")
        );
        let finding = &state.findings[0];
        assert_eq!(finding.code, "pad_mask_expansion_missing");
        assert_eq!(finding.domain, "standards");
        assert_eq!(
            finding.standards_basis.as_deref(),
            Some("datum.process_aperture_and_geometry.current")
        );
        assert_eq!(finding.rule_revision.as_deref(), Some("v1"));
        assert_eq!(finding.import_key.as_deref(), Some("kicad:pad:1"));
    }
}
