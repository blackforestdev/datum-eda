use std::collections::BTreeMap;
use std::path::Path;

use eda_engine::substrate::{
    CheckFinding, CheckRun, DesignModel, Proposal, validate_proposal_apply,
};
use uuid::Uuid;

use super::{
    NativeProjectCheckFindingView, NativeProjectCheckProposalCommandTemplates,
    NativeProjectCheckProposalLinkView,
};

pub(crate) fn apply_proposal_links(
    root: &Path,
    findings: &mut [NativeProjectCheckFindingView],
    model: &DesignModel,
) -> Vec<NativeProjectCheckProposalLinkView> {
    let mut run_links = BTreeMap::new();
    for finding in findings {
        let links = model
            .proposals
            .iter()
            .filter_map(|(proposal_id, proposal)| {
                proposal_matched_fingerprint(proposal, finding).map(|matched_fingerprint| {
                    proposal_link_view(
                        root,
                        model,
                        *proposal_id,
                        proposal,
                        Some(matched_fingerprint),
                    )
                })
            })
            .collect::<Vec<_>>();
        for link in &links {
            run_links.insert(link.proposal_id.clone(), link.clone());
            if !finding
                .proposal_refs
                .iter()
                .any(|value| value == &link.proposal_id)
            {
                finding.proposal_refs.push(link.proposal_id.clone());
            }
        }
        finding.proposal_links = links;
        finding.proposal_refs.sort();
        finding.proposal_refs.dedup();
    }
    run_links.into_values().collect()
}

pub(crate) fn apply_proposal_links_to_persisted_check_run(
    root: &Path,
    check_run: &mut CheckRun,
    model: &DesignModel,
) -> Vec<NativeProjectCheckProposalLinkView> {
    let mut run_links = BTreeMap::new();
    for finding in &mut check_run.findings {
        let links = model
            .proposals
            .iter()
            .filter_map(|(proposal_id, proposal)| {
                proposal_matched_persisted_fingerprint(proposal, finding).map(
                    |matched_fingerprint| {
                        proposal_link_view(
                            root,
                            model,
                            *proposal_id,
                            proposal,
                            Some(matched_fingerprint),
                        )
                    },
                )
            })
            .collect::<Vec<_>>();
        for link in &links {
            run_links.insert(link.proposal_id.clone(), link.clone());
            if !finding
                .proposal_refs
                .iter()
                .any(|value| value == &link.proposal_id)
            {
                finding.proposal_refs.push(link.proposal_id.clone());
            }
        }
        finding.proposal_refs.sort();
        finding.proposal_refs.dedup();
        finding.proposal_links = links
            .iter()
            .filter_map(|link| serde_json::to_value(link).ok())
            .collect();
    }
    for link in run_links.values() {
        if !check_run
            .proposal_refs
            .iter()
            .any(|value| value == &link.proposal_id)
        {
            check_run.proposal_refs.push(link.proposal_id.clone());
        }
    }
    check_run.proposal_refs.sort();
    check_run.proposal_refs.dedup();
    check_run.proposal_links = run_links
        .values()
        .filter_map(|link| serde_json::to_value(link).ok())
        .collect();
    run_links.into_values().collect()
}

fn proposal_matched_fingerprint(
    proposal: &Proposal,
    finding: &NativeProjectCheckFindingView,
) -> Option<String> {
    let legacy_finding_id = finding.finding_id.to_string();
    let legacy_uuid_fingerprint = format!("uuid:{}", finding.finding_id);
    proposal
        .finding_fingerprints
        .iter()
        .find(|fingerprint| {
            *fingerprint == &finding.fingerprint
                || *fingerprint == &legacy_finding_id
                || *fingerprint == &legacy_uuid_fingerprint
        })
        .cloned()
}

fn proposal_matched_persisted_fingerprint(
    proposal: &Proposal,
    finding: &CheckFinding,
) -> Option<String> {
    let legacy_finding_id = finding.finding_id.to_string();
    let legacy_uuid_fingerprint = format!("uuid:{}", finding.finding_id);
    proposal
        .finding_fingerprints
        .iter()
        .find(|fingerprint| {
            *fingerprint == &finding.fingerprint
                || *fingerprint == &legacy_finding_id
                || *fingerprint == &legacy_uuid_fingerprint
        })
        .cloned()
}

fn proposal_link_view(
    root: &Path,
    model: &DesignModel,
    proposal_id: Uuid,
    proposal: &Proposal,
    matched_fingerprint: Option<String>,
) -> NativeProjectCheckProposalLinkView {
    let validation = validate_proposal_apply(model, proposal_id).ok();
    let mut blocker_codes = validation
        .as_ref()
        .map(|validation| {
            validation
                .blockers
                .iter()
                .map(|blocker| blocker.code.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if matched_fingerprint.is_some() {
        blocker_codes
            .retain(|code| code != "unknown_check_run" && code != "unlinked_finding_fingerprint");
    }
    let prepared_against_current_model = validation
        .as_ref()
        .map(|validation| validation.prepared_against_current_model)
        .unwrap_or(proposal.prepared_against == model.model_revision);
    let can_apply = validation
        .as_ref()
        .map(|validation| validation.can_apply)
        .unwrap_or(false);
    NativeProjectCheckProposalLinkView {
        proposal_id: proposal_id.to_string(),
        status: serde_json_string(proposal.status),
        source: serde_json_string(proposal.source),
        rationale: proposal.rationale.clone(),
        prepared_against: proposal.prepared_against.0.clone(),
        checks_run: proposal
            .checks_run
            .iter()
            .map(|check_run_id| check_run_id.to_string())
            .collect(),
        finding_fingerprints: proposal.finding_fingerprints.clone(),
        matched_fingerprint,
        prepared_against_current_model,
        can_apply,
        blocker_codes,
        command_templates: command_templates(root, proposal_id),
    }
}

fn command_templates(root: &Path, proposal_id: Uuid) -> NativeProjectCheckProposalCommandTemplates {
    let root = root.display();
    NativeProjectCheckProposalCommandTemplates {
        show: format!("datum-eda proposal show {root} --proposal {proposal_id}"),
        preview: format!("datum-eda proposal preview {root} --proposal {proposal_id}"),
        validate: format!("datum-eda proposal validate {root} --proposal {proposal_id}"),
        accept_apply: format!("datum-eda proposal accept-apply {root} --proposal {proposal_id}"),
        apply: format!("datum-eda proposal apply {root} --proposal {proposal_id}"),
        defer: format!("datum-eda proposal defer {root} --proposal {proposal_id}"),
        reject: format!("datum-eda proposal reject {root} --proposal {proposal_id}"),
    }
}

fn serde_json_string<T: serde::Serialize>(value: T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "unknown".to_string())
}
