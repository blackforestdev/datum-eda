use super::*;

#[derive(Subcommand)]
pub(crate) enum CheckCommands {
    /// Run the native project check surface and persist CheckRun evidence
    Run(CheckRunArgs),
    /// List persisted CheckRun evidence discovered by the resolver
    List(CheckListArgs),
    /// Show one persisted CheckRun evidence record discovered by the resolver
    Show(CheckShowArgs),
    /// List supported native-project check profiles
    Profiles(CheckProfilesArgs),
    /// Persist generated ZoneFill evidence for native board zones
    FillZones(CheckFillZonesArgs),
    /// Generate standards-repair proposals from the current native project CheckRun
    RepairStandards(CheckRepairStandardsArgs),
    /// Author a fingerprint-scoped check finding waiver
    Waive(CheckWaiveArgs),
    /// Accept a fingerprint-scoped check finding as a deviation
    AcceptDeviation(CheckAcceptDeviationArgs),
    /// Run the legacy imported-design check surface
    Imported(CheckImportedArgs),
}

#[derive(clap::Args)]
pub(crate) struct CheckRunArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Check profile id: native-combined, erc, drc, standards, manufacturing, or release
    #[arg(long)]
    pub(crate) profile: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct CheckListArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct CheckShowArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// CheckRun UUID to inspect
    #[arg(long = "check-run")]
    pub(crate) check_run: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct CheckProfilesArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct CheckFillZonesArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Optional Zone UUID to fill
    #[arg(long = "zone")]
    pub(crate) zone_uuid: Option<Uuid>,
    /// Optional Net UUID to fill
    #[arg(long = "net")]
    pub(crate) net_uuid: Option<Uuid>,
}

#[derive(clap::Args)]
pub(crate) struct CheckRepairStandardsArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct CheckWaiveArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Stable CheckFinding fingerprint to waive
    #[arg(long)]
    pub(crate) fingerprint: String,
    /// Waiver rationale recorded in the authored project
    #[arg(long)]
    pub(crate) rationale: String,
    /// Optional actor/user recorded on the waiver
    #[arg(long = "created-by")]
    pub(crate) created_by: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct CheckAcceptDeviationArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Stable CheckFinding fingerprint to accept as a deviation
    #[arg(long)]
    pub(crate) fingerprint: String,
    /// Deviation rationale recorded in the authored project
    #[arg(long)]
    pub(crate) rationale: String,
    /// Optional actor/user recorded as accepting the deviation
    #[arg(long = "accepted-by")]
    pub(crate) accepted_by: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct CheckImportedArgs {
    /// Path to imported design file
    pub(crate) path: PathBuf,
    /// Exit nonzero if the check report status meets or exceeds this level
    #[arg(long, value_enum)]
    pub(crate) fail_on: Option<FailOn>,
}
