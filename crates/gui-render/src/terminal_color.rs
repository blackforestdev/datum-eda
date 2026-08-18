//! Terminal color projection for the provisional screen renderer.
//!
//! The parser stores governed color descriptors rather than renderer values.
//! This module is the single conversion seam for ANSI-16, ANSI-256, and RGB
//! foreground/background colors until TerminalCore owns the complete palette.

use crate::{PANEL_BG, TEXT_PANEL_VALUE, TEXT_PRIMARY};

type Rgb = [f32; 3];

pub(super) fn span_foreground(
    fg: Option<&str>,
    bg: Option<&str>,
    bold: bool,
    inverse: bool,
    conceal: bool,
) -> Rgb {
    let resolved = if inverse {
        bg.and_then(terminal_color).unwrap_or(PANEL_BG)
    } else {
        fg.and_then(terminal_color)
            .unwrap_or(if bold { TEXT_PRIMARY } else { TEXT_PANEL_VALUE })
    };
    if conceal {
        span_background(fg, bg, inverse).unwrap_or(PANEL_BG)
    } else {
        resolved
    }
}

pub(super) fn span_background(fg: Option<&str>, bg: Option<&str>, inverse: bool) -> Option<Rgb> {
    if inverse {
        Some(fg.and_then(terminal_color).unwrap_or(TEXT_PANEL_VALUE))
    } else {
        bg.and_then(terminal_color)
    }
}

pub(super) fn terminal_color(value: &str) -> Option<Rgb> {
    match value {
        "black" => Some(rgb8(64, 69, 77)),
        "red" => Some(rgb8(242, 82, 71)),
        "green" => Some(rgb8(115, 209, 122)),
        "yellow" => Some(rgb8(245, 199, 82)),
        "blue" => Some(rgb8(107, 158, 242)),
        "magenta" => Some(rgb8(209, 128, 230)),
        "cyan" => Some(rgb8(97, 209, 224)),
        "white" => Some(rgb8(230, 235, 240)),
        "bright_black" => Some(rgb8(122, 133, 148)),
        "bright_red" => Some(rgb8(255, 107, 92)),
        "bright_green" => Some(rgb8(148, 235, 143)),
        "bright_yellow" => Some(rgb8(255, 219, 107)),
        "bright_blue" => Some(rgb8(133, 184, 255)),
        "bright_magenta" => Some(rgb8(235, 158, 255)),
        "bright_cyan" => Some(rgb8(128, 235, 245)),
        "bright_white" => Some(rgb8(255, 255, 255)),
        _ => parse_rgb(value).or_else(|| parse_ansi256(value)),
    }
}

fn parse_rgb(value: &str) -> Option<Rgb> {
    let mut values = value.strip_prefix("rgb:")?.split(':');
    let red = values.next()?.parse::<u8>().ok()?;
    let green = values.next()?.parse::<u8>().ok()?;
    let blue = values.next()?.parse::<u8>().ok()?;
    values.next().is_none().then(|| rgb8(red, green, blue))
}

fn parse_ansi256(value: &str) -> Option<Rgb> {
    let index = value.strip_prefix("ansi256:")?.parse::<u8>().ok()?;
    match index {
        0..=15 => const {
            [
                "black",
                "red",
                "green",
                "yellow",
                "blue",
                "magenta",
                "cyan",
                "white",
                "bright_black",
                "bright_red",
                "bright_green",
                "bright_yellow",
                "bright_blue",
                "bright_magenta",
                "bright_cyan",
                "bright_white",
            ]
        }
        .get(index as usize)
        .and_then(|name| terminal_color(name)),
        16..=231 => {
            let cube = index - 16;
            let level = |component: u8| [0, 95, 135, 175, 215, 255][component as usize];
            Some(rgb8(
                level(cube / 36),
                level((cube % 36) / 6),
                level(cube % 6),
            ))
        }
        232..=255 => {
            let gray = 8 + (index - 232) * 10;
            Some(rgb8(gray, gray, gray))
        }
    }
}

const fn rgb8(red: u8, green: u8, blue: u8) -> Rgb {
    [
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi256_color_cube_and_grayscale_are_projected_exactly() {
        assert_eq!(terminal_color("ansi256:196"), Some(rgb8(255, 0, 0)));
        assert_eq!(terminal_color("ansi256:22"), Some(rgb8(0, 95, 0)));
        assert_eq!(terminal_color("ansi256:232"), Some(rgb8(8, 8, 8)));
        assert_eq!(terminal_color("ansi256:255"), Some(rgb8(238, 238, 238)));
    }

    #[test]
    fn truecolor_and_inverse_backgrounds_preserve_exact_channels() {
        assert_eq!(terminal_color("rgb:12:34:56"), Some(rgb8(12, 34, 56)));
        assert_eq!(terminal_color("rgb:999:0:0"), None);
        assert_eq!(
            span_foreground(Some("rgb:1:2:3"), Some("rgb:4:5:6"), false, true, false,),
            rgb8(4, 5, 6)
        );
        assert_eq!(
            span_background(Some("rgb:1:2:3"), None, true),
            Some(rgb8(1, 2, 3))
        );
    }
}
