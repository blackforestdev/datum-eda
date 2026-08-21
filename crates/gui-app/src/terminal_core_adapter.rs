//! Production session boundary between the Linux PTY transport and TerminalCore.
//!
//! The adapter owns exactly one parser/core pair and the application-approved
//! resource profile. PTY bytes enter once, terminal replies leave once, and
//! production rendering consumes immutable core snapshots directly.

use datum_gui_protocol::TerminalLaneState;
use datum_terminal_core::{
    CoreError, CoreEvent, CoreLimitValues, CoreLimits, CursorShape, Damage, InputError, LimitKind,
    MouseEncoding, MouseTracking, RenderSnapshot, SearchError, SelectionError, StreamingParser,
    TerminalCore, TerminalSize,
};
use std::error::Error;
use std::fmt;

#[path = "terminal_core_adapter_interaction.rs"]
mod interaction;

pub(crate) const PRODUCTION_CORE_LIMIT_VALUES: CoreLimitValues = CoreLimitValues {
    parameter_count: 64,
    parameter_digits: 16,
    parameter_value: 1_000_000,
    subparameter_count: 64,
    intermediate_bytes: 16,
    control_string_bytes: 16 * 1024 * 1024,
    cluster_bytes: 4_096,
    title_bytes: 32_768,
    working_directory_bytes: 65_536,
    clipboard_bytes: 4 * 1024 * 1024,
    hyperlink_bytes: 1024 * 1024,
    input_bytes: 4 * 1024 * 1024,
    keyboard_stack: 32,
    notification_bytes: 65_536,
    reply_bytes: 65_536,
    pending_events: 4_096,
    pending_damage: 4_096,
    history_lines: 100_000,
    history_bytes: 64 * 1024 * 1024,
    graphic_objects: 256,
    graphic_pixels: 16_777_216,
    graphic_decoded_bytes: 64 * 1024 * 1024,
    graphic_frames: 1_024,
    compression_ratio: 1_024,
    parser_work: 67_108_864,
    search_work: 67_108_864,
    reflow_work: 67_108_864,
    screen_cells: 1_048_576,
    snapshot_cells: 33_554_432,
};

#[derive(Debug)]
pub(crate) enum TerminalCoreAdapterError {
    Limits(datum_terminal_core::LimitError),
    Size(datum_terminal_core::CoordinateError),
    Screen(datum_terminal_core::ScreenError),
    Snapshot(datum_terminal_core::SnapshotError),
    Input(InputError),
    Selection(SelectionError),
    #[allow(dead_code)]
    Search(SearchError),
}

impl fmt::Display for TerminalCoreAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limits(error) => error.fmt(formatter),
            Self::Size(error) => error.fmt(formatter),
            Self::Screen(error) => error.fmt(formatter),
            Self::Snapshot(error) => error.fmt(formatter),
            Self::Input(error) => error.fmt(formatter),
            Self::Selection(error) => error.fmt(formatter),
            Self::Search(error) => error.fmt(formatter),
        }
    }
}

impl Error for TerminalCoreAdapterError {}

#[derive(Debug, Default)]
pub(crate) struct TerminalCoreAdapterUpdate {
    pub(crate) replies: Vec<Vec<u8>>,
    pub(crate) events: Vec<CoreEvent>,
    pub(crate) damage: Vec<Damage>,
    pub(crate) semantic_errors: Vec<CoreError>,
    reply_bytes: usize,
}

pub(crate) struct TerminalCoreSessionAdapter {
    session_id: String,
    context_id: String,
    limits: CoreLimits,
    parser: StreamingParser,
    core: TerminalCore,
    bell_count: usize,
    pending_render_damage: Vec<Damage>,
}

impl TerminalCoreSessionAdapter {
    pub(crate) fn new(
        session_id: impl Into<String>,
        context_id: impl Into<String>,
        columns: u16,
        rows: u16,
    ) -> Result<Self, TerminalCoreAdapterError> {
        let limits = CoreLimits::try_from(PRODUCTION_CORE_LIMIT_VALUES)
            .map_err(TerminalCoreAdapterError::Limits)?;
        let size =
            TerminalSize::new(columns, rows, 0, 0).map_err(TerminalCoreAdapterError::Size)?;
        let core = TerminalCore::new(limits, size).map_err(TerminalCoreAdapterError::Screen)?;
        Ok(Self {
            session_id: session_id.into(),
            context_id: context_id.into(),
            limits,
            parser: StreamingParser::new(limits),
            core,
            bell_count: 0,
            pending_render_damage: vec![Damage::Full],
        })
    }

