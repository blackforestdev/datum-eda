use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    AuthorityDiagnostic, AuthorityRecord, AuthorityRef, AuthoritySnapshot, EffectivityId,
    EngineeringChangeId,
};

pub const ENGINEERING_CHANGE_STATES: &[EngineeringChangeState] = &[
    EngineeringChangeState::Draft,
    EngineeringChangeState::ImpactReview,
    EngineeringChangeState::Authorized,
    EngineeringChangeState::Implementing,
    EngineeringChangeState::Verification,
    EngineeringChangeState::Closed,
    EngineeringChangeState::Rejected,
    EngineeringChangeState::Deferred,
    EngineeringChangeState::Cancelled,
];

pub const ENGINEERING_CHANGE_EVENT_KINDS: &[EngineeringChangeEventKind] = &[
    EngineeringChangeEventKind::Created,
    EngineeringChangeEventKind::RationaleAndClassificationSet,
    EngineeringChangeEventKind::AffectedItemUpserted,
    EngineeringChangeEventKind::EffectivitySet,
    EngineeringChangeEventKind::ImplementationTransactionLinked,
    EngineeringChangeEventKind::SubmittedForImpactReview,
    EngineeringChangeEventKind::Authorized,
    EngineeringChangeEventKind::ImplementationBegan,
    EngineeringChangeEventKind::ImplementationVerificationRecorded,
    EngineeringChangeEventKind::Closed,
    EngineeringChangeEventKind::Rejected,
    EngineeringChangeEventKind::Deferred,
    EngineeringChangeEventKind::Cancelled,
    EngineeringChangeEventKind::ReworkRequested,
];

pub const AFFECTED_ITEM_ACTIONS: &[AffectedItemAction] = &[
    AffectedItemAction::Add,
    AffectedItemAction::Modify,
    AffectedItemAction::Retire,
    AffectedItemAction::NoChange,
];

