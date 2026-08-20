use crate::{
    CoreError, CoreUpdate, CsiSequence, CursorShape, Damage, EraseDisplay, EraseLine,
    FoundationMode, Margins, MouseEncoding, MouseTracking, ReplyKind, ScreenAction, ScreenBuffer,
    TerminalCore,
};

impl TerminalCore {
    pub(crate) fn apply_csi(
        &mut self,
        sequence: CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let private = sequence.private_markers.as_slice();
        let intermediates = sequence.intermediates.as_slice();
        match (private, intermediates, sequence.final_byte) {
            ([], [], b'A') => self.csi_move_cursor(-count(&sequence, 0), 0, update)?,
            ([], [], b'B' | b'e') => self.csi_move_cursor(count(&sequence, 0), 0, update)?,
            ([], [], b'C' | b'a') => self.csi_move_cursor(0, count(&sequence, 0), update)?,
            ([], [], b'D') => self.csi_move_cursor(0, -count(&sequence, 0), update)?,
            ([], [], b'E') => {
                self.csi_move_cursor(count(&sequence, 0), 0, update)?;
                self.apply_screen(ScreenAction::CarriageReturn, update)?;
            }
            ([], [], b'F') => {
                self.csi_move_cursor(-count(&sequence, 0), 0, update)?;
                self.apply_screen(ScreenAction::CarriageReturn, update)?;
            }
            ([], [], b'G' | b'`') => self.set_column(one_based(&sequence, 0), update)?,
            ([], [], b'd') => self.set_row(one_based(&sequence, 0), update)?,
            ([], [], b'H' | b'f') => {
                self.set_position(one_based(&sequence, 0), one_based(&sequence, 1), update)?
            }
            ([], [], b'I') => self.tab_forward(count(&sequence, 0), update)?,
            ([], [], b'Z') => self.tab_backward(count(&sequence, 0), update)?,
            ([], [], b'@') => {
                self.apply_screen(ScreenAction::InsertCells(count_u16(&sequence, 0)), update)?
            }
            ([], [], b'P') => {
                self.apply_screen(ScreenAction::DeleteCells(count_u16(&sequence, 0)), update)?
            }
            ([], [], b'X') => {
                self.apply_screen(ScreenAction::EraseCells(count_u16(&sequence, 0)), update)?
            }
            ([], [], b'L') => {
                self.apply_screen(ScreenAction::InsertLines(count_u16(&sequence, 0)), update)?
            }
            ([], [], b'M') => {
                self.apply_screen(ScreenAction::DeleteLines(count_u16(&sequence, 0)), update)?
            }
            ([], [], b'S') => {
                self.apply_screen(ScreenAction::ScrollUp(count_u16(&sequence, 0)), update)?
            }
            ([], [], b'T') => {
                self.apply_screen(ScreenAction::ScrollDown(count_u16(&sequence, 0)), update)?
            }
            ([], [], b'J') | ([b'?'], [], b'J') => {
                self.csi_erase_display(&sequence, update)?;
            }
            ([], [], b'K') | ([b'?'], [], b'K') => {
                self.csi_erase_line(&sequence, update)?;
            }
            ([], [], b'm') => self.set_graphic_rendition(&sequence, update)?,
            ([], [], b'r') => self.set_vertical_margins(&sequence, update)?,
            ([], [], b's') if sequence.parameters.len() >= 2 => {
                self.set_horizontal_margins(&sequence, update)?;
            }
            ([], [], b's') => self.apply_screen(ScreenAction::SaveCursor, update)?,
            ([], [], b'u') if sequence.parameters.is_empty() => {
                self.apply_screen(ScreenAction::RestoreCursor, update)?
            }
            ([b'>'], [], b'u') => self.push_kitty_keyboard(&sequence, update)?,
            ([b'<'], [], b'u') => self.pop_kitty_keyboard(&sequence, update),
            ([b'?'], [], b'u') => self.query_kitty_keyboard(update)?,
            ([b'='], [], b'u') => self.set_kitty_keyboard(&sequence, update),
            ([], [], b'g') => self.clear_tab_stop(&sequence, update),
            ([], [], b'b') => self.repeat_last(&sequence, update)?,
            ([], [], b'h' | b'l') | ([b'?'], [], b'h' | b'l') => {
                self.set_modes(&sequence, update)?;
            }
            ([], [], b'n') | ([b'?'], [], b'n') => self.device_status(&sequence, update)?,
            ([], [], b'c') | ([b'>'], [], b'c') | ([b'='], [], b'c') => {
                self.device_attributes(&sequence, update)?;
            }
            ([], [b'$'], b'p') | ([b'?'], [b'$'], b'p') => {
                self.report_mode(&sequence, update)?;
            }
            ([], [b' '], b'q') => self.set_cursor_style(&sequence, update)?,
            ([], [b'"'], b'q') => self.set_character_protection(&sequence, update)?,
            ([], [], b't') => self.window_report(&sequence, update)?,
            _ => {}
        }
        Ok(())
    }

