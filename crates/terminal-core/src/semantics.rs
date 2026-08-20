use crate::{
    Action, CharacterSetSlot, Cluster, CoreEvent, Damage, DamageSet, EscapeSequence, LimitError,
    ParseError, ReplyKind, ScreenAction, ScreenError, TerminalCore, TerminalReply,
};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreError {
    Screen(ScreenError),
    Limit(LimitError),
    Sixel(crate::SixelError),
    KittyGraphics(crate::KittyGraphicsError),
    InvalidPrintable,
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Screen(error) => error.fmt(formatter),
            Self::Limit(error) => error.fmt(formatter),
            Self::Sixel(error) => error.fmt(formatter),
            Self::KittyGraphics(error) => error.fmt(formatter),
            Self::InvalidPrintable => formatter.write_str("terminal printable cluster is invalid"),
        }
    }
}

impl Error for CoreError {}

impl From<ScreenError> for CoreError {
    fn from(value: ScreenError) -> Self {
        Self::Screen(value)
    }
}

impl From<LimitError> for CoreError {
    fn from(value: LimitError) -> Self {
        Self::Limit(value)
    }
}

impl From<crate::SixelError> for CoreError {
    fn from(value: crate::SixelError) -> Self {
        Self::Sixel(value)
    }
}

impl From<crate::KittyGraphicsError> for CoreError {
    fn from(value: crate::KittyGraphicsError) -> Self {
        Self::KittyGraphics(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreUpdate {
    pub(crate) damage: DamageSet,
    pub(crate) events: Vec<CoreEvent>,
    pub(crate) replies: Vec<TerminalReply>,
    pub(crate) recognized: bool,
}

impl CoreUpdate {
    pub(crate) fn new(core: &TerminalCore) -> Self {
        Self {
            damage: DamageSet::new(core.limits.pending_damage),
            events: Vec::new(),
            replies: Vec::new(),
            recognized: false,
        }
    }

    pub fn damage(&self) -> &DamageSet {
        &self.damage
    }

    pub fn events(&self) -> &[CoreEvent] {
        &self.events
    }

    pub fn replies(&self) -> &[TerminalReply] {
        &self.replies
    }

    pub const fn recognized(&self) -> bool {
        self.recognized
    }
}

impl TerminalCore {
    pub fn apply(&mut self, action: Action) -> Result<CoreUpdate, CoreError> {
        let mut update = CoreUpdate::new(self);
        if !matches!(action, Action::Print(_)) {
            self.state.grapheme_anchor = None;
        }
        match action {
            Action::Print(character) => self.apply_print(character, &mut update)?,
            Action::Execute(control) => self.apply_control(control.byte(), &mut update)?,
            Action::Escape(sequence) => self.apply_escape(sequence, &mut update)?,
            Action::Csi(sequence) => self.apply_csi(sequence, &mut update)?,
            Action::ControlString(string) => self.apply_control_string(string, &mut update)?,
            Action::Error(ParseError::LimitExceeded(kind)) => {
                self.push_event(CoreEvent::LimitReached(kind), &mut update)?;
            }
            Action::Cancelled { .. } | Action::Error(_) => {}
        }
        Ok(update)
    }

    pub(crate) fn apply_print(
        &mut self,
        character: char,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let mapped = self.state.charsets.map(character);
        if let Some(anchor) = self.state.grapheme_anchor
            && let Some(crate::Cell {
                content: crate::CellContent::Cluster(existing),
                ..
            }) = self.state.cell(anchor.row.get(), anchor.column.get())
            && !crate::grapheme_break_before(existing.text(), mapped)
        {
            let mut text = existing.text().to_owned();
            text.push(mapped);
            let width = crate::terminal_cluster_width(&text);
            let cluster = Cluster::new(text, width, self.limits.cluster_bytes)
                .map_err(|_| CoreError::InvalidPrintable)?;
            return self.apply_screen(
                ScreenAction::AppendCluster {
                    at: anchor,
                    cluster,
                },
                update,
            );
        }
        let text = mapped.to_string();
        let width = crate::terminal_cluster_width(&text);
        let cluster = Cluster::new(text, width, self.limits.cluster_bytes)
            .map_err(|_| CoreError::InvalidPrintable)?;
        self.apply_screen(ScreenAction::Print(cluster), update)
    }

    fn apply_control(&mut self, byte: u8, update: &mut CoreUpdate) -> Result<(), CoreError> {
        match byte {
            0x07 => self.push_event(CoreEvent::Bell, update)?,
            0x08 => self.apply_screen(ScreenAction::Backspace, update)?,
            0x09 => self.apply_screen(ScreenAction::HorizontalTab, update)?,
            0x0a..=0x0c | 0x84 => self.apply_screen(ScreenAction::LineFeed, update)?,
            0x0d => self.apply_screen(ScreenAction::CarriageReturn, update)?,
            0x0e => {
                self.state.charsets.active = CharacterSetSlot::G1;
                update.recognized = true;
            }
            0x0f => {
                self.state.charsets.active = CharacterSetSlot::G0;
                update.recognized = true;
            }
            0x85 => {
                self.apply_screen(ScreenAction::CarriageReturn, update)?;
                self.apply_screen(ScreenAction::LineFeed, update)?;
            }
            0x88 => {
                self.state.tabs.set(self.state.cursor.position.column);
                update.recognized = true;
            }
            0x8d => self.apply_screen(ScreenAction::ReverseIndex, update)?,
            0x00..=0x1f | 0x7f..=0x9f => update.recognized = true,
            _ => {}
        }
        Ok(())
    }

    fn apply_escape(
        &mut self,
        sequence: EscapeSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        match (sequence.intermediates.as_slice(), sequence.final_byte) {
            ([], b'7') => self.apply_screen(ScreenAction::SaveCursor, update)?,
            ([], b'8') => self.apply_screen(ScreenAction::RestoreCursor, update)?,
            ([], b'D') => self.apply_screen(ScreenAction::LineFeed, update)?,
            ([], b'E') => {
                self.apply_screen(ScreenAction::CarriageReturn, update)?;
                self.apply_screen(ScreenAction::LineFeed, update)?;
            }
            ([], b'H') => {
                self.state.tabs.set(self.state.cursor.position.column);
                update.recognized = true;
            }
            ([], b'M') => self.apply_screen(ScreenAction::ReverseIndex, update)?,
            ([], b'Z') => {
                self.push_reply(ReplyKind::DeviceAttributes, b"\x1b[?1;2c".to_vec(), update)?
            }
            ([], b'=') => {
                self.state.modes.application_keypad = true;
                update.recognized = true;
            }
            ([], b'>') => {
                self.state.modes.application_keypad = false;
                update.recognized = true;
            }
            ([], b'c') => {
                self.apply_screen(ScreenAction::Reset, update)?;
                self.reset_p10_state();
            }
            ([b'('], final_byte) => {
                update.recognized = self.designate_charset(CharacterSetSlot::G0, final_byte);
            }
            ([b')'], final_byte) => {
                update.recognized = self.designate_charset(CharacterSetSlot::G1, final_byte);
            }
            ([b'%'], b'G' | b'8') => update.recognized = true,
            _ => {}
        }
        Ok(())
    }

    fn designate_charset(&mut self, slot: CharacterSetSlot, final_byte: u8) -> bool {
        let set = match final_byte {
            b'B' => crate::CharacterSet::Ascii,
            b'0' => crate::CharacterSet::DecSpecialGraphics,
            _ => return false,
        };
        self.state.charsets.designate(slot, set);
        true
    }

    pub(crate) fn apply_screen(
        &mut self,
        action: ScreenAction,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let reduction = self.reduce(action)?;
        update.recognized = true;
        if self.state.modes.synchronized_output {
            self.state.synchronized_dirty = true;
            return Ok(());
        }
        for damage in reduction.damage().iter() {
            self.push_damage(damage, update)?;
        }
        Ok(())
    }

    pub(crate) fn apply_state_action(
        &mut self,
        action: ScreenAction,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        self.reduce(action)?;
        update.recognized = true;
        Ok(())
    }

    pub(crate) fn push_damage(
        &mut self,
        damage: Damage,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        if self.state.modes.synchronized_output {
            self.state.synchronized_dirty = true;
            return Ok(());
        }
        update.damage.push_coalesced(damage);
        Ok(())
    }

    pub(crate) fn push_event(
        &self,
        event: CoreEvent,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let used = self
            .limits
            .pending_events
            .checked_total(update.events.len(), update.replies.len())?;
        self.limits.pending_events.checked_total(used, 1)?;
        update.events.push(event);
        update.recognized = true;
        Ok(())
    }

    pub(crate) fn push_reply(
        &self,
        kind: ReplyKind,
        bytes: Vec<u8>,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let used = self
            .limits
            .pending_events
            .checked_total(update.events.len(), update.replies.len())?;
        self.limits.pending_events.checked_total(used, 1)?;
        update
            .replies
            .push(TerminalReply::new(kind, bytes, self.limits.reply_bytes)?);
        update.recognized = true;
        Ok(())
    }

    pub(crate) fn end_synchronized_output(
        &mut self,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        self.state.modes.synchronized_output = false;
        if self.state.synchronized_dirty {
            self.state.synchronized_dirty = false;
            self.push_damage(Damage::Full, update)?;
        }
        update.recognized = true;
        Ok(())
    }

    fn reset_p10_state(&mut self) {
        self.state.title = None;
        self.state.working_directory = None;
        self.state.palette = [crate::Color::Default; 256];
        self.state.default_foreground = crate::Color::Default;
        self.state.default_background = crate::Color::Default;
        self.state.cursor_color = crate::Color::Default;
        self.state.mouse_tracking = crate::MouseTracking::Off;
        self.state.mouse_encoding = crate::MouseEncoding::Default;
        self.state.kitty_keyboard = crate::KittyKeyboardState::default();
    }
}
