use std::path::PathBuf;

use clap::Args;
use uuid::Uuid;

#[derive(Args)]
pub(crate) struct PlaceBoardDimensionArgs {
    pub(crate) path: PathBuf,
    #[arg(long = "from-x-nm")]
    pub(crate) from_x_nm: i64,
    #[arg(long = "from-y-nm")]
    pub(crate) from_y_nm: i64,
    #[arg(long = "to-x-nm")]
    pub(crate) to_x_nm: i64,
    #[arg(long = "to-y-nm")]
    pub(crate) to_y_nm: i64,
    #[arg(long = "layer")]
    pub(crate) layer: i32,
    #[arg(long)]
    pub(crate) text: Option<String>,
}

#[derive(Args)]
pub(crate) struct EditBoardDimensionArgs {
    pub(crate) path: PathBuf,
    #[arg(long = "dimension")]
    pub(crate) dimension_uuid: Uuid,
    #[arg(long = "from-x-nm")]
    pub(crate) from_x_nm: Option<i64>,
    #[arg(long = "from-y-nm")]
    pub(crate) from_y_nm: Option<i64>,
    #[arg(long = "to-x-nm")]
    pub(crate) to_x_nm: Option<i64>,
    #[arg(long = "to-y-nm")]
    pub(crate) to_y_nm: Option<i64>,
    #[arg(long = "layer")]
    pub(crate) layer: Option<i32>,
    #[arg(long)]
    pub(crate) text: Option<String>,
    #[arg(long = "clear-text", default_value_t = false)]
    pub(crate) clear_text: bool,
}
