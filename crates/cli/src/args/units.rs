use crate::*;

#[derive(Subcommand)]
pub(crate) enum UnitsCommands {
    /// Resolve one canonical-nanometer value or exact unit expression
    #[command(name = "resolve-length")]
    ResolveLength(ResolveLengthArgs),
}

#[derive(clap::Args)]
pub(crate) struct ResolveLengthArgs {
    /// Existing canonical integer nanometer value
    #[arg(long)]
    pub(crate) canonical_nm: Option<i64>,

    /// Locale-independent scalar with an optional supported unit suffix
    #[arg(long)]
    pub(crate) expression: Option<String>,

    /// Explicit bare-value quantity context: board, drill, or schematic
    #[arg(long)]
    pub(crate) quantity: Option<String>,

    /// Explicit bare-value unit context: nm, um, µm, mm, mil, or in
    #[arg(long)]
    pub(crate) unit: Option<String>,

    /// Explicit context measurement system: metric or imperial
    #[arg(long)]
    pub(crate) system: Option<String>,

    /// Stable field identity used by the caller
    #[arg(long)]
    pub(crate) field: Option<String>,

    /// Optional Project identity carried in provenance
    #[arg(long)]
    pub(crate) project_id: Option<String>,
}
