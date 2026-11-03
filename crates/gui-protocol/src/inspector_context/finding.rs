//! Read-only finding identity and display evidence, separate from mutation admission.
use crate::selection_view::InspectionValue;
use crate::{
    CheckFindingSummary, CheckRunReviewState, ReviewWorkspaceState, SelectionSubject,
    SelectionTarget,
};
use eda_engine::schematic::{CheckDeviation, CheckDomain, CheckWaiver, WaiverTarget};
use eda_engine::substrate::DesignModel;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispositionDetail {
    pub id: String,
    pub kind: String,
    pub rationale: String,
    pub project: String,
    pub revision: String,
    pub check_run: String,
    pub fingerprint: String,
}

pub struct FindingInspection<'a> {
    pub finding: &'a CheckFindingSummary,
    pub report: &'a CheckRunReviewState,
    pub currentness: &'static str,
    pub target: Option<String>,
    pub occurrence: Option<String>,
}
impl FindingInspection<'_> {
    pub fn disposition_details(&self) -> impl Iterator<Item = &DispositionDetail> {
        self.finding.disposition_details.iter().filter(|detail| {
            Some(detail.project.as_str()) == self.report.project_id.as_deref()
                && Some(detail.revision.as_str()) == self.report.model_revision.as_deref()
                && Some(detail.check_run.as_str()) == self.report.check_run_id.as_deref()
                && detail.fingerprint == self.finding.fingerprint
                && match detail.kind.as_str() {
                    "Waiver" => self.finding.waiver_refs.contains(&detail.id),
                    "Accepted deviation" => self.finding.deviation_refs.contains(&detail.id),
                    _ => false,
                }
        })
    }
}

/// None means another Inspector owner; Err is an unavailable finding, not a
/// license to fall back to unrelated current evidence with the same fingerprint.
pub fn capture(
    state: &ReviewWorkspaceState,
) -> Option<Result<FindingInspection<'_>, &'static str>> {
    let (fingerprint, view) = match &state.selection {
        SelectionTarget::CheckFinding(id) => (id, None),
        SelectionTarget::Native(view) => match &view.subject {
            SelectionSubject::Diagnostic(id) => (id, Some(view)),
            _ => return None,
        },
        _ => return None,
    };
    Some((|| {
        let run = &state.checks;
        if run.project_id.as_deref() != Some(state.scene.project_uuid.as_str())
            || run
                .check_run_id
                .as_deref()
                .is_none_or(|id| uuid::Uuid::parse_str(id).is_err())
            || run.model_revision.as_deref().is_none_or(str::is_empty)
        {
            return Err("Finding report unavailable for this Project");
        }
        if let Some(view) = view {
            if !view.members.is_empty() || view.project.to_string() != state.scene.project_uuid {
                return Err("Finding selection belongs to another source");
            }
            let Some(InspectionValue::Common(basis)) = view.field_ref("artifact_basis") else {
                return Err("Finding has no producing CheckRun basis");
            };
            if basis["check_run_id"].as_str() != run.check_run_id.as_deref()
                || basis["model_revision"].as_str() != run.model_revision.as_deref()
            {
                return Err("Finding report changed; select a finding from the shown report");
            }
        }
        let mut matches = run
            .findings
            .iter()
            .filter(|f| f.fingerprint == *fingerprint);
        let finding = matches
            .next()
            .ok_or("Finding is absent from its producing report")?;
        if matches.next().is_some() {
            return Err("Finding identity is ambiguous in this report");
        }
        let current = run.model_revision.as_deref() == Some(state.scene.source_revision.as_str())
            && view.is_none_or(|v| !v.stale && v.revision.0 == state.scene.source_revision);
        let currentness = if !current {
            "Stale — historical report; rerun checks"
        } else if run.persisted {
            "Current persisted result"
        } else if run.profile_id.as_deref() == Some("erc") {
            "Current read-only report — Run ERC before authoring"
        } else {
            "Current read-only report — rerun checks before authoring"
        };
        Ok(FindingInspection {
            finding,
            report: run,
            currentness,
            target: human_target(finding),
            occurrence: occurrence(state, finding, current),
        })
    })())
}

