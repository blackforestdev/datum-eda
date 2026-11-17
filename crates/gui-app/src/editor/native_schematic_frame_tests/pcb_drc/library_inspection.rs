//! Check evidence must refresh common inspection on its new journal basis.
use super::*;

pub(super) fn assert_current(workflow: &Workflow) {
    let model = workflow.model();
    let library = &workflow
        .host
        .app
        .runtime
        .as_ref()
        .unwrap()
        .workspace()
        .library;
    let inspection = library
        .revision_inspection
        .as_ref()
        .expect("Check completion refreshes Library inspection without reopening");
    assert_eq!(inspection["model_revision"], json!(model.model_revision));
    assert_eq!(
        inspection["transaction_tip"],
        json!(eda_engine::revision::transaction_tip(&model.journal))
    );
    // Exercise the same qualification used before ordinary revision review.
    library
        .clone()
        .set_revision_inspection(inspection.clone())
        .unwrap();
}

pub(super) fn assert_evidence_refresh(workflow: &Workflow) {
    assert_current(workflow);
    let model = workflow.model();
    let transaction = model.journal.last().unwrap();
    assert!(
        transaction
            .operations
            .iter()
            .any(|op| matches!(op, Operation::SetCheckRun { .. }))
    );
    assert_eq!(
        transaction.before_model_revision, transaction.after_model_revision,
        "Check evidence preserves the authored revision"
    );
    assert_ne!(
        eda_engine::revision::transaction_tip(&model.journal),
        eda_engine::revision::transaction_tip(&model.journal[..model.journal.len() - 1]),
        "Check evidence advances the journal tip"
    );
}
