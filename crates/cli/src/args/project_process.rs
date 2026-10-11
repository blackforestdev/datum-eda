use clap::{Args, Subcommand};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Args)]
pub(crate) struct ProjectProcessArgs {
    /// Native project root
    pub(crate) path: PathBuf,
    #[command(subcommand)]
    pub(crate) action: ProcessCommands,
}

#[derive(Subcommand)]
pub(crate) enum ProcessCommands {
    /// Inspect authored profiles, preserving historical absence
    Profiles,
    /// Capture the canonical current target pin for an explicit Pad correspondence
    PinPad {
        #[arg(long)]
        package: Uuid,
        #[arg(long)]
        footprint: Uuid,
        #[arg(long)]
        pad: Uuid,
    },
    /// Create or replace a concrete UUID-keyed profile through the journal
    SetProfile {
        #[arg(long)]
        profile_json: PathBuf,
    },
    /// Delete an unreferenced profile through the journal
    DeleteProfile {
        #[arg(long)]
        profile: Uuid,
    },
    /// Select an authored inline/IPC basis or pin a current project profile
    SelectBasis {
        #[arg(long)]
        footprint: Uuid,
        #[arg(long, conflicts_with = "profile", required_unless_present = "profile")]
        basis_json: Option<PathBuf>,
        #[arg(long, conflicts_with = "basis_json")]
        profile: Option<Uuid>,
    },
    /// Explicitly adopt process capability with actual byte preimage; forward-only
    Adopt {
        #[arg(long)]
        capability_json: PathBuf,
    },
    /// Change adopted physical face associations through the journal
    SetAssociations {
        #[arg(long)]
        capability_json: PathBuf,
    },
    /// Resolve exact library process geometry without writing
    Inspect {
        #[arg(long)]
        footprint: Uuid,
        #[arg(long)]
        padstack: Uuid,
    },
    /// Inspect frozen placed process geometry, staleness and explicit source pins
    Placements {
        #[arg(long)]
        component: Option<Uuid>,
    },
}
