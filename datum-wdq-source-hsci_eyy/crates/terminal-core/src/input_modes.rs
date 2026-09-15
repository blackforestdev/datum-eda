use crate::{
    CoreError, CoreUpdate, CsiSequence, MouseEncoding, MouseTracking, ReplyKind, TerminalCore,
};

impl TerminalCore {
    pub(crate) fn set_mouse_tracking(
        &mut self,
        mode: MouseTracking,
        enabled: bool,
        update: &mut CoreUpdate,
    ) {
        if enabled {
            self.state.mouse_tracking = mode;
        } else if self.state.mouse_tracking == mode {
            self.state.mouse_tracking = MouseTracking::Off;
        }
        update.recognized = true;
    }

    pub(crate) fn set_mouse_encoding(
        &mut self,
        mode: MouseEncoding,
        enabled: bool,
        update: &mut CoreUpdate,
    ) {
        if enabled {
            self.state.mouse_encoding = mode;
        } else if self.state.mouse_encoding == mode {
            self.state.mouse_encoding = MouseEncoding::Default;
        }
        update.recognized = true;
    }

    pub(crate) fn push_kitty_keyboard(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        self.limits
            .keyboard_stack
            .check(self.state.kitty_keyboard.stack.len().saturating_add(1))?;
        let previous = self.state.kitty_keyboard.flags;
        self.state.kitty_keyboard.stack.push(previous);
        self.state.kitty_keyboard.flags = value(sequence, 0).unwrap_or(0).min(31) as u8;
        update.recognized = true;
        Ok(())
    }

    pub(crate) fn pop_kitty_keyboard(&mut self, sequence: &CsiSequence, update: &mut CoreUpdate) {
        for _ in 0..one_based(sequence, 0) {
            self.state.kitty_keyboard.flags =
                self.state.kitty_keyboard.stack.pop().unwrap_or_default();
        }
        update.recognized = true;
    }

    pub(crate) fn query_kitty_keyboard(&self, update: &mut CoreUpdate) -> Result<(), CoreError> {
        self.push_reply(
            ReplyKind::KeyboardProtocol,
            format!("\x1b[?{}u", self.state.kitty_keyboard.flags).into_bytes(),
            update,
        )
    }

    pub(crate) fn set_kitty_keyboard(&mut self, sequence: &CsiSequence, update: &mut CoreUpdate) {
        let flags = value(sequence, 0).unwrap_or(0).min(31) as u8;
        self.state.kitty_keyboard.flags = match value(sequence, 1).unwrap_or(1) {
            1 => flags,
            2 => self.state.kitty_keyboard.flags | flags,
            3 => self.state.kitty_keyboard.flags & !flags,
            _ => return,
        };
        update.recognized = true;
    }
}

fn value(sequence: &CsiSequence, index: usize) -> Option<usize> {
    sequence
        .parameters
        .get(index)?
        .subparameters
        .first()
        .copied()
        .flatten()
}

fn one_based(sequence: &CsiSequence, index: usize) -> usize {
    value(sequence, index)
        .filter(|value| *value != 0)
        .unwrap_or(1)
}
