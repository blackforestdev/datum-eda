//! Check state and counts always retain the producing Project/revision and coverage.
use super::*;
use crate::CheckProducerPhase as P;
use crate::check_runs::marker::occurrence_path as path;

pub(super) fn append<'a>(context: &mut InspectorContext<'a>, state: &'a ReviewWorkspaceState) {
    let section = InspectorSectionId::Checks;
    let producer = &state.check_producer;
    if producer.phase != P::Idle {
        let phase = match producer.phase {
            P::Idle => "Idle",
            P::Starting => "Starting",
            P::Running => "Running",
            P::Complete => "Complete",
            P::Unavailable => "Unavailable",
        };
        context.push(section, InspectorField::text("Job", phase));
        if let Some(reason) = producer.reason.as_deref() {
            context.push(section, InspectorField::text("Job reason", reason));
        }
        if let Some(request) = &producer.request {
            context.push(
                section,
                InspectorField::text("Job UUID", request.job_id.to_string()),
            );
            context.push(
                section,
                InspectorField::text("Job source revision", request.model_revision.as_str()),
            );
            context.push(
                section,
                InspectorField::text(
                    "Job view basis",
                    if request.matches_workspace(state) {
                        "Current captured source basis"
                    } else {
                        "Previous source basis — completion cannot become current evidence here"
                    },
                ),
            );
        }
    }
    let checks = &state.checks;
    let Some(id) = checks.check_run_id.as_deref() else {
        let no_history = state
            .library
            .basis
            .as_ref()
            .is_some_and(|model| model.check_runs.is_empty());
        context.push(
            section,
            InspectorField::text(
                "State",
                if context.action_basis.is_some() && no_history {
                    "Not run"
                } else {
                    "Unavailable — no qualified persisted check result was captured"
                },
            ),
        );
        return;
    };
    let qualification = checks.qualification(state);
    let current = qualification.is_ok();
    context.push(
        section,
        InspectorField::text(
            "State",
            if current && !checks.persisted {
                "Current read-only report — run the selected check profile to persist evidence"
            } else if current {
                "Current persisted result"
            } else if checks.profile_id.as_deref() == Some("drc") {
                "Not current — historical or unqualified DRC; rerun on current basis"
            } else {
                "Stale — historical result; rerun on current basis"
            },
        ),
    );
    if let Err(reason) = qualification {
        context.push(section, InspectorField::text("Report basis", reason));
    }
    context.push(section, InspectorField::text("Run UUID", id));
    context.push(
        section,
        InspectorField::text(
            "Producing revision",
            checks.model_revision.as_deref().unwrap_or("Unavailable"),
        ),
    );
    context.push(
        section,
        InspectorField::text(
            "Profile",
            checks.profile_id.as_deref().unwrap_or("Unavailable"),
        ),
    );
    context.push(
        section,
        InspectorField::text(
            if current {
                "Project findings"
            } else {
                "Historical Project findings"
            },
            checks.finding_count.to_string(),
        ),
    );
    if checks.findings.len() == checks.finding_count {
        let count = |severity: &str| {
            checks
                .findings
                .iter()
                .filter(|finding| finding.severity == severity)
                .count()
        };
        context.push(
            section,
            InspectorField::text(
                if current {
                    "Project severity"
                } else {
                    "Historical Project severity"
                },
                format!(
                    "{} errors · {} warnings · {} info (all recorded statuses)",
                    count("error"),
                    count("warning"),
                    count("info")
                ),
            ),
        );
    }
    for coverage in &checks.coverage {
        context.push(
            section,
            InspectorField::text(
                "Coverage",
                format!(
                    "{} / {} — {} ({})",
                    coverage.domain, coverage.rule_id, coverage.status, coverage.target_scope
                ),
            ),
        );
    }
    append_recorded_findings(context, checks, current);
    if !current {
        return;
    }
    if checks.profile_id.as_deref() == Some("drc") {
        append_drc_scope(context, state);
        return;
    }
    if checks.profile_id.as_deref() != Some("erc") {
        context.push(
            section,
            InspectorField::text(
                "Scoped checks",
                "Unavailable — this profile has no supported scoped projection",
            ),
        );
        return;
    }
    let evaluated: Vec<_> = checks
        .coverage
        .iter()
        .filter(|c| c.domain == "erc" && c.status == "evaluated")
        .collect();
    if evaluated.is_empty() {
        context.push(
            section,
            InspectorField::text(
                "Scoped ERC",
                "Unavailable — no evaluated ERC coverage on this run",
            ),
        );
        return;
    }
    if evaluated
        .iter()
        .all(|coverage| coverage.target_scope != "schematic")
    {
        context.push(
            section,
            InspectorField::text(
                "Scoped ERC",
                "Unavailable — evaluated coverage has no applicable schematic target scope",
            ),
        );
        return;
    }
    if let Some(view) = context.view {
        use crate::AuthoredSelectionClass as C;
        let total = view.scoped_members().count();
        let covered = view
            .scoped_members()
            .filter(|member| {
                matches!(
                    member.class,
                    C::Symbol | C::Pin | C::Wire | C::Label | C::Port | C::Junction | C::NoConnect
                )
            })
            .count();
        if covered == 0 {
            context.push(section, InspectorField::text("Scoped ERC", "Unavailable — selected scope has no schematic electrical targets covered by this ERC run; related objects are not selected"));
            return;
        }
        context.push(section, InspectorField::text("Scope coverage", format!("{covered} schematic electrical members of {total}; other members are not evaluated by ERC")));
    } else if !context.document.is_some_and(|key| {
        state.schematic_documents.row(key).is_some()
            && state.schematic_documents.source_sheet(key).is_some()
    }) {
        context.push(section, InspectorField::text("Scoped ERC", "Unavailable — no qualified current schematic document or selected electrical scope"));
        return;
    }
    if state
        .schematic_documents
        .project
        .map(|id| id.to_string())
        .as_deref()
        != Some(state.scene.project_uuid.as_str())
        || state
            .schematic_documents
            .revision
            .as_ref()
            .map(|r| r.0.as_str())
            != Some(state.scene.source_revision.as_str())
        || checks.findings.len() != checks.finding_count
        || checks.findings.iter().any(|f| {
            f.domain == "erc"
                && !matches!(
                    f.status.as_str(),
                    "active" | "waived" | "accepted_deviation"
                )
        })
        || context.view.is_some_and(|v| {
            v.stale
                || v.project.to_string() != state.scene.project_uuid
                || v.revision.0 != state.scene.source_revision
        })
    {
        context.push(
            section,
            InspectorField::text(
                "Scoped ERC",
                "Unavailable — complete current finding/selection basis is missing",
            ),
        );
        return;
    }
    let mut errors = 0;
    let mut warnings = 0;
    let mut infos = 0;
    let mut suppressed = 0;
    for finding in checks.findings.iter().filter(|f| f.domain == "erc") {
        match affects(context, state, finding) {
            Ok(true) if finding.status != "active" => suppressed += 1,
            Ok(true) => match finding.severity.as_str() {
                "error" => errors += 1,
                "warning" => warnings += 1,
                "info" => infos += 1,
                _ => {
                    context.push(
                        section,
                        InspectorField::text(
                            "Scoped ERC",
                            "Unavailable — finding severity is not qualified",
                        ),
                    );
                    return;
                }
            },
            Ok(false) => {}
            Err(reason) => {
                context.push(
                    section,
                    InspectorField::text("Scoped ERC", format!("Unavailable — {reason}")),
                );
                return;
            }
        }
    }
    context.push(
        section,
        InspectorField::text(
            "Scoped ERC",
            format!("{errors} errors · {warnings} warnings · {infos} infos"),
        ),
    );
    if suppressed > 0 {
        context.push(section, InspectorField::text("Suppressed scoped findings", format!("{suppressed} waived or accepted-deviation findings; excluded from active counts")));
    }
    if errors + warnings + infos == 0 && suppressed == 0 {
        context.push(
            section,
            InspectorField::text(
                "Evaluated coverage",
                format!(
                    "Clean — {} only",
                    evaluated
                        .iter()
                        .map(|c| c.rule_id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ),
        );
    }
}

/// Recorded logical findings remain readable even when they have no precise
/// world location. A text row is evidence inspection, not a fabricated marker.
fn append_recorded_findings<'a>(
    context: &mut InspectorContext<'a>,
    checks: &'a crate::CheckRunReviewState,
    current: bool,
) {
    let section = InspectorSectionId::Checks;
    if checks.findings.len() != checks.finding_count {
        context.push(section, InspectorField::text("Finding details", format!(
            "Incomplete — {} of {} recorded findings captured; rerun the profile for a complete report",
            checks.findings.len(), checks.finding_count,
        )));
    }
    for finding in &checks.findings {
        context.push(
            section,
            InspectorField::text(
                if current {
                    "Finding"
                } else {
                    "Historical finding"
                },
                format!(
                    "{} · {} · {} · {}",
                    finding.domain, finding.severity, finding.code, finding.status
                ),
            ),
        );
        context.push(
            section,
            InspectorField::text("Finding rule", finding.rule_id.as_str()),
        );
        context.push(
            section,
            InspectorField::text("Finding message", finding.message.as_str()),
        );
        context.push(
            section,
            InspectorField::text(
                "Finding target",
                finding
                    .target_label()
                    .unwrap_or_else(|| "Unavailable — no recorded target identity".into()),
            ),
        );
        if !finding.explanation.is_empty() {
            context.push(
                section,
                InspectorField::text("Finding explanation", finding.explanation.as_str()),
            );
        }
        if let Some(next) = finding.suggested_next_action.as_deref() {
            context.push(section, InspectorField::text("Suggested action", next));
        }
    }
}

