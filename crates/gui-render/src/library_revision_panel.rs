//! Reviewed local revision states inside the ordinary Library surface.
use super::*;
use datum_gui_protocol::library_browser::revision_workflow::RevisionControl as R;

pub(super) fn reviews_local_revision(p: &Painter<'_>, draft: &LibraryDraft) -> bool {
    !draft.create
        && p.state
            .library
            .basis
            .as_ref()
            .is_some_and(|model| model.project.schema_version == Some(2))
}

pub(super) fn commit_label(p: &Painter<'_>, draft: &LibraryDraft, compact: bool) -> &'static str {
    match (
        reviews_local_revision(p, draft),
        compact,
        draft.preview.is_some(),
    ) {
        (true, true, _) => "Review revision",
        (false, true, _) => "Commit",
        (true, false, true) => "Review local revision",
        (true, false, false) => "Review local revision · preview required",
        (false, false, true) => "Commit to pool",
        (false, false, false) => "Commit unavailable · preview required",
    }
}

pub(super) fn actions(p: &mut Painter<'_>) {
    for (label, control) in p.state.library.revision_entry_controls(&p.state.selection) {
        p.button(&label, C::Revision(control));
    }
}

pub(super) fn render(p: &mut Painter<'_>) {
    let review = p.state.library.revision_workflow.as_ref().unwrap();
    let rect = p.rect;
    let controls: Vec<_> = review
        .review_controls()
        .into_iter()
        .filter_map(|control| {
            let label = match control {
                R::Preview { .. } => "Preview exact change",
                R::Apply { .. } => "Apply reviewed change",
                R::Cancel(_) => "Cancel review",
                _ => return None,
            };
            Some((label, control))
        })
        .collect();
    let footer = if controls.is_empty() {
        0.
    } else {
        controls
            .iter()
            .map(|(label, _)| p.button_rect(label).height + 4.)
            .sum::<f32>()
            + 8.
    };
    p.rect.height = (rect.height - footer).max(0.);
    let starts = (p.quads.len(), p.text.len(), p.hits.len());
    let title = match review.action() {
        "adopt_profile" => "Review project library format",
        "save_candidate" => "Save a local library revision",
        "adopt" | "adopt_selection" => "Review exact project library update",
        _ => "Library revision review unavailable",
    };
    p.line(title);
    match review.action() {
        "adopt_profile" => {
            p.line("This separate forward-only change creates an empty managed library baseline.");
            p.line("Existing definitions remain native. Historical authorship, approval and rights are not inferred. Undo does not return to the old format.");
        }
        "save_candidate" => p.line("Saving records a local candidate. Active project definitions and placed components remain unchanged until a separate reviewed update."),
        "adopt" | "adopt_selection" => p.line("Apply updates the exact reviewed project closure and affected component bindings through canonical history."),
        _ => {}
    }
    if matches!(review.action(), "adopt" | "adopt_selection") {
        p.line("This update can include related parent definitions. Review every included revision and affected component below before applying.");
    }
    p.line("Purpose of this change");
    acquisition_panel::field(
        p,
        review.purpose(),
        review.purpose_editor(),
        C::Revision(review.purpose_control()),
    );
    if let Some(control) = review
        .review_controls()
        .into_iter()
        .find(|control| matches!(control, R::RecordDetails { .. }))
    {
        let R::RecordDetails { show, .. } = control else {
            unreachable!()
        };
        p.button(
            if show {
                "Show full recorded provenance and revision metadata"
            } else {
                "Hide recorded metadata details"
            },
            C::Revision(control),
        );
    }
    if review.summary().is_some() {
        for line in review.summary_lines() {
            p.line(&line);
        }
    } else {
        p.line("Preview is required before Apply. Impact is unknown until the service evaluates this exact change.");
    }
    if let Some(refusal) = &p.state.library.refusal {
        p.line(refusal);
    }
    if review.is_submitted() {
        p.line("Change submitted. Inspect its confirmed outcome in Console; an unconfirmed write is not retried.");
    }
    // Only review content scrolls. The same qualified actions and keyboard
    // controls stay available while inspecting long geometry/provenance diffs.
    scroll_content(p, starts, 0);
    p.y = rect.y + p.rect.height + 4.;
    p.rect = rect;
    p.unbounded = false;
    for (label, control) in controls {
        p.button(label, C::Revision(control));
    }
}
