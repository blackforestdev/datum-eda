use crate::*;

#[derive(clap::Args)]
pub(crate) struct ProjectSetPoolPartBindingsArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Project-local pool path; defaults to pool
    #[arg(long, default_value = "pool")]
    pub(crate) pool: String,
    /// Part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
    /// Default footprint UUID for this part; omit with --clear-default-footprint to clear
    #[arg(long = "default-footprint")]
    pub(crate) default_footprint: Option<Uuid>,
    /// Clear the part default footprint binding
    #[arg(long = "clear-default-footprint")]
    pub(crate) clear_default_footprint: bool,
    /// Default pin-pad-map UUID for this part; omit with --clear-default-pin-pad-map to clear
    #[arg(long = "default-pin-pad-map")]
    pub(crate) default_pin_pad_map: Option<Uuid>,
    /// Clear the part default pin-pad-map binding
    #[arg(long = "clear-default-pin-pad-map")]
    pub(crate) clear_default_pin_pad_map: bool,
}
