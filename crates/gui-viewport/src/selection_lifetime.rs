//! Shared revision reconciliation (PM026 / UVT 2.2.18).
//!
//! A result is consumer selection state only. This service has no engine writer,
//! journal, scene reconstruction or visibility-filter dependency.

use datum_gui_protocol::selection_resolution::{
    SelectionModelRevision, SelectionResolution, SelectionResolutionError,
};
use datum_gui_protocol::{AuthoredSelectionIdentity, CompoundSelection, SelectionSubject};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionReconciliation {
    pub project: datum_gui_protocol::SelectionProjectId,
    pub revision: SelectionModelRevision,
    pub subject: SelectionSubject,
    pub members: BTreeSet<AuthoredSelectionIdentity>,
    pub dropped: BTreeSet<AuthoredSelectionIdentity>,
    pub dissolved: bool,
}

/// Reconcile against a complete, fresh resolver projection. The caller replaces
/// consumer state only after this returns successfully; authority gaps cannot
/// partially reduce an existing selection.
pub fn reconcile_selection(
    subject: &SelectionSubject,
    selected_project: datum_gui_protocol::SelectionProjectId,
    resolution: &SelectionResolution,
) -> Result<SelectionReconciliation, SelectionResolutionError> {
    subject
        .validate()
        .map_err(SelectionResolutionError::InvalidSubject)?;
    let mut dropped = BTreeSet::new();
    let next = if selected_project != resolution.project {
        SelectionSubject::None
    } else {
        match subject {
            SelectionSubject::Object(id) if !resolution.authored.contains(id) => {
                dropped.insert(*id);
                SelectionSubject::None
            }
            SelectionSubject::Compound(value) => {
                dropped = value
                    .members
                    .difference(&resolution.authored)
                    .copied()
                    .collect();
                let members: BTreeSet<_> = value
                    .members
                    .intersection(&resolution.authored)
                    .copied()
                    .collect();
                if members.is_empty() {
                    SelectionSubject::None
                } else {
                    let focus = value.focus.filter(|id| members.contains(id));
                    // A singleton remains its current Compound kind; lifecycle
                    // does not manufacture an acquisition or promote focus.
                    SelectionSubject::Compound(CompoundSelection { members, focus })
                }
            }
            SelectionSubject::Run(origin) if !resolution.authored.contains(origin) => {
                SelectionSubject::None
            }
            SelectionSubject::GlobalNet(id)
                if !resolution
                    .nets
                    .as_ref()
                    .ok_or(SelectionResolutionError::MissingDerivedAuthority)?
                    .contains_key(id) =>
            {
                SelectionSubject::None
            }
            SelectionSubject::Bus(id)
                if !resolution
                    .buses
                    .as_ref()
                    .ok_or(SelectionResolutionError::MissingDerivedAuthority)?
                    .contains_key(id) =>
            {
                SelectionSubject::None
            }
            SelectionSubject::Proposal(id) if !resolution.proposals.contains(id) => {
                SelectionSubject::None
            }
            SelectionSubject::Review(id) if !resolution.reviews.contains(id) => {
                SelectionSubject::None
            }
            SelectionSubject::Diagnostic(id) if !resolution.diagnostics.contains(id) => {
                SelectionSubject::None
            }
            _ => subject.clone(),
        }
    };
    let members = resolution.members(&next)?;
    Ok(SelectionReconciliation {
        project: resolution.project,
        revision: resolution.revision.clone(),
        dissolved: !matches!(subject, SelectionSubject::None)
            && matches!(next, SelectionSubject::None),
        subject: next,
        members,
        dropped,
    })
}
