use crate::{
    CellAttribute, CellStyle, Color, CoreError, CoreUpdate, CsiParameter, CsiSequence,
    PaletteIndex, Rgb, ScreenAction, TerminalCore, UnderlineStyle,
};

impl TerminalCore {
    pub(crate) fn set_graphic_rendition(
        &mut self,
        sequence: &CsiSequence,
        update: &mut CoreUpdate,
    ) -> Result<(), CoreError> {
        let mut style = self.state.style;
        let parameters = if sequence.parameters.is_empty() {
            vec![CsiParameter {
                subparameters: vec![Some(0)],
            }]
        } else {
            sequence.parameters.clone()
        };
        let mut index = 0;
        while index < parameters.len() {
            let parameter = &parameters[index];
            let code = parameter
                .subparameters
                .first()
                .copied()
                .flatten()
                .unwrap_or(0);
            if matches!(code, 38 | 48 | 58)
                && let Some((color, consumed)) = extended_color(&parameters, index)
            {
                match code {
                    38 => style.foreground = color,
                    48 => style.background = color,
                    58 => style.underline_color = color,
                    _ => unreachable!(),
                }
                index += consumed;
                continue;
            }
            apply_sgr_code(&mut style, code, parameter);
            index += 1;
        }
        self.apply_state_action(ScreenAction::SetStyle(style), update)
    }
}

fn apply_sgr_code(style: &mut CellStyle, code: usize, parameter: &CsiParameter) {
    match code {
        0 => *style = CellStyle::default(),
        1 => style.attributes.set(CellAttribute::Bold, true),
        2 => style.attributes.set(CellAttribute::Faint, true),
        3 => style.attributes.set(CellAttribute::Italic, true),
        4 => {
            style.underline = match parameter
                .subparameters
                .get(1)
                .copied()
                .flatten()
                .unwrap_or(1)
            {
                0 => UnderlineStyle::None,
                2 => UnderlineStyle::Double,
                3 => UnderlineStyle::Curly,
                4 => UnderlineStyle::Dotted,
                5 => UnderlineStyle::Dashed,
                _ => UnderlineStyle::Single,
            };
        }
        5 | 6 => style.attributes.set(CellAttribute::Blink, true),
        7 => style.attributes.set(CellAttribute::Inverse, true),
        8 => style.attributes.set(CellAttribute::Hidden, true),
        9 => style.attributes.set(CellAttribute::Strike, true),
        21 => style.underline = UnderlineStyle::Double,
        22 => {
            style.attributes.set(CellAttribute::Bold, false);
            style.attributes.set(CellAttribute::Faint, false);
        }
        23 => style.attributes.set(CellAttribute::Italic, false),
        24 => style.underline = UnderlineStyle::None,
        25 => style.attributes.set(CellAttribute::Blink, false),
        27 => style.attributes.set(CellAttribute::Inverse, false),
        28 => style.attributes.set(CellAttribute::Hidden, false),
        29 => style.attributes.set(CellAttribute::Strike, false),
        30..=37 => style.foreground = indexed((code - 30) as u8),
        39 => style.foreground = Color::Default,
        40..=47 => style.background = indexed((code - 40) as u8),
        49 => style.background = Color::Default,
        53 => style.attributes.set(CellAttribute::Overline, true),
        55 => style.attributes.set(CellAttribute::Overline, false),
        59 => style.underline_color = Color::Default,
        90..=97 => style.foreground = indexed((code - 90 + 8) as u8),
        100..=107 => style.background = indexed((code - 100 + 8) as u8),
        _ => {}
    }
}

fn indexed(index: u8) -> Color {
    Color::Indexed(PaletteIndex::new(index))
}

fn extended_color(parameters: &[CsiParameter], index: usize) -> Option<(Color, usize)> {
    let parameter = parameters.get(index)?;
    if parameter.subparameters.len() > 1 {
        let values = &parameter.subparameters;
        return match values.get(1).copied().flatten()? {
            5 => Some((
                indexed(u8::try_from(values.get(2).copied().flatten()?).ok()?),
                1,
            )),
            2 => {
                let components = &values[values.len().checked_sub(3)?..];
                Some((rgb(components)?, 1))
            }
            _ => None,
        };
    }
    match value_from_parameters(parameters, index + 1)? {
        5 => Some((
            indexed(u8::try_from(value_from_parameters(parameters, index + 2)?).ok()?),
            3,
        )),
        2 => Some((rgb_parameters(parameters, index + 2)?, 5)),
        _ => None,
    }
}

fn value_from_parameters(parameters: &[CsiParameter], index: usize) -> Option<usize> {
    parameters
        .get(index)?
        .subparameters
        .first()
        .copied()
        .flatten()
}

fn rgb_parameters(parameters: &[CsiParameter], index: usize) -> Option<Color> {
    let components = [
        Some(value_from_parameters(parameters, index)?),
        Some(value_from_parameters(parameters, index + 1)?),
        Some(value_from_parameters(parameters, index + 2)?),
    ];
    rgb(&components)
}

fn rgb(components: &[Option<usize>]) -> Option<Color> {
    Some(Color::Rgb(Rgb {
        red: u8::try_from(components.first().copied().flatten()?).ok()?,
        green: u8::try_from(components.get(1).copied().flatten()?).ok()?,
        blue: u8::try_from(components.get(2).copied().flatten()?).ok()?,
    }))
}
