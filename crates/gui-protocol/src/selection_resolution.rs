//! Read-only adapter boundary for authoritative selection membership.
//!
//! Callers supply engine-resolved identities at one model revision. This module
//! never reconstructs connectivity from scene names, prefixes or pixel geometry.

mod native;
pub use eda_engine::connectivity::{
    PhysicalQueryFailure, ZoneClearReason, ZoneRegionQualifier, ZoneRegionSuccessor,
};
pub use native::{NativeSelectionProjection, NativeSelectionResolution};

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
    Engine(eda_engine::substrate::ElectricalQueryFailure),
    UnsupportedEngineClass(String),
    MissingArtifactAuthority,
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
            SelectionSubject::Run(run) => {
                if run.zone_region.is_some() {
                    return Err(SelectionResolutionError::MissingDerivedAuthority);
                }
                self.runs
                    .as_ref()
                    .ok_or(SelectionResolutionError::MissingDerivedAuthority)?
                    .get(&run.origin)
            }
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
                    .then_some(id.clone())
                    .into_iter()
                    .collect());
            }
            SelectionSubject::Compound(value) => {
                return Ok(value
                    .members
                    .intersection(&self.authored)
                    .cloned()
                    .collect());
            }
            _ => return Ok(BTreeSet::new()),
        }
        .ok_or(SelectionResolutionError::MissingDerivedIdentity)?;
        if let Some(id) = derived.difference(&self.authored).next() {
            return Err(SelectionResolutionError::UnresolvedDerivedMember(
                id.clone(),
            ));
        }
        Ok(derived.clone())
    }
}

/// One shared resolver boundary for revision lifecycle. Native implementations
/// resolve on demand from immutable engine authority; the table is a supplied
/// complete read projection, never a scene connectivity implementation.
pub trait SelectionAuthority {
    fn project(&self) -> ObjectId;
    fn revision(&self) -> &SelectionModelRevision;
    fn authored_contains(&self, id: &AuthoredSelectionIdentity) -> bool;
    fn derived_exists(&self, subject: &SelectionSubject) -> Result<bool, SelectionResolutionError>;
    fn artifact_exists(&self, subject: &SelectionSubject)
    -> Result<bool, SelectionResolutionError>;
    fn members(
        &self,
        subject: &SelectionSubject,
    ) -> Result<BTreeSet<AuthoredSelectionIdentity>, SelectionResolutionError>;
}
impl SelectionAuthority for SelectionResolution {
    fn project(&self) -> ObjectId {
        self.project
    }
    fn revision(&self) -> &SelectionModelRevision {
        &self.revision
    }
    fn authored_contains(&self, id: &AuthoredSelectionIdentity) -> bool {
        self.authored.contains(id)
    }
    fn derived_exists(&self, subject: &SelectionSubject) -> Result<bool, SelectionResolutionError> {
        let exists = match subject {
            SelectionSubject::GlobalNet(id) => self
                .nets
                .as_ref()
                .ok_or(SelectionResolutionError::MissingDerivedAuthority)?
                .contains_key(id),
            SelectionSubject::Bus(id) => self
                .buses
                .as_ref()
                .ok_or(SelectionResolutionError::MissingDerivedAuthority)?
                .contains_key(id),
            _ => true,
        };
        Ok(exists)
    }
    fn artifact_exists(
        &self,
        subject: &SelectionSubject,
    ) -> Result<bool, SelectionResolutionError> {
        Ok(match subject {
            SelectionSubject::Proposal(id) => self.proposals.contains(id),
            SelectionSubject::Review(id) => self.reviews.contains(id),
            SelectionSubject::Diagnostic(id) => self.diagnostics.contains(id),
            _ => true,
        })
    }
    fn members(
        &self,
        subject: &SelectionSubject,
    ) -> Result<BTreeSet<AuthoredSelectionIdentity>, SelectionResolutionError> {
        SelectionResolution::members(self, subject)
    }
}
