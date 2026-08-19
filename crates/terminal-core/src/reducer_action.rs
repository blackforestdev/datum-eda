use crate::{CellStyle, Cluster, Margins, ScreenBuffer};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EraseLine {
    Right,
    Left,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EraseDisplay {
    Below,
    Above,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundationMode {
    AutoWrap,
    Origin,
    Insert,
    Newline,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScreenAction {
    Print(Cluster),
    Backspace,
    CarriageReturn,
    LineFeed,
    ReverseIndex,
    HorizontalTab,
    SetCursor {
        row: u16,
        column: u16,
    },
    MoveCursor {
        rows: i32,
        columns: i32,
    },
    SetMargins(Margins),
    ResetMargins,
    InsertCells(u16),
    DeleteCells(u16),
    EraseCells(u16),
    InsertLines(u16),
    DeleteLines(u16),
    ScrollUp(u16),
    ScrollDown(u16),
    EraseLine {
        mode: EraseLine,
        selective: bool,
    },
    EraseDisplay {
        mode: EraseDisplay,
        selective: bool,
    },
    SwitchBuffer {
        buffer: ScreenBuffer,
        clear: bool,
        home: bool,
    },
    SaveCursor,
    RestoreCursor,
    SetMode {
        mode: FoundationMode,
        enabled: bool,
    },
    SetStyle(CellStyle),
    SetProtection(bool),
    Reset,
}