fn append_drc_scope(context: &mut InspectorContext<'_>, state: &ReviewWorkspaceState) {
    use crate::AuthoredSelectionClass as C;
    let section = InspectorSectionId::Checks;
    context.push(section, InspectorField::text("DRC coverage", "Only the recorded rules and supported geometry are evaluated; unavailable coverage is not a clean result"));
    let checks = &state.checks;
    if !checks.persisted
        || checks.findings.len() != checks.finding_count
        || !checks
            .coverage
            .iter()
            .any(|c| c.domain == "drc" && c.status == "evaluated")
    {
        context.push(
            section,
            InspectorField::text(
                "Scoped DRC",
                "Unavailable — complete persisted findings and evaluated DRC coverage are required",
            ),
        );
        return;
    }
    let Some(view) = context.view else {
        context.push(section, InspectorField::text("DRC scope", "Project-wide board report; select board objects to inspect their recorded findings"));
        return;
    };
    if view.stale
        || view.project.to_string() != state.scene.project_uuid
        || view.revision.0 != state.scene.source_revision
    {
        context.push(
            section,
            InspectorField::text("Scoped DRC", "Unavailable — selection source is stale"),
        );
        return;
    }
    let selected: BTreeSet<_> = view
        .scoped_members()
        .filter(|m| {
            m.instance_path.is_empty()
                && matches!(
                    m.class,
                    C::Footprint
                        | C::Pad
                        | C::Track
                        | C::Via
                        | C::Zone
                        | C::BoardText
                        | C::BoardGraphic
                        | C::BoardOutline
                )
        })
        .map(|m| m.id)
        .collect();
    if selected.is_empty() {
        context.push(section, InspectorField::text("Scoped DRC", "Unavailable — selected scope has no board objects; related objects are not selected"));
        return;
    }
    let counts = (|| -> Result<[usize; 4], &'static str> {
        let mut counts = [0; 4];
        for finding in checks
            .findings
            .iter()
            .filter(|f| f.source == "drc" || f.domain == "drc")
        {
            let targets = affected_targets(state, finding)?;
            if targets
                .iter()
                .any(|(_, frame)| frame.as_ref().is_some_and(|p| !p.is_empty()))
            {
                return Err("Board finding declares a schematic occurrence frame");
            }
            if !targets.iter().any(|(id, _)| selected.contains(id)) {
                continue;
            }
            let index = match (finding.status.as_str(), finding.severity.as_str()) {
                ("waived" | "accepted_deviation", _) => 3,
                ("active", "error") => 0,
                ("active", "warning") => 1,
                ("active", "info") => 2,
                _ => return Err("Finding status or severity is not qualified"),
            };
            counts[index] += 1;
        }
        Ok(counts)
    })();
    match counts {
        Ok([errors, warnings, infos, suppressed]) => context.push(section, InspectorField::text("Scoped DRC", format!("{errors} errors · {warnings} warnings · {infos} infos · {suppressed} suppressed; exact selected object targets only"))),
        Err(reason) => context.push(section, InspectorField::text("Scoped DRC", format!("Unavailable — {reason}"))),
    }
}

