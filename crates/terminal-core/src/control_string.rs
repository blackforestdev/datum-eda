use crate::{
    CellAttribute, ClipboardBytes, ClipboardSelection, Color, ControlString, ControlStringKind,
    CoreError, CoreEvent, CoreUpdate, CursorShape, Damage, NotificationText, PaletteIndex, Percent,
    ProgressState, ReplyKind, Rgb, ShellMark, TerminalCore, TitleText, UnderlineStyle,
    WorkingDirectoryText,
};

impl TerminalCore {
    pub(crate) fn apply_control_string(
        &mut self,
        string: ControlString,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        match string.kind {
            ControlStringKind::Osc => self.apply_osc(&string.bytes, update),
            ControlStringKind::Dcs => self.apply_dcs(&string.bytes, update),
            ControlStringKind::Apc => self.apply_kitty_graphics(&string.bytes, update),
            ControlStringKind::Pm | ControlStringKind::Sos => Ok(()),
        }
    }

    fn apply_osc(&mut self, bytes: &[u8], update: &mut CoreUpdate) -> Result<(), CoreError> {
        let mut fields = bytes.split(|byte| *byte == b';');
        let Some(command) = fields.next().and_then(parse_usize) else {
            return Ok(());
        };
        match command {
            0..=2 => self.set_title(fields.collect(), update)?,
            4 => self.palette_command(fields.collect(), update)?,
            7 => self.set_working_directory(fields.collect(), update)?,
            8 => self.set_hyperlink(fields.collect())?,
            9 => self.notification_or_progress(fields.collect(), update)?,
            10..=12 => self.default_color_command(command, fields.collect(), update)?,
            52 => self.clipboard_request(fields.collect(), update)?,
            133 => self.shell_mark(fields.collect(), update)?,
            104 => self.reset_palette(fields.collect(), update)?,
            110..=112 => self.reset_default_color(command, update)?,
            777 => self.extended_notification(fields.collect(), update)?,
            _ => {}
        }
        Ok(())
    }

    fn set_hyperlink(&mut self, fields: Vec<&[u8]>) -> Result<(), CoreError> {
        let parameters = fields.first().copied().unwrap_or_default();
        let Some(uri) = join_fields(fields.get(1..).unwrap_or_default()) else {
            return Ok(());
        };
        if uri.is_empty() {
            self.state.current_hyperlink = None;
            return Ok(());
        }
        let (Ok(parameters), Ok(uri)) = (
            String::from_utf8(parameters.to_vec()),
            String::from_utf8(uri),
        ) else {
            return Ok(());
        };
        self.state.current_hyperlink = Some(self.state.hyperlinks.insert(parameters, uri)?);
        Ok(())
    }

    fn clipboard_request(
        &mut self,
        fields: Vec<&[u8]>,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let selections = fields.first().copied().unwrap_or(b"c");
        let Some(contents) = join_fields(fields.get(1..).unwrap_or_default()) else {
            return Ok(());
        };
        update.recognized = true;
        if contents == b"?" {
            return Ok(());
        }
        let contents = ClipboardBytes::new(contents, self.limits.clipboard_bytes)?;
        let selections = if selections.is_empty() {
            b"c"
        } else {
            selections
        };
        for selection in selections.iter().filter_map(|selection| match selection {
            b'c' => Some(ClipboardSelection::Clipboard),
            b'p' => Some(ClipboardSelection::Primary),
            b's' => Some(ClipboardSelection::Select),
            _ => None,
        }) {
            self.push_event(
                CoreEvent::ClipboardRequest {
                    selection,
                    encoded_contents: contents.clone(),
                },
                update,
            )?;
        }
        Ok(())
    }

    fn shell_mark(&mut self, fields: Vec<&[u8]>, update: &mut CoreUpdate) -> Result<(), CoreError> {
        let Some(mark) = fields.first().and_then(|field| match *field {
            b"A" => Some(ShellMark::PromptStart),
            b"B" => Some(ShellMark::CommandStart),
            b"C" => Some(ShellMark::CommandExecuted),
            b"D" => Some(ShellMark::CommandFinished {
                exit_code: fields.get(1).and_then(|value| parse_i32(value)),
            }),
            _ => None,
        }) else {
            return Ok(());
        };
        self.state.shell_mark = Some(mark);
        self.push_event(CoreEvent::ShellMark(mark), update)
    }

