//! Technical integrity and crash-safe local storage for Revision authority.
//!
//! This module establishes the substrate on which later product authority
//! records can depend. It deliberately defines no EngineeringRevision,
//! Release, approval, policy, or workflow record kinds.

mod backup;
mod canonical;
mod model;
mod store;

pub use backup::{
    BackupEntry, BackupManifest, RestorePreview, RestoreReceipt, export_backup, preview_restore,
    restore_backup, undo_restore, verify_backup,
};
pub use model::{
    AffectedShard, AffectedShardAction, AlgorithmQualifiedDigest, IntegrityDiagnostic,
    IntegrityGeneration, IntegrityHead, REVISION_STORE_SCHEMA_VERSION, RevisionStoreState,
    TechnicalValidationState,
};
pub use store::{
    ProjectWriteLease, RevisionAuthorityStore, StagedIntegrityGeneration, transaction_tip,
};

pub(crate) use model::StagedShardPostimage;
pub(crate) use store::IntegrityCommitFaultPoint;