    fn csi_move_cursor(
        &mut self,
        rows: i32,
        columns: i32,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        self.apply_screen(ScreenAction::MoveCursor { rows, columns }, update)
    }

    fn set_position(
        &mut self,
        row: usize,
        column: usize,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let mut row = row.saturating_sub(1).min(usize::from(u16::MAX)) as u16;
        let mut column = column.saturating_sub(1).min(usize::from(u16::MAX)) as u16;
        if self.state.modes.origin {
            row = row.saturating_add(self.state.margins.top.get());
            column = column.saturating_add(self.state.margins.left.get());
        }
        self.apply_screen(ScreenAction::SetCursor { row, column }, update)
    }

    fn set_column(&mut self, column: usize, update: &mut CoreUpdate) -> Result<(), CoreError> {
        let mut column = column.saturating_sub(1).min(usize::from(u16::MAX)) as u16;
        if self.state.modes.origin {
            column = column.saturating_add(self.state.margins.left.get());
        }
        self.apply_screen(
            ScreenAction::SetCursor {
                row: self.state.cursor.position.row.get(),
                column,
            },
            update,
        )
    }

    fn set_row(&mut self, row: usize, update: &mut CoreUpdate) -> Result<(), CoreError> {
        let mut row = row.saturating_sub(1).min(usize::from(u16::MAX)) as u16;
        if self.state.modes.origin {
            row = row.saturating_add(self.state.margins.top.get());
        }
        self.apply_screen(
            ScreenAction::SetCursor {
                row,
                column: self.state.cursor.position.column.get(),
            },
            update,
        )
    }

    fn tab_forward(&mut self, count: i32, update: &mut CoreUpdate) -> Result<(), CoreError> {
        for _ in 0..count {
            self.apply_screen(ScreenAction::HorizontalTab, update)?;
        }
        Ok(())
    }

    fn tab_backward(&mut self, count: i32, update: &mut CoreUpdate) -> Result<(), CoreError> {
        for _ in 0..count {
            let current = self.state.cursor.position.column.get();
            let previous = self
                .state
                .tabs
                .iter()
                .map(crate::Column::get)
                .rev()
                .find(|column| *column < current)
                .unwrap_or(0);
            self.apply_screen(
                ScreenAction::SetCursor {
                    row: self.state.cursor.position.row.get(),
                    column: previous,
                },
                update,
            )?;
        }
        Ok(())
    }

    fn csi_erase_display(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let mode = match value(sequence, 0).unwrap_or(0) {
            0 => EraseDisplay::Below,
            1 => EraseDisplay::Above,
            2 | 3 => EraseDisplay::All,
            _ => return Ok(()),
        };
        self.apply_screen(
            ScreenAction::EraseDisplay {
                mode,
                selective: sequence.private_markers == [b'?'],
            },
            update,
        )
    }

    fn csi_erase_line(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let mode = match value(sequence, 0).unwrap_or(0) {
            0 => EraseLine::Right,
            1 => EraseLine::Left,
            2 => EraseLine::All,
            _ => return Ok(()),
        };
        self.apply_screen(
            ScreenAction::EraseLine {
                mode,
                selective: sequence.private_markers == [b'?'],
            },
            update,
        )
    }

    fn set_vertical_margins(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let rows = self.state.size.rows.get();
        let top = one_based(sequence, 0)
            .saturating_sub(1)
            .min(usize::from(rows - 1)) as u16;
        let bottom = value(sequence, 1)
            .unwrap_or(usize::from(rows))
            .saturating_sub(1)
            .min(usize::from(rows - 1)) as u16;
        if let Ok(margins) = Margins::new(
            top,
            bottom,
            self.state.margins.left.get(),
            self.state.margins.right.get(),
            self.state.size,
        ) {
            self.apply_screen(ScreenAction::SetMargins(margins), update)?;
        }
        Ok(())
    }

    fn set_horizontal_margins(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let columns = self.state.size.columns.get();
        let left = one_based(sequence, 0)
            .saturating_sub(1)
            .min(usize::from(columns - 1)) as u16;
        let right = value(sequence, 1)
            .unwrap_or(usize::from(columns))
            .saturating_sub(1)
            .min(usize::from(columns - 1)) as u16;
        if let Ok(margins) = Margins::new(
            self.state.margins.top.get(),
            self.state.margins.bottom.get(),
            left,
            right,
            self.state.size,
        ) {
            self.apply_screen(ScreenAction::SetMargins(margins), update)?;
        }
        Ok(())
    }

