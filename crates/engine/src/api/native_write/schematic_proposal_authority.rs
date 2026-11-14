//! Manual Proposal family authority and current-source admission.
//! Preview and acceptance share this owner; it admits no arbitrary batch.
use super::*;

pub(super) fn native_proposal(model: &DesignModel, id: Uuid) -> Result<&Proposal, EngineError> {
    let p = model
        .proposals
        .get(&id)
        .ok_or_else(|| invalid("Proposal is unavailable"))?;
    if p.source != ProposalSource::Manual
        || p.batch.provenance.source != CommitSource::Manual
        || p.project_id != model.project.project_id
    {
        return Err(invalid(
            "This native action requires a manual proposal for this project",
        ));
    }
    if super::super::schematic_board_transfer::connectivity_revision::is_revision(p) {
        super::super::schematic_board_transfer::connectivity_revision::validate_recorded(p)?;
        return Ok(p);
    }
    if super::super::schematic_board_transfer::field_revision::is_revision(p) {
        super::super::schematic_board_transfer::field_revision::validate_recorded(p)?;
        return Ok(p);
    }
    if super::super::design_clipboard_receipt::is_design_edit(p) {
        super::super::design_clipboard_receipt::validate_recorded(p)?;
        return Ok(p);
    }
    if super::super::schematic_interface_binding::is_interface_binding(p) {
        super::super::schematic_interface_binding::validate_recorded(p)?;
        return Ok(p);
    }
    if super::super::schematic_child_sheet::is_child_sheet(p) {
        super::super::schematic_child_sheet::validate_recorded(p)?;
        return Ok(p);
    }
    if super::super::schematic_gate_placement::is_gate_placement(p) {
        super::super::schematic_gate_placement::validate_recorded(p)?;
        return Ok(p);
    }
    if super::super::schematic_field_batch::is_field_batch(p) {
        super::super::schematic_field_batch::validate_recorded(p)?;
        return Ok(p);
    }
    if super::super::schematic_project_annotation::is_project_annotation(p) {
        super::super::schematic_project_annotation::validate_recorded(p)?;
        return Ok(p);
    }
    if super::super::schematic_annotation::is_annotation(p) {
        super::super::schematic_annotation::validate_recorded(p)?;
        return Ok(p);
    }
    if super::super::schematic_assignment::is_assignment(p) {
        super::super::schematic_assignment::validate_recorded(p)?;
        return Ok(p);
    }
    if super::super::schematic_disposition::is_disposition(p) {
        super::super::schematic_disposition::summary(model, p, false)?;
        return Ok(p);
    }
    if !p
        .batch
        .operations
        .iter()
        .any(|o| schematic_delete::deleted_object(o).is_some())
        || p.batch.operations.iter().any(|o| {
            schematic_delete::deleted_object(o).is_none()
                && !matches!(
                    o,
                    Operation::SetComponentInstance { .. }
                        | Operation::GuardObjectRevision { .. }
                        | Operation::CreateElectricalIdentity { .. }
                        | Operation::SetElectricalIdentity { .. }
                        | Operation::DeleteElectricalIdentity { .. }
                )
        })
    {
        return Err(invalid(
            "This native action requires a manual schematic deletion proposal",
        ));
    }
    super::super::schematic_delete::validate_owned_components(&p.batch.operations)?;
    Ok(p)
}
pub(super) fn validate_disposition_current(
    model: &DesignModel,
    proposal: &Proposal,
    root: &std::path::Path,
) -> Result<(), EngineError> {
    if super::super::schematic_board_transfer::connectivity_revision::is_revision(proposal) {
        super::super::schematic_board_transfer::connectivity_revision::summary(
            model, proposal, true,
        )?;
    }
    if super::super::schematic_board_transfer::field_revision::is_revision(proposal) {
        super::super::schematic_board_transfer::field_revision::summary(model, proposal, true)?;
    }
    if super::super::design_clipboard_receipt::is_design_edit(proposal) {
        let context = NativeWriteContext {
            model,
            project_root: root,
        };
        super::super::design_clipboard_receipt::validate_current(&context, proposal)?;
    }
    if super::super::schematic_interface_binding::is_interface_binding(proposal) {
        super::super::schematic_interface_binding::summary(model, proposal, true)?;
    }
    if super::super::schematic_child_sheet::is_child_sheet(proposal) {
        super::super::schematic_child_sheet::summary(model, proposal, true)?;
    }
    if super::super::schematic_gate_placement::is_gate_placement(proposal) {
        super::super::schematic_gate_placement::summary(model, proposal, true)?;
    }
    if super::super::schematic_field_batch::is_field_batch(proposal) {
        super::super::schematic_field_batch::summary(model, proposal, true)?;
    }
    if super::super::schematic_project_annotation::is_project_annotation(proposal) {
        super::super::schematic_project_annotation::summary(model, proposal, true)?;
    }
    if super::super::schematic_annotation::is_annotation(proposal) {
        super::super::schematic_annotation::summary(model, proposal, true)?;
    }
    if super::super::schematic_assignment::is_assignment(proposal) {
        super::super::schematic_assignment::summary(model, proposal, true)?;
    }
    if super::super::schematic_disposition::is_disposition(proposal) {
        super::super::schematic_disposition::summary(model, proposal, true)?;
    }
    Ok(())
}
