use clap::Args;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Args)]
pub(crate) struct SetBoardComponentPartArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
    /// Replacement part UUID
    #[arg(long = "part")]
    pub(crate) part_uuid: Uuid,
}

#[derive(Args)]
pub(crate) struct SetBoardComponentPackageArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
    /// Replacement package UUID
    #[arg(long = "package")]
    pub(crate) package_uuid: Uuid,
    /// Authored ComponentInstance for explicit mapped pad materialization
    #[arg(long = "component-instance", requires = "pin_pad_map")]
    pub(crate) component_instance: Option<Uuid>,
    /// Explicit declared PinPadMap; uses its exact Footprint reference
    #[arg(long = "pin-pad-map", requires = "component_instance")]
    pub(crate) pin_pad_map: Option<Uuid>,
    /// Retain pad identity using <placed-pad-uuid>=<target-library-pad-uuid>
    #[arg(long = "pad-correspondence", requires = "pin_pad_map")]
    pub(crate) pad_correspondences: Vec<String>,
    /// Explicitly adopt existing unqualified pad sources
    #[arg(long = "adopt-existing", requires = "pin_pad_map")]
    pub(crate) adopt_existing: bool,
}

#[derive(Args)]
pub(crate) struct SetBoardComponentValueArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
    /// Replacement component value
    #[arg(long = "value")]
    pub(crate) value: String,
}

#[derive(Args)]
pub(crate) struct BoardComponentArgs {
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
}

#[derive(Args)]
pub(crate) struct BoardComponentModels3dArgs {
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
}

#[derive(Args)]
pub(crate) struct BoardComponentPadsArgs {
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
}

#[derive(Args)]
pub(crate) struct BoardComponentSilkscreenArgs {
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
}

#[derive(Args)]
pub(crate) struct BoardComponentMechanicalArgs {
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
}

#[derive(Args)]
pub(crate) struct SetBoardComponentReferenceArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
    /// Replacement reference designator
    #[arg(long = "reference")]
    pub(crate) reference: String,
}

#[derive(Args)]
pub(crate) struct SetBoardComponentLayerArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Component UUID
    #[arg(long = "component")]
    pub(crate) component_uuid: Uuid,
    /// Replacement layer identifier
    #[arg(long = "layer")]
    pub(crate) layer: i32,
}
