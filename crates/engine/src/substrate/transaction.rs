use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ModelRevision, ObjectId, Operation};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationBatch {
    pub batch_id: Uuid,
    pub expected_model_revision: Option<ModelRevision>,
    pub provenance: CommitProvenance,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitProvenance {
    pub actor: String,
    pub source: CommitSource,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitSource {
    Manual,
    Cli,
    Test,
    Tool,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CommitDiff {
    pub created: Vec<ObjectId>,
    pub modified: Vec<ObjectId>,
    pub deleted: Vec<ObjectId>,
}

/// Session-scoped agent authority that caused a journaled transaction.
///
/// This contains identifiers and policy evidence only. Bearer credentials and
/// their filesystem locations are deliberately excluded from the journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentCommitProvenance {
    pub agent_identity: String,
    pub agent_launch_id: String,
    pub terminal_session_id: String,
    pub context_id: String,
    pub expected_model_revision: String,
    pub accepted_transaction_tip: Option<String>,
    pub requested_capability: String,
    pub approval_policy: String,
    pub approval_reference: Option<String>,
    pub tool_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub transaction_id: Uuid,
    pub batch_id: Uuid,
    #[serde(default)]
    pub transaction_kind: TransactionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub undo_of: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redo_of: Option<Uuid>,
    pub before_model_revision: ModelRevision,
    pub after_model_revision: ModelRevision,
    pub provenance: CommitProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_provenance: Option<AgentCommitProvenance>,
    pub diff: CommitDiff,
    pub operations: Vec<Operation>,
    #[serde(default)]
    pub inverse_operations: Vec<Operation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransactionKind {
    #[default]
    Normal,
    Undo,
    Redo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitReport {
    pub transaction: TransactionRecord,
    pub journal_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalCursor {
    pub applied_transaction_count: usize,
}
