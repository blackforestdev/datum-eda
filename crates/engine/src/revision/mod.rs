//! Technical integrity and crash-safe local storage for Revision authority.
//!
//! The technical store and inert typed authority vocabulary live here. Policy,
//! lifecycle transitions, issuance, and public workflow remain later slices.

mod authority;
mod backup;
mod canonical;
mod model;
mod resolver;
mod store;
#[cfg(test)]
mod test_support;

pub use authority::*;

pub use backup::{
    BackupEntry, BackupManifest, RestorePreview, RestoreReceipt, export_backup, preview_restore,
    restore_backup, undo_restore, verify_backup,
};
pub use model::{
    AffectedShard, AffectedShardAction, AlgorithmQualifiedDigest, IntegrityDiagnostic,
    IntegrityGeneration, IntegrityHead, REVISION_STORE_SCHEMA_VERSION, RevisionStoreState,
    TechnicalValidationState,
};
pub use resolver::AuthorityResolution;
pub use store::{
    ProjectWriteLease, RevisionAuthorityStore, StagedIntegrityGeneration, transaction_tip,
};

pub(crate) use model::StagedShardPostimage;
pub(crate) use store::IntegrityCommitFaultPoint;
