use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;

use super::{
    AuthorityRecord, AuthorityRecordBody, AuthorityResolution, AuthoritySnapshot,
    DeviationDepartureData, DeviationDepartureId, EngineeringChangeData, EngineeringChangeEvent,
    EngineeringChangeEventKind, EngineeringChangeId, LegacyRevisionFactMappingData,
    LegacyRevisionFactMappingId, RevisionAuthorityStore, RevisionReservationData,
    RevisionReservationId, RevisionReservationStanding, WaiverDepartureData, WaiverDepartureId,
    evaluate_change_transition, evaluate_reservation_availability,
    query_legacy_revision_fact_mapping, resolve_engineering_change, resolve_revision_reservation,
};

pub const REV_I04_MUTATIONS: &[&str] = &[
    "create_engineering_change",
    "set_change_rationale_and_classification",
    "add_or_update_affected_item",
    "set_change_effectivity",
    "link_implementation_transaction",
    "submit_change_for_impact_review",
    "authorize_change",
    "begin_change_implementation",
    "record_implementation_verification",
    "request_change_rework",
    "close_change",
    "reject_change",
    "defer_change",
    "cancel_change",
    "split_successor_work",
    "merge_successor_work",
    "reassign_successor_transactions",
    "reserve_revision_label",
    "release_revision_reservation",
    "expire_revision_reservation",
    "record_waiver_departure",
    "record_deviation_departure",
    "record_legacy_revision_fact_mapping",
];