    fn clear_tab_stop(&mut self, sequence: &CsiSequence, update: &mut CoreUpdate) {
        match value(sequence, 0).unwrap_or(0) {
            0 => self.state.tabs.clear(self.state.cursor.position.column),
            3 => self.state.tabs.clear_all(),
            _ => return,
        }
        update.recognized = true;
    }

    fn repeat_last(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let Some(cluster) = self.state.last_printed.clone() else {
            return Ok(());
        };
        for _ in 0..count(sequence, 0) {
            self.apply_screen(ScreenAction::Print(cluster.clone()), update)?;
        }
        Ok(())
    }

    fn set_modes(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let enabled = sequence.final_byte == b'h';
        for parameter in &sequence.parameters {
            let Some(mode) = parameter.subparameters.first().copied().flatten() else {
                continue;
            };
            if sequence.private_markers == [b'?'] {
                self.set_private_mode(mode, enabled, update)?;
            } else {
                self.set_standard_mode(mode, enabled, update)?;
            }
        }
        Ok(())
    }

    fn set_standard_mode(
        &mut self,
        mode: usize,
        enabled: bool,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let mode = match mode {
            4 => FoundationMode::Insert,
            20 => FoundationMode::Newline,
            _ => return Ok(()),
        };
        self.apply_state_action(ScreenAction::SetMode { mode, enabled }, update)
    }

    fn set_private_mode(
        &mut self,
        mode: usize,
        enabled: bool,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        match mode {
            1 => {
                self.state.modes.application_cursor = enabled;
                update.recognized = true;
            }
            6 => {
                self.apply_state_action(
                    ScreenAction::SetMode {
                        mode: FoundationMode::Origin,
                        enabled,
                    },
                    update,
                )?;
                self.set_position(1, 1, update)?;
            }
            7 => self.apply_state_action(
                ScreenAction::SetMode {
                    mode: FoundationMode::AutoWrap,
                    enabled,
                },
                update,
            )?,
            12 => {
                self.state.cursor.blinking = enabled;
                self.push_damage(Damage::Cursor, update)?;
                update.recognized = true;
            }
            25 => {
                self.state.cursor.visible = enabled;
                self.push_damage(Damage::Cursor, update)?;
                update.recognized = true;
            }
            47 | 1047 => self.apply_screen(
                ScreenAction::SwitchBuffer {
                    buffer: if enabled {
                        ScreenBuffer::Alternate
                    } else {
                        ScreenBuffer::Primary
                    },
                    clear: mode == 1047 && enabled,
                    home: false,
                },
                update,
            )?,
            1048 => self.apply_screen(
                if enabled {
                    ScreenAction::SaveCursor
                } else {
                    ScreenAction::RestoreCursor
                },
                update,
            )?,
            1049 => {
                self.apply_screen(
                    if enabled {
                        ScreenAction::SaveCursor
                    } else {
                        ScreenAction::SwitchBuffer {
                            buffer: ScreenBuffer::Primary,
                            clear: false,
                            home: false,
                        }
                    },
                    update,
                )?;
                if enabled {
                    self.apply_screen(
                        ScreenAction::SwitchBuffer {
                            buffer: ScreenBuffer::Alternate,
                            clear: true,
                            home: true,
                        },
                        update,
                    )?;
                } else {
                    self.apply_screen(ScreenAction::RestoreCursor, update)?;
                }
            }
            66 => {
                self.state.modes.application_keypad = enabled;
                update.recognized = true;
            }
            9 => self.set_mouse_tracking(MouseTracking::X10, enabled, update),
            1000 => self.set_mouse_tracking(MouseTracking::Button, enabled, update),
            1002 => self.set_mouse_tracking(MouseTracking::Drag, enabled, update),
            1003 => self.set_mouse_tracking(MouseTracking::Any, enabled, update),
            1005 => self.set_mouse_encoding(MouseEncoding::Utf8, enabled, update),
            1006 => self.set_mouse_encoding(MouseEncoding::Sgr, enabled, update),
            1015 => self.set_mouse_encoding(MouseEncoding::Urxvt, enabled, update),
            1016 => self.set_mouse_encoding(MouseEncoding::SgrPixels, enabled, update),
            1004 => {
                self.state.modes.focus_reporting = enabled;
                update.recognized = true;
            }
            2004 => {
                self.state.modes.bracketed_paste = enabled;
                update.recognized = true;
            }
            2026 if enabled => {
                self.state.modes.synchronized_output = true;
                update.recognized = true;
            }
            2026 => self.end_synchronized_output(update)?,
            _ => {}
        }
        Ok(())
    }