pub const CHANGE_CLOSURE_KINDS: &[&str] =
    &["released_successor_reference", "no_release_disposition"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineeringChangeState {
    Draft,
    ImpactReview,
    Authorized,
    Implementing,
    Verification,
    Closed,
    Rejected,
    Deferred,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineeringChangeEventKind {
    Created,
    RationaleAndClassificationSet,
    AffectedItemUpserted,
    EffectivitySet,
    ImplementationTransactionLinked,
    SubmittedForImpactReview,
    Authorized,
    ImplementationBegan,
    ImplementationVerificationRecorded,
    Closed,
    Rejected,
    Deferred,
    Cancelled,
    ReworkRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AffectedItemAction {
    Add,
    Modify,
    Retire,
    NoChange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AffectedItem {
    pub item: AuthorityRef,
    pub action: AffectedItemAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<AuthorityRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_after: Option<AuthorityRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effectivity: Option<EffectivityId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeClosureKind {
    ReleasedSuccessorReference(AuthorityRef),
    NoReleaseDisposition(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineeringChangeEvent {
    pub kind: EngineeringChangeEventKind,
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affected_item: Option<AffectedItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effectivity: Option<EffectivityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closure: Option<ChangeClosureKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineeringChangeData {
    pub change_key: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<EngineeringChangeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_baseline: Option<super::ConfigurationBaselineId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration_item: Option<super::ConfigurationItemId>,
    pub events: Vec<EngineeringChangeEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEngineeringChange {
    pub record_id: EngineeringChangeId,
    pub data: EngineeringChangeData,
    pub state: EngineeringChangeState,
}

pub fn engineering_change_state(
    events: &[EngineeringChangeEvent],
) -> Option<EngineeringChangeState> {
    let mut state = None;
    for event in events {
        state = match event.kind {
            EngineeringChangeEventKind::Created => Some(EngineeringChangeState::Draft),
            EngineeringChangeEventKind::SubmittedForImpactReview => {
                Some(EngineeringChangeState::ImpactReview)
            }
            EngineeringChangeEventKind::Authorized => Some(EngineeringChangeState::Authorized),
            EngineeringChangeEventKind::ImplementationBegan => {
                Some(EngineeringChangeState::Implementing)
            }
            EngineeringChangeEventKind::ImplementationVerificationRecorded => {
                Some(EngineeringChangeState::Verification)
            }
            EngineeringChangeEventKind::Closed => Some(EngineeringChangeState::Closed),
            EngineeringChangeEventKind::Rejected => Some(EngineeringChangeState::Rejected),
            EngineeringChangeEventKind::Deferred => Some(EngineeringChangeState::Deferred),
            EngineeringChangeEventKind::Cancelled => Some(EngineeringChangeState::Cancelled),
            EngineeringChangeEventKind::ReworkRequested => match state {
                Some(EngineeringChangeState::ImpactReview) => Some(EngineeringChangeState::Draft),
                Some(EngineeringChangeState::Authorized) => {
                    Some(EngineeringChangeState::ImpactReview)
                }
                Some(EngineeringChangeState::Implementing) => {
                    Some(EngineeringChangeState::Authorized)
                }
                Some(EngineeringChangeState::Verification) => {
                    Some(EngineeringChangeState::Implementing)
                }
                other => other,
            },
            EngineeringChangeEventKind::RationaleAndClassificationSet
            | EngineeringChangeEventKind::AffectedItemUpserted
            | EngineeringChangeEventKind::EffectivitySet
            | EngineeringChangeEventKind::ImplementationTransactionLinked => state,
        };
    }
    state
}

pub fn evaluate_change_transition(
    state: EngineeringChangeState,
    event: EngineeringChangeEventKind,
) -> bool {
    use EngineeringChangeEventKind as E;
    use EngineeringChangeState as S;
    matches!(
        (state, event),
        (S::Draft, E::SubmittedForImpactReview)
            | (S::ImpactReview, E::Authorized)
            | (S::Authorized, E::ImplementationBegan)
            | (S::Implementing, E::ImplementationVerificationRecorded)
            | (S::Verification, E::Closed)
            | (
                S::Draft | S::ImpactReview | S::Authorized | S::Implementing | S::Verification,
                E::Rejected | E::Deferred | E::Cancelled
            )
            | (
                S::ImpactReview | S::Authorized | S::Implementing | S::Verification,
                E::ReworkRequested
            )
    )
}

pub fn resolve_engineering_change(
    snapshot: &AuthoritySnapshot,
    id: EngineeringChangeId,
) -> Option<ResolvedEngineeringChange> {
    let first = snapshot.records.iter().find_map(|record| match record {
        AuthorityRecord::EngineeringChange(body) if body.id == id => Some(body),
        _ => None,
    })?;
    let key = first.semantics.change_key;
    let superseded: BTreeSet<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::EngineeringChange(body) if body.semantics.change_key == key => {
                body.semantics.supersedes
            }
            _ => None,
        })
        .collect();
    let active = snapshot.records.iter().find_map(|record| match record {
        AuthorityRecord::EngineeringChange(body)
            if body.semantics.change_key == key && !superseded.contains(&body.id) =>
        {
            Some(body)
        }
        _ => None,
    })?;
    Some(ResolvedEngineeringChange {
        record_id: active.id,
        data: active.semantics.clone(),
        state: engineering_change_state(&active.semantics.events)?,
    })
}

pub fn query_open_successor_work(
    snapshot: &AuthoritySnapshot,
    predecessor: super::ConfigurationBaselineId,
    item: super::ConfigurationItemId,
) -> Option<ResolvedEngineeringChange> {
    snapshot.records.iter().find_map(|record| match record {
        AuthorityRecord::EngineeringChange(body)
            if body.semantics.predecessor_baseline == Some(predecessor)
                && body.semantics.configuration_item == Some(item) =>
        {
            let resolved = resolve_engineering_change(snapshot, body.id)?;
            (!matches!(
                resolved.state,
                EngineeringChangeState::Closed
                    | EngineeringChangeState::Rejected
                    | EngineeringChangeState::Cancelled
            ))
            .then_some(resolved)
        }
        _ => None,
    })
}

pub fn query_change_transaction_links(
    snapshot: &AuthoritySnapshot,
    id: EngineeringChangeId,
) -> Vec<Uuid> {
    let mut links = resolve_engineering_change(snapshot, id)
        .map(|resolved| {
            resolved
                .data
                .events
                .into_iter()
                .filter(|event| {
                    event.kind == EngineeringChangeEventKind::ImplementationTransactionLinked
                })
                .filter_map(|event| event.transaction_id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut seen = BTreeSet::new();
    links.retain(|transaction_id| seen.insert(*transaction_id));
    links
}

pub(crate) fn validate_engineering_change(
    data: &EngineeringChangeData,
) -> Vec<AuthorityDiagnostic> {
    let mut diagnostics = Vec::new();
    if data.events.first().map(|event| event.kind) != Some(EngineeringChangeEventKind::Created)
        || data
            .events
            .iter()
            .enumerate()
            .any(|(index, event)| event.sequence != index as u64)
    {
        diagnostics.push(diag(
            "revision_change_event_chain_invalid",
            "EngineeringChange events must begin with Created and have contiguous sequence",
        ));
        return diagnostics;
    }
    let mut state = EngineeringChangeState::Draft;
    for event in data.events.iter().skip(1) {
        if is_transition_event(event.kind) && !evaluate_change_transition(state, event.kind) {
            diagnostics.push(diag(
                "revision_change_transition_invalid",
                "EngineeringChange event is not permitted from the projected state",
            ));
            break;
        }
        state = engineering_change_state(&data.events[..=event.sequence as usize]).unwrap_or(state);
    }
    diagnostics
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

fn diag(code: &str, message: &str) -> AuthorityDiagnostic {
    AuthorityDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}
