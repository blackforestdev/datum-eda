use datum_gui_protocol::TerminalTextStyle;

pub(super) fn apply_sgr(raw_params: &[u8], current_style: &mut TerminalTextStyle) {
    let params = sgr_params(raw_params);
    let mut index = 0;
    while index < params.len() {
        match params[index] {
            0 => *current_style = TerminalTextStyle::default(),
            1 => current_style.bold = true,
            2 => current_style.dim = true,
            3 => current_style.italic = true,
            4 => current_style.underline = true,
            5 | 6 => current_style.blink = true,
            7 => current_style.inverse = true,
            8 => current_style.conceal = true,
            9 => current_style.strikethrough = true,
            22 => {
                current_style.bold = false;
                current_style.dim = false;
            }
            23 => current_style.italic = false,
            24 => current_style.underline = false,
            25 => current_style.blink = false,
            27 => current_style.inverse = false,
            28 => current_style.conceal = false,
            29 => current_style.strikethrough = false,
            30 => current_style.fg = Some("black".to_string()),
            31 => current_style.fg = Some("red".to_string()),
            32 => current_style.fg = Some("green".to_string()),
            33 => current_style.fg = Some("yellow".to_string()),
            34 => current_style.fg = Some("blue".to_string()),
            35 => current_style.fg = Some("magenta".to_string()),
            36 => current_style.fg = Some("cyan".to_string()),
            37 => current_style.fg = Some("white".to_string()),
            39 => current_style.fg = None,
            38 => {
                if let Some((color, consumed)) = extended_sgr_color(&params[index + 1..]) {
                    current_style.fg = Some(color);
                    index += consumed;
                } else if let Some(consumed) =
                    malformed_extended_sgr_color_params(&params[index + 1..])
                {
                    index += consumed;
                }
            }
            40 => current_style.bg = Some("black".to_string()),
            41 => current_style.bg = Some("red".to_string()),
            42 => current_style.bg = Some("green".to_string()),
            43 => current_style.bg = Some("yellow".to_string()),
            44 => current_style.bg = Some("blue".to_string()),
            45 => current_style.bg = Some("magenta".to_string()),
            46 => current_style.bg = Some("cyan".to_string()),
            47 => current_style.bg = Some("white".to_string()),
            49 => current_style.bg = None,
            48 => {
                if let Some((color, consumed)) = extended_sgr_color(&params[index + 1..]) {
                    current_style.bg = Some(color);
                    index += consumed;
                } else if let Some(consumed) =
                    malformed_extended_sgr_color_params(&params[index + 1..])
                {
                    index += consumed;
                }
            }
            90 => current_style.fg = Some("bright_black".to_string()),
            91 => current_style.fg = Some("bright_red".to_string()),
            92 => current_style.fg = Some("bright_green".to_string()),
            93 => current_style.fg = Some("bright_yellow".to_string()),
            94 => current_style.fg = Some("bright_blue".to_string()),
            95 => current_style.fg = Some("bright_magenta".to_string()),
            96 => current_style.fg = Some("bright_cyan".to_string()),
            97 => current_style.fg = Some("bright_white".to_string()),
            100 => current_style.bg = Some("bright_black".to_string()),
            101 => current_style.bg = Some("bright_red".to_string()),
            102 => current_style.bg = Some("bright_green".to_string()),
            103 => current_style.bg = Some("bright_yellow".to_string()),
            104 => current_style.bg = Some("bright_blue".to_string()),
            105 => current_style.bg = Some("bright_magenta".to_string()),
            106 => current_style.bg = Some("bright_cyan".to_string()),
            107 => current_style.bg = Some("bright_white".to_string()),
            53 => current_style.overline = true,
            55 => current_style.overline = false,
            _ => {}
        }
        index += 1;
    }
}

fn sgr_params(raw_params: &[u8]) -> Vec<usize> {
    let Ok(params) = std::str::from_utf8(raw_params) else {
        return vec![0];
    };
    if params.is_empty() {
        return vec![0];
    }
    params
        .split(';')
        .map(|param| {
            if param.is_empty() {
                0
            } else {
                param.parse::<usize>().unwrap_or(0)
            }
        })
        .collect()
}

pub(super) fn extended_sgr_color(params: &[usize]) -> Option<(String, usize)> {
    match params {
        [5, index, ..] if *index <= 255 => Some((format!("ansi256:{index}"), 2)),
        [2, red, green, blue, ..] if *red <= 255 && *green <= 255 && *blue <= 255 => {
            Some((format!("rgb:{red}:{green}:{blue}"), 4))
        }
        _ => None,
    }
}

pub(super) fn malformed_extended_sgr_color_params(params: &[usize]) -> Option<usize> {
    match params {
        [5, ..] if params.len() >= 2 => Some(2),
        [2, ..] if params.len() >= 4 => Some(4),
        _ => None,
    }
}
