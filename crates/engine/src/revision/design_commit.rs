use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;

use super::{
    AuthorityRecord, AuthorityRecordBody, AuthorityRef, AuthorityResolution, AuthoritySnapshot,
    ConfigurationBaselineId, ConfigurationItemId, EarlierControlMode, EngineeringChangeData,
    EngineeringChangeEvent, EngineeringChangeEventKind, EngineeringChangeId,
    EngineeringChangeState, ProjectRevisionPolicyResolution, RevI04Mutation,
    RevisionAuthorityStore, apply_rev_i04_mutations_to_snapshot, query_change_transaction_links,
    query_open_successor_work, resolve_engineering_change, resolve_project_revision_policy,
};

pub const REVISION_DESIGN_COMMIT_CONTEXTS: &[RevisionDesignCommitContext] = &[
    RevisionDesignCommitContext::Normal,
    RevisionDesignCommitContext::AcceptedProposalApply,
    RevisionDesignCommitContext::Undo,
    RevisionDesignCommitContext::Redo,
];

pub const REVISION_DESIGN_COMMIT_OUTCOMES: &[&str] = &[
    "unmanaged_pass_through",
    "no_earlier_control_collect",
    "authorized_change_validated",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionDesignCommitContext {
    Normal,
    AcceptedProposalApply,
    Undo,
    Redo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SuccessorCollectionContext {
    pub predecessor_baseline: ConfigurationBaselineId,
    pub configuration_item: ConfigurationItemId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RevisionDesignCommitInput {
    pub governing_change: Option<EngineeringChangeId>,
    pub affected_scope: Vec<AuthorityRef>,
    pub successor: Option<SuccessorCollectionContext>,
    pub related_transaction: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DesignMutationAuthorityEvaluation {
    Authorized(EngineeringChangeId),
    Refused,
}

#[derive(Debug, Clone)]
pub(crate) enum RevisionDesignCommitPlan {
    UnmanagedPassThrough,
    NoEarlierControlCollect {
        snapshot: AuthoritySnapshot,
        successor: Option<SuccessorCollectionContext>,
    },
    AuthorizedChangeValidated {
        snapshot: AuthoritySnapshot,
        governing_change: EngineeringChangeId,
    },
}

impl RevisionDesignCommitPlan {
    pub(crate) const fn wire_tag(&self) -> &'static str {
        match self {
            Self::UnmanagedPassThrough => REVISION_DESIGN_COMMIT_OUTCOMES[0],
            Self::NoEarlierControlCollect { .. } => REVISION_DESIGN_COMMIT_OUTCOMES[1],
            Self::AuthorizedChangeValidated { .. } => REVISION_DESIGN_COMMIT_OUTCOMES[2],
        }
    }
}

pub(crate) fn prepare_revision_design_commit(
    store: &RevisionAuthorityStore,
    project_id: Uuid,
    _context: RevisionDesignCommitContext,
    input: &RevisionDesignCommitInput,
) -> Result<RevisionDesignCommitPlan, EngineError> {
    let snapshot = match store.resolve_authority(project_id) {
        AuthorityResolution::Unconfigured => {
            return Ok(RevisionDesignCommitPlan::UnmanagedPassThrough);
        }
        AuthorityResolution::Resolved { snapshot } => snapshot,
        AuthorityResolution::ReadOnlyDiagnostic { diagnostics, .. } => {
            return Err(EngineError::Validation(format!(
                "change_not_authorized: revision authority is read-only: {}",
                diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.code.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            )));
        }
    };
    let policy = match resolve_project_revision_policy(Some(&snapshot)) {
        ProjectRevisionPolicyResolution::Unmanaged => {
            return Ok(RevisionDesignCommitPlan::UnmanagedPassThrough);
        }
        ProjectRevisionPolicyResolution::Managed { policy, .. } => policy,
        ProjectRevisionPolicyResolution::Refused(diagnostic) => {
            return Err(EngineError::Validation(format!(
                "change_not_authorized: {}",
                diagnostic.code
            )));
        }
    };
    match policy.earlier_control {
        EarlierControlMode::NoEarlierControl => {
            Ok(RevisionDesignCommitPlan::NoEarlierControlCollect {
                snapshot,
                successor: input.successor,
            })
        }
        EarlierControlMode::AuthorizedChangeRequired => {
            let governing_change = input.governing_change.or_else(|| {
                input.related_transaction.and_then(|transaction_id| {
                    snapshot.records.iter().find_map(|record| match record {
                        AuthorityRecord::EngineeringChange(body)
                            if query_change_transaction_links(&snapshot, body.id)
                                .contains(&transaction_id) =>
                        {
                            Some(body.id)
                        }
                        _ => None,
                    })
                })
            });
            match evaluate_design_mutation_authority(
                &snapshot,
                governing_change,
                &input.affected_scope,
            ) {
                DesignMutationAuthorityEvaluation::Authorized(governing_change) => {
                    Ok(RevisionDesignCommitPlan::AuthorizedChangeValidated {
                        snapshot,
                        governing_change,
                    })
                }
                DesignMutationAuthorityEvaluation::Refused => Err(EngineError::Validation(
                    "change_not_authorized: supply the exact Authorized or Implementing governing Change for the affected scope".to_string(),
                )),
            }
        }
    }
}

pub(crate) fn evaluate_design_mutation_authority(
    snapshot: &AuthoritySnapshot,
    governing_change: Option<EngineeringChangeId>,
    affected_scope: &[AuthorityRef],
) -> DesignMutationAuthorityEvaluation {
    let Some(requested) = governing_change else {
        return DesignMutationAuthorityEvaluation::Refused;
    };
    let Some(change) = resolve_engineering_change(snapshot, requested) else {
        return DesignMutationAuthorityEvaluation::Refused;
    };
    if !matches!(
        change.state,
        EngineeringChangeState::Authorized | EngineeringChangeState::Implementing
    ) {
        return DesignMutationAuthorityEvaluation::Refused;
    }
    let declared: Vec<_> = change
        .data
        .events
        .iter()
        .filter_map(|event| event.affected_item.as_ref().map(|item| item.item))
        .collect();
    if affected_scope.iter().all(|item| declared.contains(item)) {
        DesignMutationAuthorityEvaluation::Authorized(change.record_id)
    } else {
        DesignMutationAuthorityEvaluation::Refused
    }
}

pub(crate) fn finalize_revision_design_commit_plan(
    plan: RevisionDesignCommitPlan,
    project_id: Uuid,
    transaction_id: Uuid,
) -> Result<Option<AuthoritySnapshot>, EngineError> {
    match plan {
        RevisionDesignCommitPlan::UnmanagedPassThrough => Ok(None),
        RevisionDesignCommitPlan::NoEarlierControlCollect {
            mut snapshot,
            successor,
        } => {
            let Some(successor) = successor.filter(|context| {
                snapshot
                    .record(AuthorityRef::ConfigurationBaseline(
                        context.predecessor_baseline,
                    ))
                    .is_some()
                    && snapshot
                        .record(AuthorityRef::ConfigurationItem(context.configuration_item))
                        .is_some()
            }) else {
                return Ok(Some(snapshot));
            };
            let existing = query_open_successor_work(
                &snapshot,
                successor.predecessor_baseline,
                successor.configuration_item,
            );
            let change_id = existing.as_ref().map_or_else(
                || deterministic_change_id(project_id, successor),
                |change| change.record_id,
            );
            let mut mutations = Vec::new();
            if existing.is_none() {
                mutations.push(RevI04Mutation::CreateEngineeringChange(
                    AuthorityRecordBody {
                        schema_version: super::AUTHORITY_SCHEMA_VERSION,
                        id: change_id,
                        project_id,
                        display_name: None,
                        physical_locator: None,
                        references: vec![
                            AuthorityRef::ConfigurationItem(successor.configuration_item),
                            AuthorityRef::ConfigurationBaseline(successor.predecessor_baseline),
                        ],
                        semantics: EngineeringChangeData {
                            change_key: change_id.0,
                            supersedes: None,
                            predecessor_baseline: Some(successor.predecessor_baseline),
                            configuration_item: Some(successor.configuration_item),
                            events: vec![event(EngineeringChangeEventKind::Created, 0, None)],
                        },
                    },
                ));
            }
            mutations.push(link_mutation(change_id, project_id, transaction_id));
            apply_rev_i04_mutations_to_snapshot(&mut snapshot, mutations)?;
            Ok(Some(snapshot))
        }
        RevisionDesignCommitPlan::AuthorizedChangeValidated {
            mut snapshot,
            governing_change,
        } => {
            let active = resolve_engineering_change(&snapshot, governing_change)
                .ok_or_else(|| EngineError::Validation("change_not_authorized".to_string()))?;
            apply_rev_i04_mutations_to_snapshot(
                &mut snapshot,
                vec![link_mutation(active.record_id, project_id, transaction_id)],
            )?;
            Ok(Some(snapshot))
        }
    }
}

fn link_mutation(
    current: EngineeringChangeId,
    project_id: Uuid,
    transaction_id: Uuid,
) -> RevI04Mutation {
    let next = EngineeringChangeId(Uuid::new_v5(
        &project_id,
        format!("datum-eda:change-link:{}:{}", current.0, transaction_id).as_bytes(),
    ));
    RevI04Mutation::LinkImplementationTransaction(super::change_transaction::ChangeEventUpdate {
        current,
        next,
        event: event(
            EngineeringChangeEventKind::ImplementationTransactionLinked,
            0,
            Some(transaction_id),
        ),
    })
}

fn deterministic_change_id(
    project_id: Uuid,
    successor: SuccessorCollectionContext,
) -> EngineeringChangeId {
    EngineeringChangeId(Uuid::new_v5(
        &project_id,
        format!(
            "datum-eda:quiet-successor:{}:{}",
            successor.predecessor_baseline.0, successor.configuration_item.0
        )
        .as_bytes(),
    ))
}

fn event(
    kind: EngineeringChangeEventKind,
    sequence: u64,
    transaction_id: Option<Uuid>,
) -> EngineeringChangeEvent {
    EngineeringChangeEvent {
        kind,
        sequence,
        transaction_id,
        affected_item: None,
        effectivity: None,
        closure: None,
        note: None,
    }
}
