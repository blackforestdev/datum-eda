//! Authored electrical identity; graph/member results remain derived.
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ModelRevision, ObjectRevision, RevisionedRef};

pub const ELECTRICAL_IDENTITY_SCHEMA_VERSION: u64 = 1;

/// Stable source plus hierarchy context, ordered independently of display names.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElectricalOccurrence {
    pub class: String,
    pub source_id: Uuid,
    pub instance_path: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElectricalIdentityRecord {
    pub id: Uuid,
    pub object_revision: ObjectRevision,
    pub identity: ElectricalIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ElectricalIdentity {
    Net {
        anchor: ElectricalOccurrence,
        anchor_reason: NetAnchorReason,
        retired: bool,
        /// Recorded transition provenance, not current graph authority.
        predecessors: BTreeSet<Uuid>,
    },
    Bus {
        name: String,
        scalar_nets: BTreeSet<Uuid>,
        representations: BTreeSet<ElectricalOccurrence>,
        retired: bool,
    },
    NetRelationship {
        logical_net: Uuid,
        board_net: Option<RevisionedRef>,
        intent: NetRelationshipIntent,
        evidence: Vec<NetCorrespondence>,
    },
    BusInterface {
        left_bus: Uuid,
        right_bus: Uuid,
        left_occurrence: ElectricalOccurrence,
        right_occurrence: ElectricalOccurrence,
        equivalence: BusInterfaceEquivalence,
        scalar_mapping: BTreeSet<(Uuid, Uuid)>,
    },
}

/// Why the recorded transaction chose this anchor; never inferred on reopen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetAnchorReason {
    Authored,
    UnclaimedFinalGroup,
    SurvivingAnchor,
    DeletedAnchorFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetRelationshipIntent {
    Implemented,
    BoardOnly,
    SchematicOnly,
    Pending,
    Mismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusInterfaceEquivalence {
    Identity,
    Related,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetCorrespondence {
    pub component_instance: RevisionedRef,
    pub pin_pad_map: RevisionedRef,
    pub schematic_terminal: ElectricalOccurrence,
    pub board_pad: RevisionedRef,
    /// Explicit PinPadMap cell reference; no source-pad/name inference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_pad: Option<RevisionedRef>,
    /// Explicit legacy terminal adoption; an absent value needs source provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_pin: Option<RevisionedRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElectricalIdentityShard {
    pub schema_version: u64,
    pub record: ElectricalIdentityRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetCorrespondenceStatus {
    /// Explicit intent and (for Implemented) certified pin/pad correspondence.
    /// This is not a complete derived Global Net membership result.
    Complete,
    /// Implemented intent exists, but placed-terminal/library lineage has not
    /// been certified. Never expose a complete cross-domain selection from this.
    Unverified,
    Pending,
    Mismatch,
    Stale,
    Absent,
}

/// Immutable engine-derived pre-batch group, never a source member list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviousNetGroup {
    pub net_id: Uuid,
    pub anchor: ElectricalOccurrence,
    pub members: BTreeSet<ElectricalOccurrence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetIdentityTransition {
    pub source_revision: ModelRevision,
    pub final_groups: Vec<ResolvedNetIdentity>,
    pub retired: BTreeSet<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedNetIdentity {
    pub net_id: Uuid,
    pub anchor: ElectricalOccurrence,
    pub anchor_reason: NetAnchorReason,
    pub members: BTreeSet<ElectricalOccurrence>,
    pub predecessors: BTreeSet<Uuid>,
    pub allocated: bool,
}
