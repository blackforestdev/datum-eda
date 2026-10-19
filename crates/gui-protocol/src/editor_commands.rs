//! Typed field commands and status shared by native editor input clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextBooleanField {
    Mirrored,
    KeepUpright,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextAlignmentField {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextLineSpacingStep {
    Decrease,
    Increase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextHeightStep {
    Decrease,
    Increase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextRotationStep {
    CounterClockwise90,
    Clockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTextCycleField {
    RenderIntent,
    Family,
}

pub const BOARD_TEXT_LINE_SPACING_MIN_PPM: i32 = 500_000;
pub const BOARD_TEXT_LINE_SPACING_MAX_PPM: i32 = 2_000_000;
pub const BOARD_TEXT_LINE_SPACING_STEP_PPM: i32 = 100_000;
pub const BOARD_TEXT_HEIGHT_MIN_NM: i64 = 50_000;
pub const BOARD_TEXT_HEIGHT_MAX_NM: i64 = 100_000_000;
pub const BOARD_TEXT_HEIGHT_STEP_PPM: i64 = 100_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorCommandStatus {
    pub action: String,
    pub detail: String,
}
