use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectCreatePoolPinPadMapArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// PinPadMap UUID
    #[arg(long = "map")]
    pub(crate) map_uuid: Uuid,
    /// Part UUID this PinPadMap binds
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Optional Footprint UUID; if omitted mappings target package pads
    #[arg(long = "footprint")]
    pub(crate) footprint_uuid: Option<Uuid>,
    /// Mapping entry as pad_uuid:gate_uuid:pin_uuid; pin_uuid:pad_uuid is allowed only when unambiguous
    #[arg(long = "entry", required = true)]
    pub(crate) entries: Vec<String>,
    /// Also set this map as the part default_pin_pad_map in the same journal batch
    #[arg(long = "set-default")]
    pub(crate) set_default: bool,
}

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPinPadMapArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// PinPadMap UUID
    #[arg(long = "map")]
    pub(crate) map_uuid: Uuid,
    /// Merge listed mappings or replace the full mapping table
    #[arg(long, default_value = "merge")]
    pub(crate) mode: String,
    /// Mapping entry as pad_uuid:gate_uuid:pin_uuid; pin_uuid:pad_uuid is allowed only when unambiguous
    #[arg(long = "entry", required = true)]
    pub(crate) entries: Vec<String>,
}
