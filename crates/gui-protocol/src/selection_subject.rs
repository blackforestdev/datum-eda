//! Shared selection identity vocabulary (PM026 / UVT 2.2.20).
//!
//! These are source identities, never scene-primitive keys or connectivity names.
//! Resolution and acquisition are separate: constructing a subject grants no
//! mutation or canvas-acquisition authority.

use eda_engine::substrate::ObjectId;

pub type SelectionProjectId = ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Transitional singleton consumed by the existing application while U1–U3
/// migrate entry, state and output together. It is not a second state store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionTarget {
    None,
    ReviewAction(String),
    AuthoredObject(String),
    CheckFinding(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthoredSelectionClass {
    Footprint,
    Pad,
    Track,
    Via,
    Zone,
    BoardText,
    BoardGraphic,
    BoardOutline,
    Symbol,
    Pin,
    Wire,
    BusSection,
    Label,
    Port,
    Junction,
    NoConnect,
    SchematicText,
    SchematicGraphic,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AuthoredSelectionIdentity {
    pub class: AuthoredSelectionClass,
    pub id: ObjectId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instance_path: Vec<ObjectId>,
}

impl AuthoredSelectionIdentity {
    pub fn can_originate_run(&self) -> bool {
        matches!(
            self.class,
            AuthoredSelectionClass::Track
                | AuthoredSelectionClass::Via
                | AuthoredSelectionClass::Zone
                | AuthoredSelectionClass::Wire
                | AuthoredSelectionClass::BusSection
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompoundSelection {
    pub members: BTreeSet<AuthoredSelectionIdentity>,
    pub focus: Option<AuthoredSelectionIdentity>,
}

impl CompoundSelection {
    pub fn new(
        members: impl IntoIterator<Item = AuthoredSelectionIdentity>,
        focus: Option<AuthoredSelectionIdentity>,
    ) -> Result<Self, SelectionSubjectError> {
        let value = Self {
            members: members.into_iter().collect(),
            focus,
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), SelectionSubjectError> {
        if self.members.is_empty() {
            return Err(SelectionSubjectError::EmptyCompound);
        }
        if self
            .focus
            .as_ref()
            .is_some_and(|id| !self.members.contains(id))
        {
            return Err(SelectionSubjectError::FocusOutsideMembership);
        }
        Ok(())
    }
}

/// A stable derivation origin. Zone qualification is engine-issued and bound to
/// captured source/model basis, never a retained hit point or polygon index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSelectionIdentity {
    pub origin: AuthoredSelectionIdentity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_region: Option<eda_engine::connectivity::ZoneRegionQualifier>,
}
impl From<AuthoredSelectionIdentity> for RunSelectionIdentity {
    fn from(origin: AuthoredSelectionIdentity) -> Self {
        Self {
            origin,
            zone_region: None,
        }
    }
}

/// Closed nine-kind selection vocabulary. Derived subjects contain identity,
/// not a cached list; complete membership belongs to revision-bound resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", content = "identity", rename_all = "snake_case")]
pub enum SelectionSubject {
    #[default]
    None,
    Object(AuthoredSelectionIdentity),
    Compound(CompoundSelection),
    Run(RunSelectionIdentity),
    GlobalNet(ObjectId),
    Bus(ObjectId),
    Proposal(String),
    Review(String),
    Diagnostic(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionSubjectError {
    EmptyCompound,
    FocusOutsideMembership,
    InvalidRunOrigin,
    InvalidRegionQualifier,
}

impl SelectionSubject {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Object(_) => "object",
            Self::Compound(_) => "compound",
            Self::Run(_) => "run",
            Self::GlobalNet(_) => "global_net",
            Self::Bus(_) => "bus",
            Self::Proposal(_) => "proposal",
            Self::Review(_) => "review",
            Self::Diagnostic(_) => "diagnostic",
        }
    }

    pub fn validate(&self) -> Result<(), SelectionSubjectError> {
        match self {
            Self::Compound(value) => value.validate(),
            Self::Run(run) if !run.origin.can_originate_run() => {
                Err(SelectionSubjectError::InvalidRunOrigin)
            }
            Self::Run(run)
                if run.zone_region.as_ref().is_some_and(|q| {
                    run.origin.class != AuthoredSelectionClass::Zone
                        || q.source().source_id != run.origin.id
                        || q.source().class != "zones"
                        || q.source().instance_path != run.origin.instance_path
                }) =>
            {
                Err(SelectionSubjectError::InvalidRegionQualifier)
            }
            _ => Ok(()),
        }
    }
}