    fn notification_or_progress(
        &mut self,
        fields: Vec<&[u8]>,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        if fields.first().copied() == Some(b"4") {
            return self.set_progress(fields.get(1..).unwrap_or_default(), update);
        }
        self.emit_notification(&fields, update)
    }

    fn extended_notification(
        &mut self,
        fields: Vec<&[u8]>,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        if fields.first().copied() != Some(b"notify") {
            return Ok(());
        }
        self.emit_notification(fields.get(1..).unwrap_or_default(), update)
    }

    fn emit_notification(
        &mut self,
        fields: &[&[u8]],
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let Some(text) = join_fields(fields).and_then(|bytes| String::from_utf8(bytes).ok()) else {
            return Ok(());
        };
        let notification = NotificationText::new(text, self.limits.notification_bytes)?;
        self.push_event(CoreEvent::Notification(notification), update)
    }

    fn set_progress(&mut self, fields: &[&[u8]], update: &mut CoreUpdate) -> Result<(), CoreError> {
        let percent = fields
            .get(1)
            .and_then(|value| parse_usize(value))
            .and_then(|value| u8::try_from(value).ok())
            .and_then(Percent::new);
        let Some(progress) = fields.first().and_then(|state| match *state {
            b"0" => Some(ProgressState::Clear),
            b"1" => percent.map(|percent| ProgressState::Set { percent }),
            b"2" => percent.map(|percent| ProgressState::Error { percent }),
            b"3" => Some(ProgressState::Indeterminate),
            b"4" => percent.map(|percent| ProgressState::Paused { percent }),
            _ => None,
        }) else {
            return Ok(());
        };
        self.state.progress = progress;
        self.push_event(CoreEvent::Progress(progress), update)
    }

    fn set_title(&mut self, fields: Vec<&[u8]>, update: &mut CoreUpdate) -> Result<(), CoreError> {
        let Some(text) = join_fields(&fields).and_then(|bytes| String::from_utf8(bytes).ok())
        else {
            return Ok(());
        };
        let title = TitleText::new(text, self.limits.title_bytes)?;
        self.state.title = Some(title.clone());
        self.push_event(CoreEvent::TitleChanged(title), update)?;
        self.push_damage(Damage::Title, update)
    }

    fn set_working_directory(
        &mut self,
        fields: Vec<&[u8]>,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let Some(text) = join_fields(&fields).and_then(|bytes| String::from_utf8(bytes).ok())
        else {
            return Ok(());
        };
        let cwd = WorkingDirectoryText::new(text, self.limits.working_directory_bytes)?;
        self.state.working_directory = Some(cwd.clone());
        self.push_event(CoreEvent::WorkingDirectoryChanged(cwd), update)
    }

    fn palette_command(
        &mut self,
        fields: Vec<&[u8]>,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        for pair in fields.chunks_exact(2) {
            let Some(index) = parse_usize(pair[0]).and_then(|value| u8::try_from(value).ok())
            else {
                continue;
            };
            if pair[1] == b"?" {
                let color = self.state.palette[index as usize];
                if let Some(specification) = color_specification(color) {
                    self.push_reply(
                        ReplyKind::ColorReport,
                        format!("\x1b]4;{index};{specification}\x1b\\").into_bytes(),
                        update,
                    )?;
                }
                continue;
            }
            let Some(color) = parse_color(pair[1]) else {
                continue;
            };
            self.state.palette[index as usize] = color;
            self.push_event(
                CoreEvent::PaletteChanged {
                    index: PaletteIndex::new(index),
                    color,
                },
                update,
            )?;
            self.push_damage(Damage::Palette(PaletteIndex::new(index)), update)?;
        }
        Ok(())
    }