    fn device_status(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let private = sequence.private_markers == [b'?'];
        match value(sequence, 0).unwrap_or(0) {
            5 if !private => {
                self.push_reply(ReplyKind::DeviceStatus, b"\x1b[0n".to_vec(), update)?
            }
            6 => {
                let prefix = if private { "?" } else { "" };
                let bytes = format!(
                    "\x1b[{prefix}{};{}R",
                    self.state.cursor.position.row.get() + 1,
                    self.state.cursor.position.column.get() + 1
                )
                .into_bytes();
                self.push_reply(ReplyKind::CursorPosition, bytes, update)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn device_attributes(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let bytes = match sequence.private_markers.as_slice() {
            [] => b"\x1b[?1;2c".to_vec(),
            [b'>'] => b"\x1b[>0;1;0c".to_vec(),
            [b'='] => b"\x1bP!|00000000\x1b\\".to_vec(),
            _ => return Ok(()),
        };
        self.push_reply(ReplyKind::DeviceAttributes, bytes, update)
    }

    fn report_mode(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let mode = value(sequence, 0).unwrap_or(0);
        let private = sequence.private_markers == [b'?'];
        let status = self
            .mode_status(mode, private)
            .map_or(0, |set| if set { 1 } else { 2 });
        let marker = if private { "?" } else { "" };
        self.push_reply(
            ReplyKind::ModeReport,
            format!("\x1b[{marker}{mode};{status}$y").into_bytes(),
            update,
        )
    }

    fn mode_status(&self, mode: usize, private: bool) -> Option<bool> {
        if private {
            return match mode {
                1 => Some(self.state.modes.application_cursor),
                6 => Some(self.state.modes.origin),
                7 => Some(self.state.modes.auto_wrap),
                12 => Some(self.state.cursor.blinking),
                25 => Some(self.state.cursor.visible),
                47 | 1047 | 1049 => Some(self.state.active_buffer == ScreenBuffer::Alternate),
                66 => Some(self.state.modes.application_keypad),
                9 => Some(self.state.mouse_tracking == MouseTracking::X10),
                1000 => Some(self.state.mouse_tracking == MouseTracking::Button),
                1002 => Some(self.state.mouse_tracking == MouseTracking::Drag),
                1003 => Some(self.state.mouse_tracking == MouseTracking::Any),
                1005 => Some(self.state.mouse_encoding == MouseEncoding::Utf8),
                1006 => Some(self.state.mouse_encoding == MouseEncoding::Sgr),
                1015 => Some(self.state.mouse_encoding == MouseEncoding::Urxvt),
                1016 => Some(self.state.mouse_encoding == MouseEncoding::SgrPixels),
                1004 => Some(self.state.modes.focus_reporting),
                2004 => Some(self.state.modes.bracketed_paste),
                2026 => Some(self.state.modes.synchronized_output),
                _ => None,
            };
        }
        match mode {
            4 => Some(self.state.modes.insert),
            20 => Some(self.state.modes.newline),
            _ => None,
        }
    }

    fn set_cursor_style(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let value = value(sequence, 0).unwrap_or(0);
        let (shape, blinking) = match value {
            0 | 1 => (CursorShape::Block, true),
            2 => (CursorShape::Block, false),
            3 => (CursorShape::Underline, true),
            4 => (CursorShape::Underline, false),
            5 => (CursorShape::Bar, true),
            6 => (CursorShape::Bar, false),
            _ => return Ok(()),
        };
        self.state.cursor.shape = shape;
        self.state.cursor.blinking = blinking;
        self.push_damage(Damage::Cursor, update)?;
        update.recognized = true;
        Ok(())
    }

    fn set_character_protection(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let protected = matches!(value(sequence, 0).unwrap_or(0), 1);
        self.apply_state_action(ScreenAction::SetProtection(protected), update)
    }

    fn window_report(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let size = self.state.size;
        let bytes = match value(sequence, 0).unwrap_or(0) {
            14 => format!("\x1b[4;{};{}t", size.pixels.height, size.pixels.width).into_bytes(),
            18 => format!("\x1b[8;{};{}t", size.rows.get(), size.columns.get()).into_bytes(),
            _ => return Ok(()),
        };
        self.push_reply(ReplyKind::DeviceStatus, bytes, update)
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

fn count(sequence: &CsiSequence, index: usize) -> i32 {
    one_based(sequence, index).min(i32::MAX as usize) as i32
}

fn count_u16(sequence: &CsiSequence, index: usize) -> u16 {
    one_based(sequence, index).min(usize::from(u16::MAX)) as u16
}
