//! Renderer-independent, first-party terminal state for Datum EDA.
//!
//! This crate is intentionally `std`-only. It owns terminal semantics but no
//! PTY, GUI, filesystem, process, network, clipboard, MCP, or design-model I/O.
//! DTC-P07 establishes the closed data contract; later DTC packages add the
//! streaming parser and the sole state reducer behind these types.

#![forbid(unsafe_code)]

mod cell;
mod charset;
mod color;
mod control_string;
mod coordinates;
mod csi;
mod damage;
mod event;
mod grid;
mod history;
mod hyperlink;
mod input;
mod input_key;
mod input_modes;
mod input_mouse;
mod limits;
mod mode;
mod parser;
mod parser_action;
mod reducer;
mod reducer_action;
mod reflow;
mod screen;
mod search;
mod search_regex;
mod selection;
mod semantics;
mod sgr;
mod snapshot;
mod unicode;
mod unicode_grapheme_tables;
mod unicode_width_tables;

pub use cell::{
    Cell, CellAttribute, CellAttributes, CellContent, CellStyle, CellWidth, Cluster, ClusterError,
    HyperlinkId, UnderlineStyle,
};
pub use charset::{CharacterSet, CharacterSetSlot, CharacterSetState};
pub use color::{Color, PaletteIndex, Rgb};
pub use coordinates::{
    CellPoint, Column, Columns, CoordinateError, LogicalLineId, LogicalPoint, PixelSize, Row, Rows,
    TerminalSize,
};
pub use damage::{Damage, DamageSet, ScrollDirection};
pub use event::{
    ClipboardBytes, ClipboardSelection, CoreEvent, NotificationText, Percent, ProgressState,
    ReplyKind, ShellMark, TerminalReply, TitleText, WorkingDirectoryText,
};
pub use history::{AnchorResolution, HistoryRowSnapshot, HistorySnapshot};
pub use hyperlink::Hyperlink;
pub use input::{
    FocusInput, ImeInput, InputBytes, InputDisposition, InputError, KeyCode, KeyEventKind,
    KeyInput, KeyModifiers, KeypadKey, MediaKey, ModifierKey,
};
pub use input_mouse::{MouseAction, MouseButton, MouseInput, MousePosition};
pub use limits::{
    ClipboardBytesLimit, ClusterBytesLimit, CompressionRatioLimit, ControlStringBytesLimit,
    CoreLimitValues, CoreLimits, GraphicDecodedBytesLimit, GraphicFramesLimit, GraphicObjectsLimit,
    GraphicPixelsLimit, HistoryBytesLimit, HistoryLinesLimit, HyperlinkBytesLimit, InputBytesLimit,
    IntermediateBytesLimit, KeyboardStackLimit, LimitError, LimitKind, NotificationBytesLimit,
    ParameterCountLimit, ParameterDigitsLimit, ParameterValueLimit, ParserWorkLimit,
    PendingDamageLimit, PendingEventsLimit, ReflowWorkLimit, ReplyBytesLimit, ScreenCellsLimit,
    SearchWorkLimit, SnapshotCellsLimit, SubparameterCountLimit, TitleBytesLimit,
    WorkingDirectoryBytesLimit,
};
pub use mode::{
    CursorShape, CursorState, KittyKeyboardState, Margins, ModeState, MouseEncoding, MouseTracking,
    SavedCursorState, ScreenBuffer, TabStops,
};
pub use parser::{FeedReport, StreamingParser};
pub use parser_action::{
    Action, ControlCode, ControlString, ControlStringKind, CsiParameter, CsiSequence,
    EscapeSequence, ParseError, ParserStateKind, StringTerminator,
};
pub use reducer::{Reduction, ScreenError};
pub use reducer_action::{EraseDisplay, EraseLine, FoundationMode, ScreenAction};
pub use screen::{ScreenState, TerminalCore};
pub use search::{
    SearchCase, SearchCursor, SearchDirection, SearchError, SearchMatch, SearchMatchState,
    SearchQuery, SearchResult,
};
pub use selection::{CopiedText, Selection, SelectionError, SelectionScope, SelectionState};
pub use semantics::{CoreError, CoreUpdate};
pub use snapshot::{SnapshotCell, SnapshotError, SnapshotRow, TerminalSnapshot};
pub use unicode::{
    BIDIRECTIONAL_TEXT_POLICY, BidirectionalTextPolicy, GraphemeIndices, ShapingCluster,
    UNICODE_VERSION, grapheme_break_before, grapheme_indices, terminal_cluster_width,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod parser_tests;

#[cfg(test)]
mod reducer_tests;

#[cfg(test)]
mod semantic_tests;

#[cfg(test)]
mod unicode_tests;

#[cfg(test)]
mod history_tests;

#[cfg(test)]
mod selection_tests;

#[cfg(test)]
mod search_tests;

#[cfg(test)]
mod input_tests;