    fn reset_palette(
        &mut self,
        fields: Vec<&[u8]>,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        if fields.is_empty() || fields.iter().all(|field| field.is_empty()) {
            self.state.palette = [Color::Default; 256];
            self.push_damage(Damage::Full, update)?;
            update.recognized = true;
            return Ok(());
        }
        for field in fields {
            let Some(index) = parse_usize(field).and_then(|value| u8::try_from(value).ok()) else {
                continue;
            };
            self.state.palette[index as usize] = Color::Default;
            self.push_damage(Damage::Palette(PaletteIndex::new(index)), update)?;
            update.recognized = true;
        }
        Ok(())
    }

    fn default_color_command(
        &mut self,
        command: usize,
        fields: Vec<&[u8]>,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let Some(field) = fields.first() else {
            return Ok(());
        };
        if *field == b"?" {
            let color = self.default_color(command);
            if let Some(specification) = color_specification(color) {
                self.push_reply(
                    ReplyKind::ColorReport,
                    format!("\x1b]{command};{specification}\x1b\\").into_bytes(),
                    update,
                )?;
            }
            return Ok(());
        }
        let Some(color) = parse_color(field) else {
            return Ok(());
        };
        *self.default_color_mut(command) = color;
        self.push_damage(Damage::Full, update)?;
        update.recognized = true;
        Ok(())
    }

    fn reset_default_color(
        &mut self,
        command: usize,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        *self.default_color_mut(command - 100) = Color::Default;
        self.push_damage(Damage::Full, update)?;
        update.recognized = true;
        Ok(())
    }

    fn default_color(&self, command: usize) -> Color {
        match command {
            10 => self.state.default_foreground,
            11 => self.state.default_background,
            12 => self.state.cursor_color,
            _ => Color::Default,
        }
    }

    fn default_color_mut(&mut self, command: usize) -> &mut Color {
        match command {
            10 => &mut self.state.default_foreground,
            11 => &mut self.state.default_background,
            12 => &mut self.state.cursor_color,
            _ => unreachable!("caller validates default-color OSC command"),
        }
    }

    fn apply_dcs(&mut self, bytes: &[u8], update: &mut CoreUpdate) -> Result<(), CoreError> {
        if let Some(query) = bytes.strip_prefix(b"$q") {
            let response = self.status_string(query);
            let valid = response.is_some();
            let body = response.unwrap_or_default();
            return self.push_reply(
                ReplyKind::DeviceStatus,
                format!("\x1bP{}$r{body}\x1b\\", usize::from(valid)).into_bytes(),
                update,
            );
        }
        if bytes.starts_with(b"+q") {
            return self.push_reply(ReplyKind::DeviceStatus, b"\x1bP0+r\x1b\\".to_vec(), update);
        }
        if let Some((parameters, data)) = sixel_body(bytes) {
            return self.apply_sixel(&parameters, data, update);
        }
        Ok(())
    }

    fn apply_sixel(
        &mut self,
        parameters: &[usize],
        data: &[u8],
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let background =
            (parameters.get(1).copied().unwrap_or(0) != 1).then(|| sixel_background(&self.state));
        let mut colors = if self.state.modes.sixel_private_colors {
            crate::SixelColorRegisters::default()
        } else {
            self.state.sixel_colors.clone()
        };
        let image = crate::decode_sixel(
            data,
            background,
            &mut colors,
            sixel_aspect(parameters.first().copied().unwrap_or(0)),
            crate::SixelLimits {
                pixels: self.limits.graphic_pixels,
                decoded_bytes: self.limits.graphic_decoded_bytes,
                work: self.limits.parser_work,
            },
        )?;
        update.recognized = true;
        if image.width == 0 || image.height == 0 {
            if !self.state.modes.sixel_private_colors {
                self.state.sixel_colors = colors;
            }
            return Ok(());
        }
        let pending = self
            .limits
            .pending_events
            .checked_total(update.events.len(), update.replies.len())?;
        self.limits.pending_events.checked_total(pending, 1)?;
        let anchor = self
            .state
            .logical_point_at(
                self.state.cursor.position.row.get(),
                self.state.cursor.position.column.get(),
            )
            .expect("cursor belongs to active grid");
        let width = image.width;
        let height = image.height;
        let id = self
            .state
            .graphics
            .insert_sixel(self.state.active_buffer, anchor, image)?;
        if !self.state.modes.sixel_private_colors {
            self.state.sixel_colors = colors;
        }
        self.push_event(CoreEvent::GraphicAdded(id), update)?;
        self.push_damage(Damage::Graphics, update)?;
        self.advance_after_sixel(width, height, update)
    }