pub const REV_I04_QUERIES: &[&str] = &[
    "resolve_engineering_change",
    "query_open_successor_work",
    "query_change_transaction_links",
    "evaluate_change_transition",
    "evaluate_design_mutation_authority",
    "resolve_revision_reservation",
    "evaluate_reservation_availability",
    "assess_legacy_revision_fact",
    "query_legacy_revision_fact_mapping",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RevI04MutationKind {
    CreateEngineeringChange,
    SetChangeRationaleAndClassification,
    AddOrUpdateAffectedItem,
    SetChangeEffectivity,
    LinkImplementationTransaction,
    SubmitChangeForImpactReview,
    AuthorizeChange,
    BeginChangeImplementation,
    RecordImplementationVerification,
    RequestChangeRework,
    CloseChange,
    RejectChange,
    DeferChange,
    CancelChange,
    SplitSuccessorWork,
    MergeSuccessorWork,
    ReassignSuccessorTransactions,
    ReserveRevisionLabel,
    ReleaseRevisionReservation,
    ExpireRevisionReservation,
    RecordWaiverDeparture,
    RecordDeviationDeparture,
    RecordLegacyRevisionFactMapping,
}

impl RevI04MutationKind {
    pub(crate) const ALL: &'static [Self] = &[
        Self::CreateEngineeringChange,
        Self::SetChangeRationaleAndClassification,
        Self::AddOrUpdateAffectedItem,
        Self::SetChangeEffectivity,
        Self::LinkImplementationTransaction,
        Self::SubmitChangeForImpactReview,
        Self::AuthorizeChange,
        Self::BeginChangeImplementation,
        Self::RecordImplementationVerification,
        Self::RequestChangeRework,
        Self::CloseChange,
        Self::RejectChange,
        Self::DeferChange,
        Self::CancelChange,
        Self::SplitSuccessorWork,
        Self::MergeSuccessorWork,
        Self::ReassignSuccessorTransactions,
        Self::ReserveRevisionLabel,
        Self::ReleaseRevisionReservation,
        Self::ExpireRevisionReservation,
        Self::RecordWaiverDeparture,
        Self::RecordDeviationDeparture,
        Self::RecordLegacyRevisionFactMapping,
    ];

    pub(crate) const fn wire_tag(self) -> &'static str {
        REV_I04_MUTATIONS[self as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RevI04QueryKind {
    ResolveEngineeringChange,
    QueryOpenSuccessorWork,
    QueryChangeTransactionLinks,
    EvaluateChangeTransition,
    EvaluateDesignMutationAuthority,
    ResolveRevisionReservation,
    EvaluateReservationAvailability,
    AssessLegacyRevisionFact,
    QueryLegacyRevisionFactMapping,
}

impl RevI04QueryKind {
    pub(crate) const ALL: &'static [Self] = &[
        Self::ResolveEngineeringChange,
        Self::QueryOpenSuccessorWork,
        Self::QueryChangeTransactionLinks,
        Self::EvaluateChangeTransition,
        Self::EvaluateDesignMutationAuthority,
        Self::ResolveRevisionReservation,
        Self::EvaluateReservationAvailability,
        Self::AssessLegacyRevisionFact,
        Self::QueryLegacyRevisionFactMapping,
    ];

    pub(crate) const fn wire_tag(self) -> &'static str {
        REV_I04_QUERIES[self as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevI04RefusalCode {
    ChangeTransitionInvalid,
    ChangeNotAuthorized,
    RevisionReservationConflict,
    DepartureInvalidOrExpired,
}

impl RevI04RefusalCode {
    pub const ALL: &'static [Self] = &[
        Self::ChangeTransitionInvalid,
        Self::ChangeNotAuthorized,
        Self::RevisionReservationConflict,
        Self::DepartureInvalidOrExpired,
    ];

    pub const fn wire_tag(self) -> &'static str {
        match self {
            Self::ChangeTransitionInvalid => "change_transition_invalid",
            Self::ChangeNotAuthorized => "change_not_authorized",
            Self::RevisionReservationConflict => "revision_reservation_conflict",
            Self::DepartureInvalidOrExpired => "departure_invalid_or_expired",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevI04Refusal {
    pub code: RevI04RefusalCode,
    pub affected_identities: Vec<String>,
    pub controlling_policy_or_requirement: String,
    pub expected_facts: Vec<String>,
    pub current_facts: Vec<String>,
    pub remediation: String,
}

impl RevI04Refusal {
    pub(crate) fn into_engine_error(self) -> EngineError {
        EngineError::Validation(format!(
            "{}: {}; remediation: {}",
            self.code.wire_tag(),
            self.current_facts.join(", "),
            self.remediation
        ))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChangeEventUpdate {
    pub current: EngineeringChangeId,
    pub next: EngineeringChangeId,
    pub event: EngineeringChangeEvent,
}

#[derive(Debug, Clone)]
pub(crate) struct SuccessorWorkUpdate {
    pub sources: Vec<EngineeringChangeId>,
    pub successors: Vec<AuthorityRecordBody<EngineeringChangeId, EngineeringChangeData>>,
}

#[derive(Debug, Clone)]
pub(crate) struct ReservationDispositionUpdate {
    pub current: RevisionReservationId,
    pub next: RevisionReservationId,
    pub standing: RevisionReservationStanding,
}

pub(crate) enum RevI04Mutation {
    CreateEngineeringChange(AuthorityRecordBody<EngineeringChangeId, EngineeringChangeData>),
    SetChangeRationaleAndClassification(ChangeEventUpdate),
    AddOrUpdateAffectedItem(ChangeEventUpdate),
    SetChangeEffectivity(ChangeEventUpdate),
    LinkImplementationTransaction(ChangeEventUpdate),
    SubmitChangeForImpactReview(ChangeEventUpdate),
    AuthorizeChange(ChangeEventUpdate),
    BeginChangeImplementation(ChangeEventUpdate),
    RecordImplementationVerification(ChangeEventUpdate),
    RequestChangeRework(ChangeEventUpdate),
    CloseChange(ChangeEventUpdate),
    RejectChange(ChangeEventUpdate),
    DeferChange(ChangeEventUpdate),
    CancelChange(ChangeEventUpdate),
    SplitSuccessorWork(SuccessorWorkUpdate),
    MergeSuccessorWork(SuccessorWorkUpdate),
    ReassignSuccessorTransactions(SuccessorWorkUpdate),
    ReserveRevisionLabel(AuthorityRecordBody<RevisionReservationId, RevisionReservationData>),
    ReleaseRevisionReservation(ReservationDispositionUpdate),
    ExpireRevisionReservation(ReservationDispositionUpdate),
    RecordWaiverDeparture(AuthorityRecordBody<WaiverDepartureId, WaiverDepartureData>),
    RecordDeviationDeparture(AuthorityRecordBody<DeviationDepartureId, DeviationDepartureData>),
    RecordLegacyRevisionFactMapping(
        AuthorityRecordBody<LegacyRevisionFactMappingId, LegacyRevisionFactMappingData>,
    ),
}

impl RevI04Mutation {
    pub(crate) const fn kind(&self) -> RevI04MutationKind {
        match self {
            Self::CreateEngineeringChange(_) => RevI04MutationKind::CreateEngineeringChange,
            Self::SetChangeRationaleAndClassification(_) => {
                RevI04MutationKind::SetChangeRationaleAndClassification
            }
            Self::AddOrUpdateAffectedItem(_) => RevI04MutationKind::AddOrUpdateAffectedItem,
            Self::SetChangeEffectivity(_) => RevI04MutationKind::SetChangeEffectivity,
            Self::LinkImplementationTransaction(_) => {
                RevI04MutationKind::LinkImplementationTransaction
            }
            Self::SubmitChangeForImpactReview(_) => RevI04MutationKind::SubmitChangeForImpactReview,
            Self::AuthorizeChange(_) => RevI04MutationKind::AuthorizeChange,
            Self::BeginChangeImplementation(_) => RevI04MutationKind::BeginChangeImplementation,
            Self::RecordImplementationVerification(_) => {
                RevI04MutationKind::RecordImplementationVerification
            }
            Self::RequestChangeRework(_) => RevI04MutationKind::RequestChangeRework,
            Self::CloseChange(_) => RevI04MutationKind::CloseChange,
            Self::RejectChange(_) => RevI04MutationKind::RejectChange,
            Self::DeferChange(_) => RevI04MutationKind::DeferChange,
            Self::CancelChange(_) => RevI04MutationKind::CancelChange,
            Self::SplitSuccessorWork(_) => RevI04MutationKind::SplitSuccessorWork,
            Self::MergeSuccessorWork(_) => RevI04MutationKind::MergeSuccessorWork,
            Self::ReassignSuccessorTransactions(_) => {
                RevI04MutationKind::ReassignSuccessorTransactions
            }
            Self::ReserveRevisionLabel(_) => RevI04MutationKind::ReserveRevisionLabel,
            Self::ReleaseRevisionReservation(_) => RevI04MutationKind::ReleaseRevisionReservation,
            Self::ExpireRevisionReservation(_) => RevI04MutationKind::ExpireRevisionReservation,
            Self::RecordWaiverDeparture(_) => RevI04MutationKind::RecordWaiverDeparture,
            Self::RecordDeviationDeparture(_) => RevI04MutationKind::RecordDeviationDeparture,
            Self::RecordLegacyRevisionFactMapping(_) => {
                RevI04MutationKind::RecordLegacyRevisionFactMapping
            }
        }
    }

    pub(crate) const fn wire_tag(&self) -> &'static str {
        self.kind().wire_tag()
    }
}

pub(crate) fn apply_rev_i04_mutations(
    store: &RevisionAuthorityStore,
    project_id: Uuid,
    mutations: Vec<RevI04Mutation>,
) -> Result<super::IntegrityHead, EngineError> {
    let head = store.read_head().transpose()?.ok_or_else(|| {
        EngineError::Validation(
            "revision authority mutation requires an accepted Project transaction".to_string(),
        )
    })?;
    let mut snapshot = match store.resolve_authority(project_id) {
        AuthorityResolution::Unconfigured => empty_snapshot(project_id),
        AuthorityResolution::Resolved { snapshot } => snapshot,
        AuthorityResolution::ReadOnlyDiagnostic { .. } => {
            return Err(EngineError::Validation(
                "revision authority is read-only; preserved unknown data cannot be rewritten"
                    .to_string(),
            ));
        }
    };
    apply_rev_i04_mutations_to_snapshot(&mut snapshot, mutations)?;
    store.commit_authority_snapshot(project_id, &head.integrity_root, &snapshot)
}

pub(crate) fn apply_rev_i04_mutations_to_snapshot(
    snapshot: &mut AuthoritySnapshot,
    mutations: Vec<RevI04Mutation>,
) -> Result<(), EngineError> {
    if mutations.is_empty() {
        return Err(EngineError::Validation(
            "REV-I04 mutation batch cannot be empty".to_string(),
        ));
    }
    for mutation in mutations {
        apply_one(snapshot, mutation)?;
    }
    let diagnostics = snapshot.validate();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(EngineError::Validation(format!(
            "REV-I04 authority mutation refused: {}",
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )))
    }
}

fn apply_one(
    snapshot: &mut AuthoritySnapshot,
    mutation: RevI04Mutation,
) -> Result<(), EngineError> {
    match mutation {
        RevI04Mutation::CreateEngineeringChange(body) => {
            super::transaction::append(snapshot, AuthorityRecord::EngineeringChange(body))
        }
        RevI04Mutation::SetChangeRationaleAndClassification(update) => {
            append_expected_change_event(
                snapshot,
                update,
                EngineeringChangeEventKind::RationaleAndClassificationSet,
            )
        }
        RevI04Mutation::AddOrUpdateAffectedItem(update) => append_expected_change_event(
            snapshot,
            update,
            EngineeringChangeEventKind::AffectedItemUpserted,
        ),
        RevI04Mutation::SetChangeEffectivity(update) => append_expected_change_event(
            snapshot,
            update,
            EngineeringChangeEventKind::EffectivitySet,
        ),
        RevI04Mutation::LinkImplementationTransaction(update) => append_expected_change_event(
            snapshot,
            update,
            EngineeringChangeEventKind::ImplementationTransactionLinked,
        ),
        RevI04Mutation::SubmitChangeForImpactReview(update) => append_expected_change_event(
            snapshot,
            update,
            EngineeringChangeEventKind::SubmittedForImpactReview,
        ),
        RevI04Mutation::AuthorizeChange(update) => {
            append_expected_change_event(snapshot, update, EngineeringChangeEventKind::Authorized)
        }
        RevI04Mutation::BeginChangeImplementation(update) => append_expected_change_event(
            snapshot,
            update,
            EngineeringChangeEventKind::ImplementationBegan,
        ),
        RevI04Mutation::RecordImplementationVerification(update) => append_expected_change_event(
            snapshot,
            update,
            EngineeringChangeEventKind::ImplementationVerificationRecorded,
        ),
        RevI04Mutation::RequestChangeRework(update) => append_expected_change_event(
            snapshot,
            update,
            EngineeringChangeEventKind::ReworkRequested,
        ),
        RevI04Mutation::CloseChange(update) => {
            append_expected_change_event(snapshot, update, EngineeringChangeEventKind::Closed)
        }
        RevI04Mutation::RejectChange(update) => {
            append_expected_change_event(snapshot, update, EngineeringChangeEventKind::Rejected)
        }
        RevI04Mutation::DeferChange(update) => {
            append_expected_change_event(snapshot, update, EngineeringChangeEventKind::Deferred)
        }
        RevI04Mutation::CancelChange(update) => {
            append_expected_change_event(snapshot, update, EngineeringChangeEventKind::Cancelled)
        }
        RevI04Mutation::SplitSuccessorWork(update)
        | RevI04Mutation::MergeSuccessorWork(update)
        | RevI04Mutation::ReassignSuccessorTransactions(update) => {
            apply_successor_update(snapshot, update)
        }
        RevI04Mutation::ReserveRevisionLabel(body) => {
            let data = &body.semantics;
            if !evaluate_reservation_availability(
                snapshot,
                data.configuration_item,
                data.scheme_id,
                data.scheme_version,
                &data.proposed_label,
            ) || data.standing != RevisionReservationStanding::Active
                || data.supersedes.is_some()
            {
                return Err(refusal(
                    RevI04RefusalCode::RevisionReservationConflict,
                    body.id.0,
                    "exact CI revision namespace",
                    "label must be unreserved and initial standing Active",
                ));
            }
            super::transaction::append(snapshot, AuthorityRecord::RevisionReservation(body))
        }
        RevI04Mutation::ReleaseRevisionReservation(update) => {
            append_expected_reservation_disposition(
                snapshot,
                update,
                RevisionReservationStanding::Released,
            )
        }
        RevI04Mutation::ExpireRevisionReservation(update) => {
            append_expected_reservation_disposition(
                snapshot,
                update,
                RevisionReservationStanding::Expired,
            )
        }
        RevI04Mutation::RecordWaiverDeparture(body) => {
            super::transaction::append(snapshot, AuthorityRecord::WaiverDeparture(body))
        }
        RevI04Mutation::RecordDeviationDeparture(body) => {
            super::transaction::append(snapshot, AuthorityRecord::DeviationDeparture(body))
        }
        RevI04Mutation::RecordLegacyRevisionFactMapping(body) => {
            if query_legacy_revision_fact_mapping(snapshot, body.semantics.source_fact_id).is_some()
            {
                return Err(refusal(
                    RevI04RefusalCode::DepartureInvalidOrExpired,
                    body.semantics.source_fact_id,
                    "one disposition per assessed legacy fact",
                    "source has already received an append-only disposition",
                ));
            }
            super::transaction::append(snapshot, AuthorityRecord::LegacyRevisionFactMapping(body))
        }
    }
}

fn append_change_event(
    snapshot: &mut AuthoritySnapshot,
    mut update: ChangeEventUpdate,
) -> Result<(), EngineError> {
    let current = resolve_engineering_change(snapshot, update.current).ok_or_else(|| {
        refusal(
            RevI04RefusalCode::ChangeTransitionInvalid,
            update.current.0,
            "EngineeringChange lifecycle",
            "governing Change does not resolve",
        )
    })?;
    if is_transition_event(update.event.kind)
        && !evaluate_change_transition(current.state, update.event.kind)
    {
        return Err(refusal(
            RevI04RefusalCode::ChangeTransitionInvalid,
            update.current.0,
            "EngineeringChange lifecycle",
            "event is not allowed from the current state",
        ));
    }
    update.event.sequence = current.data.events.len() as u64;
    let mut data = current.data;
    data.supersedes = Some(current.record_id);
    data.events.push(update.event);
    let references = collect_change_references(&data);
    super::transaction::append(
        snapshot,
        AuthorityRecord::EngineeringChange(AuthorityRecordBody {
            schema_version: super::AUTHORITY_SCHEMA_VERSION,
            id: update.next,
            project_id: snapshot.project_id,
            display_name: None,
            physical_locator: None,
            references,
            semantics: data,
        }),
    )
}

fn append_expected_change_event(
    snapshot: &mut AuthoritySnapshot,
    update: ChangeEventUpdate,
    expected: EngineeringChangeEventKind,
) -> Result<(), EngineError> {
    if update.event.kind != expected {
        return Err(refusal(
            RevI04RefusalCode::ChangeTransitionInvalid,
            update.current.0,
            "closed REV-I04 event vocabulary",
            "mutation and event kind do not match",
        ));
    }
    append_change_event(snapshot, update)
}

fn apply_successor_update(
    snapshot: &mut AuthoritySnapshot,
    update: SuccessorWorkUpdate,
) -> Result<(), EngineError> {
    if update.sources.is_empty() || update.successors.is_empty() {
        return Err(refusal(
            RevI04RefusalCode::ChangeTransitionInvalid,
            snapshot.project_id,
            "successor work accounting",
            "split, merge, or reassignment requires sources and successors",
        ));
    }
    for source in &update.sources {
        if resolve_engineering_change(snapshot, *source).is_none() {
            return Err(refusal(
                RevI04RefusalCode::ChangeTransitionInvalid,
                source.0,
                "successor work accounting",
                "source Change does not resolve",
            ));
        }
    }
    for successor in update.successors {
        if successor.semantics.predecessor_baseline.is_none()
            || successor.semantics.configuration_item.is_none()
            || successor.semantics.events.first().map(|event| event.kind)
                != Some(EngineeringChangeEventKind::Created)
        {
            return Err(refusal(
                RevI04RefusalCode::ChangeTransitionInvalid,
                successor.id.0,
                "successor work accounting",
                "successor must preserve exact predecessor and begin with Created",
            ));
        }
        super::transaction::append(snapshot, AuthorityRecord::EngineeringChange(successor))?;
    }
    Ok(())
}

fn append_reservation_disposition(
    snapshot: &mut AuthoritySnapshot,
    update: ReservationDispositionUpdate,
) -> Result<(), EngineError> {
    if !matches!(
        update.standing,
        RevisionReservationStanding::Released | RevisionReservationStanding::Expired
    ) {
        return Err(refusal(
            RevI04RefusalCode::RevisionReservationConflict,
            update.current.0,
            "reservation standing",
            "only release or expiry may dispose an active reservation",
        ));
    }
    let current = resolve_revision_reservation(snapshot, update.current).ok_or_else(|| {
        refusal(
            RevI04RefusalCode::RevisionReservationConflict,
            update.current.0,
            "reservation standing",
            "reservation does not resolve",
        )
    })?;
    if current.data.standing != RevisionReservationStanding::Active {
        return Err(refusal(
            RevI04RefusalCode::RevisionReservationConflict,
            update.current.0,
            "reservation standing",
            "only an active reservation may be disposed",
        ));
    }
    let mut data = current.data;
    data.supersedes = Some(current.record_id);
    data.standing = update.standing;
    super::transaction::append(
        snapshot,
        AuthorityRecord::RevisionReservation(AuthorityRecordBody {
            schema_version: super::AUTHORITY_SCHEMA_VERSION,
            id: update.next,
            project_id: snapshot.project_id,
            display_name: None,
            physical_locator: None,
            references: vec![super::AuthorityRef::EngineeringChange(
                data.governing_change,
            )],
            semantics: data,
        }),
    )
}

fn append_expected_reservation_disposition(
    snapshot: &mut AuthoritySnapshot,
    update: ReservationDispositionUpdate,
    expected: RevisionReservationStanding,
) -> Result<(), EngineError> {
    if update.standing != expected {
        return Err(refusal(
            RevI04RefusalCode::RevisionReservationConflict,
            update.current.0,
            "closed reservation standing transition",
            "mutation and requested standing do not match",
        ));
    }
    append_reservation_disposition(snapshot, update)
}

fn collect_change_references(data: &EngineeringChangeData) -> Vec<super::AuthorityRef> {
    let mut references = Vec::new();
    if let Some(id) = data.predecessor_baseline {
        references.push(super::AuthorityRef::ConfigurationBaseline(id));
    }
    if let Some(id) = data.configuration_item {
        references.push(super::AuthorityRef::ConfigurationItem(id));
    }
    for event in &data.events {
        if let Some(item) = &event.affected_item {
            references.push(item.item);
            references.extend(item.before);
            references.extend(item.proposed_after);
            references.extend(item.effectivity.map(super::AuthorityRef::Effectivity));
        }
        references.extend(event.effectivity.map(super::AuthorityRef::Effectivity));
        if let Some(super::ChangeClosureKind::ReleasedSuccessorReference(reference)) = event.closure
        {
            references.push(reference);
        }
    }
    references.sort();
    references.dedup();
    references
}

fn is_transition_event(kind: EngineeringChangeEventKind) -> bool {
    !matches!(
        kind,
        EngineeringChangeEventKind::Created
            | EngineeringChangeEventKind::RationaleAndClassificationSet
            | EngineeringChangeEventKind::AffectedItemUpserted
            | EngineeringChangeEventKind::EffectivitySet
            | EngineeringChangeEventKind::ImplementationTransactionLinked
    )
}

fn refusal(
    code: RevI04RefusalCode,
    identity: Uuid,
    controlling: &str,
    current: &str,
) -> EngineError {
    RevI04Refusal {
        code,
        affected_identities: vec![identity.to_string()],
        controlling_policy_or_requirement: controlling.to_string(),
        expected_facts: vec!["closed REV-I04 contract".to_string()],
        current_facts: vec![current.to_string()],
        remediation: "supply the exact required authority or choose an authorized transition"
            .to_string(),
    }
    .into_engine_error()
}

fn empty_snapshot(project_id: Uuid) -> AuthoritySnapshot {
    AuthoritySnapshot {
        schema_version: super::AUTHORITY_SCHEMA_VERSION,
        project_id,
        records: Vec::new(),
        events: Vec::new(),
        opaque_records: Vec::new(),
        opaque_events: Vec::new(),
    }
}
