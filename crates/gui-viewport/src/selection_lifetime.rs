//! Shared revision reconciliation (PM026 / UVT 2.2.18 / PM054).
//! Consumer state only: no writer, scene reconstruction or visibility dependency.
use datum_gui_protocol::selection_resolution::{
    NativeSelectionResolution, PhysicalQueryFailure, SelectionAuthority, SelectionModelRevision,
    SelectionResolutionError, ZoneClearReason, ZoneRegionQualifier, ZoneRegionSuccessor,
};
use datum_gui_protocol::{AuthoredSelectionIdentity, CompoundSelection, SelectionSubject};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionReconciliationState {
    Current,
    RegionSplit {
        successors: Vec<ZoneRegionQualifier>,
    },
    RegionUnknown,
    RegionCleared(ZoneClearReason),
    Suspended {
        basis_revision: SelectionModelRevision,
        failure: PhysicalQueryFailure,
    },
}
impl SelectionReconciliationState {
    pub fn explanation(&self) -> Option<String> {
        match self {
            Self::Current => None,
            Self::RegionSplit { successors } => Some(format!(
                "Selected copper region split into {} components; selection cleared.",
                successors.len()
            )),
            Self::RegionUnknown => {
                Some("Copper region changed without certified lineage; reacquire a region.".into())
            }
            Self::RegionCleared(ZoneClearReason::Deleted) => {
                Some("Selected copper source was deleted; selection cleared.".into())
            }
            Self::RegionCleared(ZoneClearReason::CurrentEmpty) => {
                Some("Current fill contains no copper; selection cleared.".into())
            }
            Self::Suspended { .. } => Some(
                "Current copper is unavailable; previous selection is explicitly stale.".into(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionReconciliation {
    pub project: datum_gui_protocol::SelectionProjectId,
    pub revision: SelectionModelRevision,
    pub subject: SelectionSubject,
    pub members: BTreeSet<AuthoredSelectionIdentity>,
    pub dropped: BTreeSet<AuthoredSelectionIdentity>,
    pub dissolved: bool,
    pub state: SelectionReconciliationState,
}

/// The caller replaces consumer state only after a complete successful result.
/// A subject-local authority gap cannot silently prune the selected set.
pub fn reconcile_selection(
    subject: &SelectionSubject,
    selected_project: datum_gui_protocol::SelectionProjectId,
    resolution: &(impl SelectionAuthority + ?Sized),
) -> Result<SelectionReconciliation, SelectionResolutionError> {
    subject
        .validate()
        .map_err(SelectionResolutionError::InvalidSubject)?;
    let mut dropped = BTreeSet::new();
    let next = if selected_project != resolution.project() {
        SelectionSubject::None
    } else {
        match subject {
            SelectionSubject::Object(id) if !resolution.authored_contains(id) => {
                dropped.insert(id.clone());
                SelectionSubject::None
            }
            SelectionSubject::Compound(value) => {
                dropped = value
                    .members
                    .iter()
                    .filter(|id| !resolution.authored_contains(id))
                    .cloned()
                    .collect();
                let members: BTreeSet<_> = value
                    .members
                    .iter()
                    .filter(|id| resolution.authored_contains(id))
                    .cloned()
                    .collect();
                if members.is_empty() {
                    SelectionSubject::None
                } else {
                    let focus = value.focus.clone().filter(|id| members.contains(id));
                    SelectionSubject::Compound(CompoundSelection { members, focus })
                }
            }
            SelectionSubject::Run(run) if !resolution.authored_contains(&run.origin) => {
                SelectionSubject::None
            }
            SelectionSubject::GlobalNet(_) | SelectionSubject::Bus(_)
                if !resolution.derived_exists(subject)? =>
            {
                SelectionSubject::None
            }
            SelectionSubject::Proposal(_)
            | SelectionSubject::Review(_)
            | SelectionSubject::Diagnostic(_)
                if !resolution.artifact_exists(subject)? =>
            {
                SelectionSubject::None
            }
            _ => subject.clone(),
        }
    };
    let members = resolution.members(&next)?;
    Ok(SelectionReconciliation {
        project: resolution.project(),
        revision: resolution.revision().clone(),
        dissolved: !matches!(subject, SelectionSubject::None)
            && matches!(next, SelectionSubject::None),
        subject: next,
        members,
        dropped,
        state: SelectionReconciliationState::Current,
    })
}

/// Consume the engine's old/new-basis-bound Zone succession in this same shared
/// lifetime owner. No hit-point replay, arbitrary component choice or resurrection.
pub fn reconcile_native_selection(
    subject: &SelectionSubject,
    selected_project: datum_gui_protocol::SelectionProjectId,
    previous: &NativeSelectionResolution,
    current: &NativeSelectionResolution,
) -> Result<SelectionReconciliation, SelectionResolutionError> {
    subject
        .validate()
        .map_err(SelectionResolutionError::InvalidSubject)?;
    if selected_project != current.project() {
        return reconcile_selection(subject, selected_project, current);
    }
    let SelectionSubject::Run(run) = subject else {
        return reconcile_selection(subject, selected_project, current);
    };
    if run.zone_region.is_none() {
        return reconcile_selection(subject, selected_project, current);
    }
    match current.region_successor(previous, run)? {
        ZoneRegionSuccessor::Unique { qualifier, .. } => {
            let mut successor = run.clone();
            successor.zone_region = Some(qualifier);
            reconcile_selection(&SelectionSubject::Run(successor), selected_project, current)
        }
        ZoneRegionSuccessor::Suspended { failure } => Ok(SelectionReconciliation {
            project: current.project(),
            revision: current.revision().clone(),
            subject: subject.clone(),
            members: previous.members(subject)?,
            dropped: BTreeSet::new(),
            dissolved: false,
            state: SelectionReconciliationState::Suspended {
                basis_revision: previous.revision().clone(),
                failure,
            },
        }),
        disposition => {
            let state = match disposition {
                ZoneRegionSuccessor::Split { successors } => {
                    SelectionReconciliationState::RegionSplit { successors }
                }
                ZoneRegionSuccessor::Unknown => SelectionReconciliationState::RegionUnknown,
                ZoneRegionSuccessor::Cleared { reason } => {
                    SelectionReconciliationState::RegionCleared(reason)
                }
                _ => unreachable!(),
            };
            Ok(SelectionReconciliation {
                project: current.project(),
                revision: current.revision().clone(),
                subject: SelectionSubject::None,
                members: BTreeSet::new(),
                dropped: previous.members(subject)?,
                dissolved: true,
                state,
            })
        }
    }
}
