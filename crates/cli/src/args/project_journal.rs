use crate::*;

#[derive(clap::Args)]
pub(crate) struct ProjectUndoArgs {
    /// Project root directory
    pub(crate) path: PathBuf,

    /// Refuse undo unless the resolved model revision matches this value
    #[arg(long = "expected-model-revision")]
    pub(crate) expected_model_revision: Option<String>,

    /// Refuse undo unless the current journal tip has this transaction UUID
    #[arg(long = "expected-tip-transaction")]
    pub(crate) expected_tip_transaction: Option<Uuid>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRedoArgs {
    /// Project root directory
    pub(crate) path: PathBuf,

    /// Refuse redo unless the resolved model revision matches this value
    #[arg(long = "expected-model-revision")]
    pub(crate) expected_model_revision: Option<String>,

    /// Refuse redo unless the current journal tip has this transaction UUID
    #[arg(long = "expected-tip-transaction")]
    pub(crate) expected_tip_transaction: Option<Uuid>,
}
