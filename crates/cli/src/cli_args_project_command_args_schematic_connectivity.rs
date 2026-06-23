use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectCreateSheetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Sheet name
    #[arg(long)]
    pub(crate) name: String,
    /// Optional sheet UUID. Omit to allocate a fresh UUID.
    #[arg(long)]
    pub(crate) sheet: Option<Uuid>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteSheetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRenameSheetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// New sheet name
    #[arg(long)]
    pub(crate) name: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreateSheetDefinitionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Root sheet UUID for this sheet definition
    #[arg(long)]
    pub(crate) root_sheet: Uuid,
    /// Sheet definition name
    #[arg(long)]
    pub(crate) name: String,
    /// Optional sheet definition UUID. Omit to allocate a fresh UUID.
    #[arg(long)]
    pub(crate) definition: Option<Uuid>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreateSheetInstanceArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Sheet definition UUID to instantiate
    #[arg(long)]
    pub(crate) definition: Uuid,
    /// Optional parent sheet UUID for this instance
    #[arg(long)]
    pub(crate) parent_sheet: Option<Uuid>,
    /// Instance name
    #[arg(long)]
    pub(crate) name: String,
    /// X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
    /// Optional sheet instance UUID. Omit to allocate a fresh UUID.
    #[arg(long)]
    pub(crate) instance: Option<Uuid>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteSheetInstanceArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Sheet instance UUID
    #[arg(long)]
    pub(crate) instance: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectMoveSheetInstanceArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Sheet instance UUID
    #[arg(long)]
    pub(crate) instance: Uuid,
    /// New X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// New Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectBindSheetInstancePortArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Sheet instance UUID
    #[arg(long)]
    pub(crate) instance: Uuid,
    /// Parent-sheet hierarchical port UUID to expose through this instance
    #[arg(long)]
    pub(crate) port: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectUnbindSheetInstancePortArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Sheet instance UUID
    #[arg(long)]
    pub(crate) instance: Uuid,
    /// Parent-sheet hierarchical port UUID to remove from this instance binding
    #[arg(long)]
    pub(crate) port: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceLabelArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Label name
    #[arg(long)]
    pub(crate) name: String,
    /// Label kind
    #[arg(long, value_enum, default_value = "local")]
    pub(crate) kind: NativeLabelKindArg,
    /// X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRenameLabelArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Label UUID
    #[arg(long)]
    pub(crate) label: Uuid,
    /// New label name
    #[arg(long)]
    pub(crate) name: String,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteLabelArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Label UUID
    #[arg(long)]
    pub(crate) label: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDrawWireArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Start X coordinate in nm
    #[arg(long)]
    pub(crate) from_x_nm: i64,
    /// Start Y coordinate in nm
    #[arg(long)]
    pub(crate) from_y_nm: i64,
    /// End X coordinate in nm
    #[arg(long)]
    pub(crate) to_x_nm: i64,
    /// End Y coordinate in nm
    #[arg(long)]
    pub(crate) to_y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteWireArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Wire UUID
    #[arg(long)]
    pub(crate) wire: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceJunctionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteJunctionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Junction UUID
    #[arg(long)]
    pub(crate) junction: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlacePortArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Port name
    #[arg(long)]
    pub(crate) name: String,
    /// Port direction
    #[arg(long, value_enum)]
    pub(crate) direction: NativePortDirectionArg,
    /// X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditPortArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Port UUID
    #[arg(long)]
    pub(crate) port: Uuid,
    /// New port name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// New port direction
    #[arg(long, value_enum)]
    pub(crate) direction: Option<NativePortDirectionArg>,
    /// New X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: Option<i64>,
    /// New Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: Option<i64>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeletePortArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Port UUID
    #[arg(long)]
    pub(crate) port: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreateBusArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Bus name
    #[arg(long = "name")]
    pub(crate) name: String,
    /// Bus member labels
    #[arg(long = "member")]
    pub(crate) members: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectEditBusMembersArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Bus UUID
    #[arg(long)]
    pub(crate) bus: Uuid,
    /// Replacement bus member labels
    #[arg(long = "member")]
    pub(crate) members: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBusArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Bus UUID
    #[arg(long)]
    pub(crate) bus: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceBusEntryArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Parent bus UUID
    #[arg(long)]
    pub(crate) bus: Uuid,
    /// Optional attached wire UUID
    #[arg(long)]
    pub(crate) wire: Option<Uuid>,
    /// X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteBusEntryArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Bus entry UUID
    #[arg(long = "bus-entry")]
    pub(crate) bus_entry: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPlaceNoConnectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Target symbol UUID
    #[arg(long)]
    pub(crate) symbol: Uuid,
    /// Target pin UUID
    #[arg(long)]
    pub(crate) pin: Uuid,
    /// X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteNoConnectArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// No-connect UUID
    #[arg(long = "noconnect")]
    pub(crate) noconnect: Uuid,
}