    fn advance_after_sixel(
        &mut self,
        width: u32,
        height: u32,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let cell_width = cell_pixel_extent(
            self.state.size.pixels.width,
            u32::from(self.state.size.columns.get()),
            1,
        );
        let cell_height = cell_pixel_extent(
            self.state.size.pixels.height,
            u32::from(self.state.size.rows.get()),
            6,
        );
        if self.state.modes.sixel_cursor_right {
            let columns = width.div_ceil(cell_width).min(u32::from(u16::MAX)) as i32;
            return self.apply_screen(crate::ScreenAction::MoveCursor { rows: 0, columns }, update);
        }
        if self.state.modes.sixel_scrolling {
            self.apply_screen(crate::ScreenAction::CarriageReturn, update)?;
            for _ in 0..height.div_ceil(cell_height) {
                self.apply_screen(crate::ScreenAction::LineFeed, update)?;
            }
        }
        Ok(())
    }

    fn status_string(&self, query: &[u8]) -> Option<String> {
        match query {
            b"m" => Some(style_status(self.state.style)),
            b"r" => Some(format!(
                "{};{}r",
                self.state.margins.top.get() + 1,
                self.state.margins.bottom.get() + 1
            )),
            b"\"q" => Some(format!("{}\"q", usize::from(self.state.protected))),
            b" q" => Some(format!("{} q", cursor_style_status(self.state.cursor))),
            _ => None,
        }
    }
}

fn parse_usize(bytes: &[u8]) -> Option<usize> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

fn sixel_body(bytes: &[u8]) -> Option<(Vec<usize>, &[u8])> {
    let final_index = bytes.iter().position(|byte| *byte == b'q')?;
    let introducer = &bytes[..final_index];
    if !introducer
        .iter()
        .all(|byte| byte.is_ascii_digit() || *byte == b';')
    {
        return None;
    }
    let parameters = if introducer.is_empty() {
        Vec::new()
    } else {
        introducer
            .split(|byte| *byte == b';')
            .map(|field| {
                if field.is_empty() {
                    Some(0)
                } else {
                    parse_usize(field)
                }
            })
            .collect::<Option<Vec<_>>>()?
    };
    (parameters.len() <= 3).then_some((parameters, &bytes[final_index + 1..]))
}

fn sixel_aspect(macro_parameter: usize) -> crate::PixelAspect {
    let (numerator, denominator) = match macro_parameter {
        2 => (5, 1),
        3 | 4 => (3, 1),
        7..=9 => (1, 1),
        _ => (2, 1),
    };
    crate::PixelAspect::new(numerator, denominator).expect("DEC aspect ratios are nonzero")
}

fn sixel_background(state: &crate::ScreenState) -> crate::Rgba8 {
    let color = match state.default_background {
        Color::Default => {
            return crate::Rgba8 {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            };
        }
        Color::Indexed(index) => state.palette[index.get() as usize],
        color => color,
    };
    match color {
        Color::Rgb(rgb) => crate::Rgba8 {
            red: rgb.red,
            green: rgb.green,
            blue: rgb.blue,
            alpha: 255,
        },
        Color::Default | Color::Indexed(_) => crate::Rgba8 {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 255,
        },
    }
}

fn cell_pixel_extent(total: u32, cells: u32, fallback: u32) -> u32 {
    if total == 0 {
        fallback
    } else {
        (total / cells).max(1)
    }
}

fn parse_i32(bytes: &[u8]) -> Option<i32> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

fn join_fields(fields: &[&[u8]]) -> Option<Vec<u8>> {
    let length = fields.iter().try_fold(0usize, |length, field| {
        length.checked_add(field.len())?.checked_add(1)
    })?;
    let mut joined = Vec::new();
    joined.try_reserve_exact(length.saturating_sub(1)).ok()?;
    for (index, field) in fields.iter().enumerate() {
        if index != 0 {
            joined.push(b';');
        }
        joined.extend_from_slice(field);
    }
    Some(joined)
}