    pub(crate) fn session_id(&self) -> &str {
        &self.session_id
    }

    pub(crate) fn context_id(&self) -> &str {
        &self.context_id
    }

    #[cfg(test)]
    pub(crate) fn bracketed_paste_enabled(&self) -> bool {
        self.core.state().modes().bracketed_paste
    }

    #[cfg(test)]
    pub(crate) fn test_render_snapshot(&self) -> RenderSnapshot {
        self.core
            .render_snapshot()
            .expect("test core snapshot should remain within governed limits")
    }

    #[cfg(test)]
    pub(crate) fn test_plain_lines(&self) -> Vec<String> {
        use datum_terminal_core::CellContent;

        self.test_render_snapshot()
            .rows()
            .map(|row| {
                row.cells()
                    .iter()
                    .filter_map(|cell| match &cell.content {
                        CellContent::Cluster(cluster) => Some(cluster.text()),
                        CellContent::Empty | CellContent::Continuation { .. } => None,
                    })
                    .collect()
            })
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn test_plain_text(&self) -> String {
        self.test_plain_lines().join("\n")
    }

    pub(crate) fn take_render_state(
        &mut self,
    ) -> Result<(RenderSnapshot, Vec<Damage>), TerminalCoreAdapterError> {
        let snapshot = self
            .core
            .render_snapshot()
            .map_err(TerminalCoreAdapterError::Snapshot)?;
        let damage = std::mem::take(&mut self.pending_render_damage);
        Ok((snapshot, damage))
    }

    pub(crate) fn apply_output(
        &mut self,
        lane: &mut TerminalLaneState,
        bytes: &[u8],
    ) -> Result<TerminalCoreAdapterUpdate, TerminalCoreAdapterError> {
        self.limits
            .parser_work
            .check(bytes.len())
            .map_err(TerminalCoreAdapterError::Limits)?;
        let mut update = TerminalCoreAdapterUpdate::default();
        let parser = &mut self.parser;
        let core = &mut self.core;
        let limits = self.limits;
        let bell_count = &mut self.bell_count;
        let report = parser.feed(bytes, |action| {
            apply_action(core, limits, bell_count, action, &mut update);
        });
        debug_assert_eq!(report.consumed, bytes.len());
        self.merge_render_damage(&update.damage);
        self.project(lane)?;
        Ok(update)
    }

    pub(crate) fn finish(
        &mut self,
        lane: &mut TerminalLaneState,
    ) -> Result<TerminalCoreAdapterUpdate, TerminalCoreAdapterError> {
        let mut update = TerminalCoreAdapterUpdate::default();
        let parser = &mut self.parser;
        let core = &mut self.core;
        let limits = self.limits;
        let bell_count = &mut self.bell_count;
        parser.finish(|action| {
            apply_action(core, limits, bell_count, action, &mut update);
        });
        self.merge_render_damage(&update.damage);
        self.project(lane)?;
        Ok(update)
    }

    pub(crate) fn resize(
        &mut self,
        columns: u16,
        rows: u16,
        pixel_width: u32,
        pixel_height: u32,
    ) -> Result<(), TerminalCoreAdapterError> {
        let size = TerminalSize::new(columns, rows, pixel_width, pixel_height)
            .map_err(TerminalCoreAdapterError::Size)?;
        let reduction = self
            .core
            .resize(size)
            .map_err(TerminalCoreAdapterError::Screen)?;
        self.merge_render_damage(&reduction.damage().iter().collect::<Vec<_>>());
        Ok(())
    }

    pub(crate) fn project(
        &self,
        lane: &mut TerminalLaneState,
    ) -> Result<(), TerminalCoreAdapterError> {
        let state = self.core.state();
        let cursor = state.cursor();
        let modes = state.modes();
        lane.title = state.title().map(|title| title.as_str().to_owned());
        lane.current_working_directory = state
            .working_directory()
            .map(|directory| directory.as_str().to_owned());
        lane.bell_count = self.bell_count;
        lane.columns = state.size().columns.get();
        lane.rows = state.size().rows.get();
        lane.screen_cursor_row = usize::from(cursor.position.row.get());
        lane.screen_cursor_col = usize::from(cursor.position.column.get());
        lane.screen_cursor_visible = cursor.visible;
        lane.screen_cursor_style = Some(cursor_style(cursor.shape, cursor.blinking).to_string());
        lane.application_cursor_keys = modes.application_cursor;
        lane.application_keypad = modes.application_keypad;
        lane.focus_event_reporting = modes.focus_reporting;
        lane.mouse_reporting_mode = mouse_tracking(state.mouse_tracking()).map(str::to_string);
        lane.mouse_coordinate_encoding = mouse_encoding(state.mouse_encoding()).map(str::to_string);
        lane.scroll_offset = 0;
        Ok(())
    }

    fn merge_render_damage(&mut self, entries: &[Damage]) {
        for &entry in entries {
            if self.pending_render_damage.contains(&Damage::Full) {
                return;
            }
            if entry == Damage::Full {
                self.pending_render_damage.clear();
                self.pending_render_damage.push(Damage::Full);
                return;
            }
            if !self.pending_render_damage.contains(&entry) {
                self.pending_render_damage.push(entry);
            }
        }
    }
}

fn apply_action(
    core: &mut TerminalCore,
    limits: CoreLimits,
    bell_count: &mut usize,
    action: datum_terminal_core::Action,
    batch: &mut TerminalCoreAdapterUpdate,
) {
    match core.apply(action) {
        Ok(update) => {
            for reply in update.replies() {
                let total = batch.reply_bytes.saturating_add(reply.bytes().len());
                if limits.reply_bytes.check(total).is_ok() {
                    batch.replies.push(reply.bytes().to_vec());
                    batch.reply_bytes = total;
                } else {
                    push_event_bounded(
                        batch,
                        limits,
                        CoreEvent::LimitReached(LimitKind::ReplyBytes),
                    );
                }
            }
            for event in update.events() {
                if matches!(event, CoreEvent::Bell) {
                    *bell_count = bell_count.saturating_add(1);
                }
                push_event_bounded(batch, limits, event.clone());
            }
            for damage in update.damage().iter() {
                push_damage_bounded(&mut batch.damage, limits, damage);
            }
        }
        Err(error) => {
            if batch.semantic_errors.is_empty() {
                batch.semantic_errors.push(error);
            }
        }
    }
}

fn push_event_bounded(batch: &mut TerminalCoreAdapterUpdate, limits: CoreLimits, event: CoreEvent) {
    if limits.pending_events.check(batch.events.len() + 1).is_ok() {
        batch.events.push(event);
    } else if !batch.events.is_empty()
        && !batch
            .events
            .iter()
            .any(|event| matches!(event, CoreEvent::LimitReached(LimitKind::PendingEvents)))
    {
        let last = batch.events.len().saturating_sub(1);
        batch.events[last] = CoreEvent::LimitReached(LimitKind::PendingEvents);
    }
}

fn push_damage_bounded(damage: &mut Vec<Damage>, limits: CoreLimits, entry: Damage) {
    if damage.contains(&Damage::Full) || damage.contains(&entry) {
        return;
    }
    if limits.pending_damage.check(damage.len() + 1).is_ok() {
        damage.push(entry);
    } else {
        damage.clear();
        damage.push(Damage::Full);
    }
}

const fn cursor_style(shape: CursorShape, blinking: bool) -> &'static str {
    match (shape, blinking) {
        (CursorShape::Block, true) => "blinking_block",
        (CursorShape::Block, false) => "steady_block",
        (CursorShape::Underline, true) => "blinking_underline",
        (CursorShape::Underline, false) => "steady_underline",
        (CursorShape::Bar, true) => "blinking_bar",
        (CursorShape::Bar, false) => "steady_bar",
    }
}

const fn mouse_tracking(mode: MouseTracking) -> Option<&'static str> {
    match mode {
        MouseTracking::Off => None,
        MouseTracking::X10 => Some("x10"),
        MouseTracking::Button => Some("normal"),
        MouseTracking::Drag => Some("button_event"),
        MouseTracking::Any => Some("any_event"),
    }
}

const fn mouse_encoding(encoding: MouseEncoding) -> Option<&'static str> {
    match encoding {
        MouseEncoding::Default => None,
        MouseEncoding::Utf8 => Some("utf8"),
        MouseEncoding::Sgr => Some("sgr"),
        MouseEncoding::Urxvt => Some("urxvt"),
        MouseEncoding::SgrPixels => Some("sgr_pixels"),
    }
}

#[cfg(test)]
#[path = "terminal_core_adapter_tests.rs"]
mod tests;
