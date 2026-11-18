//! The ordinary declaration form uses the shared Library field renderer.
use super::*;
use datum_gui_protocol::library_browser::revision_workflow::{
    LibraryRevisionWorkflow,
    provenance::{ProvenanceAction as A, ProvenanceField},
};

pub(super) fn render(p: &mut Painter<'_>, review: &LibraryRevisionWorkflow) {
    let Some(form) = review.provenance() else {
        return;
    };
    if review.is_submitted() {
        p.line("Declaration change submitted; reconcile its confirmed outcome before editing.");
        return;
    }
    p.line(&format!("Selected asset: {}", form.asset_name));
    p.line("Declarations only. Redistribution authority and engineering approval remain separate. Existing authorship and other evidence are preserved.");
    if review.summary().is_some() {
        p.button(
            "Edit declarations",
            C::Revision(review.provenance_control(A::EditFields)),
        );
        p.line("Selected payload and forward dependencies unchanged. Other assets keep their declarations; parent revisions record their own technical history.");
        return;
    }
    let draw_field =
        |p: &mut Painter<'_>,
         row,
         index,
         field: &datum_gui_protocol::library_authoring::LibraryField| {
            p.line(field.label);
            acquisition_panel::field(
                p,
                &field.text,
                field.editor,
                C::Revision(review.provenance_control(A::Field(ProvenanceField { row, index }))),
            );
        };
    draw_field(p, form.id, 0, &form.license);
    p.line("Attribution entries");
    for (index, row) in form.attributions.iter().enumerate() {
        p.line(&format!("Attribution {}", index + 1));
        draw_field(p, row.id, 0, &row.field);
        p.button(
            &format!("Remove attribution {}", index + 1),
            C::Revision(review.provenance_control(A::RemoveAttribution(row.id))),
        );
    }
    p.button(
        "Add attribution",
        C::Revision(review.provenance_control(A::AddAttribution)),
    );
    for (index, row) in form.citations.iter().enumerate() {
        p.line(&format!("Source citation {}", index + 1));
        for (field_index, field) in row.fields.iter().enumerate() {
            draw_field(p, row.id, field_index, field);
        }
        if let Some(retained) = row.retained_file() {
            p.line(&format!("Retained file: {retained} · read-only"));
            p.line("Remove citation unavailable · retained file reference kept");
        } else {
            p.button(
                &format!("Remove citation {}", index + 1),
                C::Revision(review.provenance_control(A::RemoveCitation(row.id))),
            );
        }
    }
    p.button(
        "Add source citation",
        C::Revision(review.provenance_control(A::AddCitation)),
    );
    p.line("Each citation retains its own fields. This editor does not copy or remove source documents.");
}
