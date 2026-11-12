//! Transient review over the shared engine initial-transfer planner.
use eda_engine::api::native_write::schematic_board_transfer::{TransferPlan, TransferRequest};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardTransferReview {
    pub request: TransferRequest,
    pub plan: TransferPlan,
    pub first_row: usize,
    pub refusal: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTransferAction {
    Surface,
    ChooseMap(usize),
    Previous,
    Next,
    PrepareSources,
    Accept,
    Cancel,
}
