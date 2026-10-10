//! Generic native pool object creation, replacement and deletion arguments.
use crate::*;

#[derive(clap::Args)]
pub(crate) struct ProjectCreatePoolLibraryObjectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Native pool object kind
    #[arg(long)]
    pub(crate) kind: String,
    /// Object UUID
    #[arg(long = "object")]
    pub(crate) object_uuid: Uuid,
    /// JSON file containing the object payload
    #[arg(long = "from-json")]
    pub(crate) from_json: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeletePoolLibraryObjectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Native pool object kind
    #[arg(long)]
    pub(crate) kind: String,
    /// Object UUID
    #[arg(long = "object")]
    pub(crate) object_uuid: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolLibraryObjectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Native pool object kind
    #[arg(long)]
    pub(crate) kind: String,
    /// Object UUID
    #[arg(long = "object")]
    pub(crate) object_uuid: Uuid,
    /// Replace a supported section instead of the whole object
    #[arg(long, value_parser = ["fields", "drawings"])]
    pub(crate) section: Option<String>,
    /// JSON file containing the replacement object or section payload
    #[arg(long = "from-json")]
    pub(crate) from_json: PathBuf,
}