// The (Uuid, optional dependent-Uuids) pairs are a localized return shape used by
// this one analysis only; a type alias would add indirection without clarifying
// the single call site.
#[allow(clippy::type_complexity)]
fn affected_targets(
    state: &ReviewWorkspaceState,
    finding: &crate::CheckFindingSummary,
) -> Result<Vec<(Uuid, Option<Vec<Uuid>>)>, &'static str> {
    let payload_frame = path(&finding.payload)?;
    let primary_frame = path(&finding.primary_target)?;
    if primary_frame
        .as_ref()
        .zip(payload_frame.as_ref())
        .is_some_and(|(a, b)| a != b)
    {
        return Err("Finding primary and producing occurrence frames disagree");
    }
    let mut targets = Vec::new();
    for (index, target) in std::iter::once(&finding.primary_target)
        .chain(&finding.related_targets)
        .enumerate()
    {
        if target.get("kind").and_then(serde_json::Value::as_str) != Some("object_uuid") {
            return Err("Finding has no typed affected-object identity for scope qualification");
        }
        let id = target
            .get("id")
            .and_then(serde_json::Value::as_str)
            .and_then(|s| s.parse::<Uuid>().ok())
            .ok_or("Finding affected-object UUID is unavailable")?;
        let frame = path(target)?.or_else(|| payload_frame.clone());
        targets.push((
            id,
            if index == 0 {
                primary_frame.clone().or(frame)
            } else {
                frame
            },
        ));
    }
    if let Some(objects) = finding.payload.get("object_uuids").filter(|v| !v.is_null()) {
        for id in objects
            .as_array()
            .ok_or("Affected object list is malformed")?
        {
            let id = id
                .as_str()
                .and_then(|id| id.parse::<Uuid>().ok())
                .ok_or("Affected object UUID is malformed")?;
            targets.push((id, primary_frame.clone().or_else(|| payload_frame.clone())));
        }
    }
    if let Some(pins) = finding.payload.get("pin_evidence").filter(|v| !v.is_null()) {
        for pin in pins
            .as_array()
            .ok_or("Affected pin evidence is malformed")?
        {
            let id = pin
                .get("pin_uuid")
                .and_then(serde_json::Value::as_str)
                .and_then(|id| id.parse::<Uuid>().ok())
                .ok_or("Affected pin UUID is malformed")?;
            if !state.schematic_documents.is_source_pin(id) {
                return Err("Affected pin UUID is not a captured schematic Pin");
            }
            let frame = path(pin)?
                .or_else(|| primary_frame.clone())
                .or_else(|| payload_frame.clone());
            targets.push((id, frame));
        }
    }
    Ok(targets)
}

fn affects(
    context: &InspectorContext<'_>,
    state: &ReviewWorkspaceState,
    finding: &crate::CheckFindingSummary,
) -> Result<bool, &'static str> {
    let mut matched = false;
    for (id, declared_frame) in affected_targets(state, finding)? {
        let occurrence = crate::schematic_documents::qualify_source_occurrence(
            state.schematic_documents.source_occurrences(id),
            declared_frame.as_deref(),
        )?;
        if let Some(view) = context.view {
            matched |= view.scoped_members().any(|member| {
                member.instance_path == occurrence.instance_path
                    && (member.id == id
                        || (member.class == crate::AuthoredSelectionClass::Symbol
                            && state.schematic_documents.symbol_has_pin(member.id, id)))
            });
        } else if let Some(document) = context.document {
            matched |= document == occurrence;
        } else {
            return Err("No qualified document or authored selection scope");
        }
    }
    Ok(matched)
}