fn human_target(finding: &CheckFindingSummary) -> Option<String> {
    // Names come from the producing evidence, never today's object lookup.
    finding
        .evidence
        .iter()
        .chain(std::iter::once(&finding.payload))
        .find_map(|e| {
            e.get("objects")
                .and_then(Value::as_array)
                .and_then(|objects| {
                    objects.iter().find_map(|o| {
                        o.get("key")
                            .and_then(Value::as_str)
                            .filter(|s| !s.is_empty() && uuid::Uuid::parse_str(s).is_err())
                            .map(str::to_owned)
                    })
                })
                .or_else(|| {
                    let component = e.get("component")?.as_str()?;
                    let pin = e.get("pin").and_then(Value::as_str);
                    Some(
                        pin.map_or_else(
                            || component.to_owned(),
                            |pin| format!("{component}.{pin}"),
                        ),
                    )
                })
        })
}
fn occurrence(
    state: &ReviewWorkspaceState,
    finding: &CheckFindingSummary,
    current: bool,
) -> Option<String> {
    let primary = crate::check_runs::marker::occurrence_path(&finding.primary_target);
    let payload = crate::check_runs::marker::occurrence_path(&finding.payload);
    let (Ok(primary), Ok(payload)) = (primary, payload) else {
        return Some("Occurrence unavailable".into());
    };
    if primary
        .as_ref()
        .zip(payload.as_ref())
        .is_some_and(|(a, b)| a != b)
    {
        return Some("Occurrence evidence disagrees".into());
    }
    let path = primary.or(payload)?;
    if path.is_empty() {
        return Some("Root sheet".into());
    }
    if current
        && state
            .schematic_documents
            .project
            .map(|p| p.to_string())
            .as_deref()
            == Some(state.scene.project_uuid.as_str())
        && state
            .schematic_documents
            .revision
            .as_ref()
            .map(|r| r.0.as_str())
            == Some(state.scene.source_revision.as_str())
    {
        let mut rows = state
            .schematic_documents
            .rows
            .iter()
            .filter(|row| row.key.instance_path == path);
        if let Some(row) = rows.next()
            && rows.next().is_none()
        {
            return Some(row.breadcrumb.join(" / "));
        }
    }
    Some("Qualified occurrence — see technical details".into())
}

/// Called only by existing loading workers. Display enrichment cannot create
/// check authority and does not load today's rationale into a historical report.
pub(crate) fn capture_dispositions(
    run: &mut CheckRunReviewState,
    model: &DesignModel,
) -> anyhow::Result<()> {
    for finding in &mut run.findings {
        finding.disposition_details.clear();
    }
    if run.project_id.as_deref() != Some(model.project.project_id.to_string().as_str())
        || run.model_revision.as_deref() != Some(model.model_revision.0.as_str())
        || run.check_run_id.is_none()
        || !run
            .findings
            .iter()
            .any(|f| !f.waiver_refs.is_empty() || !f.deviation_refs.is_empty())
    {
        return Ok(());
    }
    let root = crate::check_runs::loading::disposition_basis(model)?;
    let waivers: Vec<CheckWaiver> = serde_json::from_value(
        root.get("waivers")
            .cloned()
            .filter(|v| !v.is_null())
            .unwrap_or_else(|| serde_json::json!([])),
    )?;
    let deviations: Vec<CheckDeviation> = serde_json::from_value(
        root.get("deviations")
            .cloned()
            .filter(|v| !v.is_null())
            .unwrap_or_else(|| serde_json::json!([])),
    )?;
    attach_dispositions(run, &waivers, &deviations);
    Ok(())
}
fn attach_dispositions(
    run: &mut CheckRunReviewState,
    waivers: &[CheckWaiver],
    deviations: &[CheckDeviation],
) {
    let (Some(project), Some(revision), Some(check_run)) =
        (&run.project_id, &run.model_revision, &run.check_run_id)
    else {
        return;
    };
    for finding in &mut run.findings {
        for (kind, refs) in [
            ("Waiver", &finding.waiver_refs),
            ("Accepted deviation", &finding.deviation_refs),
        ] {
            for id in refs {
                if refs.iter().filter(|r| *r == id).count() != 1 {
                    continue;
                }
                let records: Vec<_> = if kind == "Waiver" {
                    waivers
                        .iter()
                        .filter(|w| w.uuid.to_string() == *id)
                        .map(|w| (&w.domain, &w.target, &w.rationale))
                        .collect()
                } else {
                    deviations
                        .iter()
                        .filter(|d| d.uuid.to_string() == *id)
                        .map(|d| (&d.domain, &d.target, &d.rationale))
                        .collect()
                };
                if records.len() != 1 {
                    continue;
                }
                let (domain, target, rationale) = records[0];
                let WaiverTarget::Fingerprint(fingerprint) = target else {
                    continue;
                };
                if *domain != CheckDomain::ERC
                    || finding.domain != "erc"
                    || fingerprint != &finding.fingerprint
                {
                    continue;
                }
                finding.disposition_details.push(DispositionDetail {
                    id: id.clone(),
                    kind: kind.into(),
                    rationale: rationale.clone(),
                    project: project.clone(),
                    revision: revision.clone(),
                    check_run: check_run.clone(),
                    fingerprint: finding.fingerprint.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests;
