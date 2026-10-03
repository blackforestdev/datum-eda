//! One read-only acquisition reducer for clicks, menus, keyboard and regions.
//! Eligibility/parent collapse precede this boundary; full membership never does.
use datum_gui_protocol::selection_resolution::{SelectionAuthority, SelectionResolutionError};
use datum_gui_protocol::{AuthoredSelectionIdentity, CompoundSelection, SelectionSubject};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquisitionMode {
    Replace,
    Add,
    Remove,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquisitionEntry {
    Individual,
    Region,
    Global,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquisitionRefusal {
    pub mode: AcquisitionMode,
    pub subject_kind: &'static str,
}
impl AcquisitionRefusal {
    /// Shared visible and programmatic reason; announce once on completion.
    pub fn explanation(&self) -> String {
        if matches!(self.subject_kind, "proposal" | "review" | "check_finding") {
            return "Mixed authored/artifact and multi-artifact compounds are unavailable. Select without modifiers to replace the current subject.".into();
        }
        let verb = match self.mode {
            AcquisitionMode::Remove => "remove members from",
            _ => "add members to",
        };
        format!(
            "Cannot {verb} a {} selection. Membership follows connectivity. Select an object without modifiers to start an explicit selection.",
            self.subject_kind
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquisitionResult {
    pub subject: SelectionSubject,
    pub members: BTreeSet<AuthoredSelectionIdentity>,
    pub changed: bool,
    pub refusal: Option<AcquisitionRefusal>,
}

/// Qualified authored candidates only: no pad/pin/entry child acquisition,
/// visibility bypass or semantic expansion is introduced by this function.
/// The authority must be current at commit, including for a long-lived region.
/// No-op/refusal still refresh complete derived membership at that revision.
pub fn acquire_authored(
    prior: &SelectionSubject,
    candidates: BTreeSet<AuthoredSelectionIdentity>,
    mode: AcquisitionMode,
    entry: AcquisitionEntry,
    authority: &(impl SelectionAuthority + ?Sized),
) -> Result<AcquisitionResult, SelectionResolutionError> {
    prior
        .validate()
        .map_err(SelectionResolutionError::InvalidSubject)?;
    for candidate in &candidates {
        if !authority.authored_contains(candidate) {
            return Err(SelectionResolutionError::UnresolvedDerivedMember(
                candidate.clone(),
            ));
        }
    }
    if candidates.is_empty()
        && entry == AcquisitionEntry::Global
        && mode == AcquisitionMode::Replace
    {
        return Ok(AcquisitionResult {
            subject: SelectionSubject::None,
            members: BTreeSet::new(),
            changed: !matches!(prior, SelectionSubject::None),
            refusal: None,
        });
    }
    let members = if mode == AcquisitionMode::Replace && !candidates.is_empty() {
        BTreeSet::new()
    } else {
        authority.members(prior)?
    };
    // An empty canvas/region preserves the prior selection and focus.
    if candidates.is_empty() {
        return Ok(unchanged(prior, members, None));
    }
    if mode != AcquisitionMode::Replace
        && matches!(
            prior,
            SelectionSubject::Run(_) | SelectionSubject::GlobalNet(_) | SelectionSubject::Bus(_)
        )
    {
        let changes_membership = match mode {
            AcquisitionMode::Add => !candidates.is_subset(&members),
            AcquisitionMode::Remove => !candidates.is_disjoint(&members),
            AcquisitionMode::Replace => unreachable!(),
        };
        return Ok(unchanged(
            prior,
            members,
            changes_membership.then_some(AcquisitionRefusal {
                mode,
                subject_kind: prior.kind(),
            }),
        ));
    }
    if mode == AcquisitionMode::Add && datum_gui_protocol::selection_artifacts::is_artifact(prior) {
        return Ok(unchanged(
            prior,
            members,
            Some(AcquisitionRefusal {
                mode,
                subject_kind: prior.kind(),
            }),
        ));
    }
    let old_focus = match prior {
        SelectionSubject::Object(id) => Some(id.clone()),
        SelectionSubject::Compound(value) => value.focus.clone(),
        _ => None,
    };
    let (next_members, focus) = match mode {
        AcquisitionMode::Replace => {
            let focus = if entry == AcquisitionEntry::Individual && candidates.len() == 1 {
                candidates.first().cloned()
            } else {
                None
            };
            (candidates, focus)
        }
        AcquisitionMode::Add => {
            let focus = if entry == AcquisitionEntry::Individual && candidates.len() == 1 {
                candidates.first().cloned()
            } else {
                old_focus
            };
            (members.union(&candidates).cloned().collect(), focus)
        }
        AcquisitionMode::Remove => {
            let next: BTreeSet<_> = members.difference(&candidates).cloned().collect();
            let focus = old_focus.filter(|id| next.contains(id));
            (next, focus)
        }
    };
    // A subtractive no-op must also preserve non-authored subjects.
    if mode == AcquisitionMode::Remove && next_members == members {
        return Ok(unchanged(prior, members, None));
    }
    let subject = if next_members.is_empty() {
        SelectionSubject::None
    } else if next_members.len() == 1 && focus.is_some() {
        SelectionSubject::Object(focus.unwrap())
    } else {
        SelectionSubject::Compound(CompoundSelection {
            members: next_members.clone(),
            focus,
        })
    };
    Ok(AcquisitionResult {
        changed: subject != *prior,
        subject,
        members: next_members,
        refusal: None,
    })
}

fn unchanged(
    subject: &SelectionSubject,
    members: BTreeSet<AuthoredSelectionIdentity>,
    refusal: Option<AcquisitionRefusal>,
) -> AcquisitionResult {
    AcquisitionResult {
        subject: subject.clone(),
        members,
        changed: false,
        refusal,
    }
}

/// Explicit whole producing artifact acquisition. It shares the one modifier
/// law; a mixed artifact/authored or multi-artifact compound has no authority.
pub fn acquire_artifact(
    prior: &SelectionSubject,
    candidate: SelectionSubject,
    mode: AcquisitionMode,
    authority: &(impl SelectionAuthority + ?Sized),
) -> Result<AcquisitionResult, SelectionResolutionError> {
    candidate
        .validate()
        .map_err(SelectionResolutionError::InvalidSubject)?;
    if !datum_gui_protocol::selection_artifacts::is_artifact(&candidate)
        || !authority.artifact_exists(&candidate)?
    {
        return Err(SelectionResolutionError::MissingArtifactAuthority);
    }
    let members = authority.members(prior)?;
    if mode == AcquisitionMode::Replace
        || (mode == AcquisitionMode::Add && matches!(prior, SelectionSubject::None))
    {
        return Ok(AcquisitionResult {
            changed: candidate != *prior,
            subject: candidate,
            members: BTreeSet::new(),
            refusal: None,
        });
    }
    if mode == AcquisitionMode::Remove && *prior == candidate {
        return Ok(AcquisitionResult {
            changed: true,
            subject: SelectionSubject::None,
            members: BTreeSet::new(),
            refusal: None,
        });
    }
    Ok(unchanged(
        prior,
        members,
        (mode == AcquisitionMode::Add && *prior != candidate).then_some(AcquisitionRefusal {
            mode,
            subject_kind: candidate.kind(),
        }),
    ))
}
