use crate::*;

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum ProjectUnitsSourceArg {
    Global,
    Factory,
}

#[derive(clap::Args)]
pub(crate) struct ProjectNewArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project display name; defaults to the directory basename
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Reproducible Project identity; generated when omitted
    #[arg(long)]
    pub(crate) project_id: Option<Uuid>,
    /// Idempotent creation-request identity; generated when omitted
    #[arg(long)]
    pub(crate) request_id: Option<Uuid>,
    /// Units source for this new Project
    #[arg(long, value_enum, default_value = "global")]
    pub(crate) units_source: ProjectUnitsSourceArg,
    /// Full expected Global Preferences GenerationRef JSON; Global mode only
    #[arg(long)]
    pub(crate) expected_preferences: Option<String>,
    /// Render the exact ProjectGenesisResultV1 as JSON
    #[arg(long)]
    pub(crate) json: bool,
}
