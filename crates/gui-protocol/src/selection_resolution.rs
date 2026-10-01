//! Read-only adapter boundary for authoritative selection membership.
//!
//! Callers supply engine-resolved identities at one model revision. This module
//! never reconstructs connectivity from scene names, prefixes or pixel geometry.

use crate::selection_subject::{
    AuthoredSelectionIdentity, SelectionSubject, SelectionSubjectError,
};
pub use eda_engine::substrate::ModelRevision as SelectionModelRevision;
use eda_engine::substrate::ObjectId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
/// `None` derivations mean authority unavailable, not an empty complete result.
/// `Some(empty)` means the provider proved no identities exist at this revision.
pub struct SelectionResolution {
    pub project: ObjectId,
    pub revision: SelectionModelRevision,
    pub authored: BTreeSet<AuthoredSelectionIdentity>,
    pub runs: Option<BTreeMap<AuthoredSelectionIdentity, BTreeSet<AuthoredSelectionIdentity>>>,
    pub nets: Option<BTreeMap<ObjectId, BTreeSet<AuthoredSelectionIdentity>>>,
    pub buses: Option<BTreeMap<ObjectId, BTreeSet<AuthoredSelectionIdentity>>>,
    pub proposals: BTreeSet<String>,
    pub reviews: BTreeSet<String>,
    pub diagnostics: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionResolutionError {
    InvalidSubject(SelectionSubjectError),
    MissingDerivedAuthority,
    MissingDerivedIdentity,
    UnresolvedDerivedMember(AuthoredSelectionIdentity),
}

impl SelectionResolution {
    /// Resolve full membership before any visibility, class or pane projection.
    /// Missing authoritative derivation is an error, never an empty-set success.
    pub fn members(
        &self,
        subject: &SelectionSubject,
    ) -> Result<BTreeSet<AuthoredSelectionIdentity>, SelectionResolutionError> {
        subject
            .validate()
            .map_err(SelectionResolutionError::InvalidSubject)?;
        let derived = match subject {
            SelectionSubject::Run(origin) => self
                .runs
                .as_ref()
                .ok_or(SelectionResolutionError::MissingDerivedAuthority)?
                .get(origin),
            SelectionSubject::GlobalNet(id) => self
                .nets
                .as_ref()
                .ok_or(SelectionResolutionError::MissingDerivedAuthority)?
                .get(id),
            SelectionSubject::Bus(id) => self
                .buses
                .as_ref()
                .ok_or(SelectionResolutionError::MissingDerivedAuthority)?
                .get(id),
            SelectionSubject::Object(id) => {
                return Ok(self
                    .authored
                    .contains(id)
                    .then_some(*id)
                    .into_iter()
                    .collect());
            }
            SelectionSubject::Compound(value) => {
                return Ok(value
                    .members
                    .intersection(&self.authored)
                    .copied()
                    .collect());
            }
            _ => return Ok(BTreeSet::new()),
        }
        .ok_or(SelectionResolutionError::MissingDerivedIdentity)?;
        if let Some(id) = derived.difference(&self.authored).next() {
            return Err(SelectionResolutionError::UnresolvedDerivedMember(*id));
        }
        Ok(derived.clone())
    }
}
