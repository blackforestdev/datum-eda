//! Physical qualification over commit-certified final source values.
//! This is independent of the unchanged-pool compensation and metadata gates.
use super::*;
use crate::substrate::journal_source_frame::VerifiedPendingSourceFrame;
use serde_json::Value;

pub(in crate::substrate) fn projection(
    frame: &VerifiedPendingSourceFrame,
    board: &Value,
) -> Result<ComponentPadProjection, EngineError> {
    super::projection::project_with_part_basis(
        frame.candidate(),
        frame.candidate(),
        None,
        board,
        &[],
        None,
        Some(frame),
    )
}

pub(super) fn status(
    frame: &VerifiedPendingSourceFrame,
    pad: &ComponentPadSource,
    part: Uuid,
    package: Uuid,
) -> ComponentPadMaterializationStatus {
    let Some(binding) = &pad.source_binding else {
        return ComponentPadMaterializationStatus::Unverified;
    };
    if pad.validate_frozen().is_err() || binding.project_id != frame.candidate().project.project_id
    {
        return ComponentPadMaterializationStatus::Unverified;
    }
    let context = pins::PinFrame::Pending(frame);
    if binding.part.object_id != part
        || binding.package.object_id != package
        || [
            &binding.part,
            &binding.package,
            &binding.land_pattern,
            &binding.library_pad,
            &binding.padstack,
        ]
        .into_iter()
        .chain(binding.pin_pad_map.iter())
        .any(|pin| pins::check_pin_in(&context, pin).is_err())
        || binding
            .process_snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.check_current_in_frame(frame).is_err())
    {
        ComponentPadMaterializationStatus::Stale
    } else {
        ComponentPadMaterializationStatus::Current
    }
}