fn parse_color(bytes: &[u8]) -> Option<Color> {
    if let Some(hex) = bytes.strip_prefix(b"#") {
        if hex.len() != 6 {
            return None;
        }
        return Some(Color::Rgb(Rgb {
            red: parse_hex(&hex[0..2])?,
            green: parse_hex(&hex[2..4])?,
            blue: parse_hex(&hex[4..6])?,
        }));
    }
    let rgb = bytes.strip_prefix(b"rgb:")?;
    let mut components = rgb.split(|byte| *byte == b'/');
    Some(Color::Rgb(Rgb {
        red: scale_hex(components.next()?)?,
        green: scale_hex(components.next()?)?,
        blue: scale_hex(components.next()?)?,
    }))
}

fn parse_hex(bytes: &[u8]) -> Option<u8> {
    u8::from_str_radix(std::str::from_utf8(bytes).ok()?, 16).ok()
}

fn scale_hex(bytes: &[u8]) -> Option<u8> {
    if bytes.is_empty() || bytes.len() > 4 {
        return None;
    }
    let value = u16::from_str_radix(std::str::from_utf8(bytes).ok()?, 16).ok()?;
    let maximum = (1u32 << (bytes.len() * 4)) - 1;
    Some(((u32::from(value) * 255 + maximum / 2) / maximum) as u8)
}

fn color_specification(color: Color) -> Option<String> {
    let Color::Rgb(rgb) = color else {
        return None;
    };
    Some(format!(
        "rgb:{0:02x}{0:02x}/{1:02x}{1:02x}/{2:02x}{2:02x}",
        rgb.red, rgb.green, rgb.blue
    ))
}

fn style_status(style: crate::CellStyle) -> String {
    let mut values = Vec::new();
    if style.attributes.contains(CellAttribute::Bold) {
        values.push("1".to_owned());
    }
    if style.attributes.contains(CellAttribute::Faint) {
        values.push("2".to_owned());
    }
    if style.attributes.contains(CellAttribute::Italic) {
        values.push("3".to_owned());
    }
    match style.underline {
        UnderlineStyle::None => {}
        UnderlineStyle::Single => values.push("4".to_owned()),
        UnderlineStyle::Double => values.push("4:2".to_owned()),
        UnderlineStyle::Curly => values.push("4:3".to_owned()),
        UnderlineStyle::Dotted => values.push("4:4".to_owned()),
        UnderlineStyle::Dashed => values.push("4:5".to_owned()),
    }
    if style.attributes.contains(CellAttribute::Blink) {
        values.push("5".to_owned());
    }
    if style.attributes.contains(CellAttribute::Inverse) {
        values.push("7".to_owned());
    }
    if style.attributes.contains(CellAttribute::Hidden) {
        values.push("8".to_owned());
    }
    if style.attributes.contains(CellAttribute::Strike) {
        values.push("9".to_owned());
    }
    if style.attributes.contains(CellAttribute::Overline) {
        values.push("53".to_owned());
    }
    values.extend(color_status(38, style.foreground));
    values.extend(color_status(48, style.background));
    values.extend(color_status(58, style.underline_color));
    if values.is_empty() {
        "0m".to_owned()
    } else {
        format!("{}m", values.join(";"))
    }
}

fn color_status(prefix: usize, color: Color) -> Vec<String> {
    match color {
        Color::Default => Vec::new(),
        Color::Indexed(index) => vec![format!("{prefix};5;{}", index.get())],
        Color::Rgb(rgb) => vec![format!("{prefix};2;{};{};{}", rgb.red, rgb.green, rgb.blue)],
    }
}

fn cursor_style_status(cursor: crate::CursorState) -> usize {
    match (cursor.shape, cursor.blinking) {
        (CursorShape::Block, true) => 1,
        (CursorShape::Block, false) => 2,
        (CursorShape::Underline, true) => 3,
        (CursorShape::Underline, false) => 4,
        (CursorShape::Bar, true) => 5,
        (CursorShape::Bar, false) => 6,
    }
}
