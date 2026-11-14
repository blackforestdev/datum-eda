//! Full-text Proposal card measurement shared by presentation and acceptance.
pub struct ProposalReviewLayout {
    pub width: f32,
    pub height: f32,
    pub lines: Vec<String>,
    pub row_heights: Vec<f32>,
    pub control_widths: Option<(f32, f32)>,
    pub fits: bool,
}

pub fn measure(
    rows: &[String],
    accept_hint: &str,
    viewport_width: f32,
    viewport_height: f32,
    bottom_clearance: f32,
) -> ProposalReviewLayout {
    let width = (viewport_width - 28.).clamp(80., 520.);
    let columns = ((width - 20.) / 6.7).max(1.) as usize;
    let lines: Vec<String> = rows
        .iter()
        .flat_map(|row| physical_rows(row))
        .flat_map(|row| {
            let chars: Vec<char> = row.chars().collect();
            if chars.is_empty() {
                return vec![String::new()];
            }
            chars
                .chunks(columns)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect()
        })
        .collect();
    let height = 56. + lines.len() as f32 * 17.;
    ProposalReviewLayout {
        width,
        height,
        row_heights: vec![17.; lines.len()],
        control_widths: None,
        lines,
        fits: width <= viewport_width - 28.
            && height <= viewport_height - 28. - bottom_clearance
            && columns >= accept_hint.chars().count(),
    }
}

/// Complete rows measured by the caller's existing native font authority.
/// Each row is painted at this exact width and advances by its measured height.
pub fn measure_rows(
    rows: &[String],
    label_widths: (f32, f32),
    viewport_width: f32,
    viewport_height: f32,
    bottom_clearance: f32,
    mut measure_height: impl FnMut(&str, f32) -> Result<f32, String>,
) -> Result<ProposalReviewLayout, String> {
    let width = (viewport_width - 28.).clamp(80., 520.);
    let content_width = (width - 20.).floor().max(1.);
    let row_heights = rows
        .iter()
        .map(|row| {
            let height = measure_height(row, content_width)?;
            if !height.is_finite() || height <= 0. {
                return Err("Text measurement unavailable".into());
            }
            Ok(height.ceil().max(14.) + 3.)
        })
        .collect::<Result<Vec<f32>, String>>()?;
    let height = 56. + row_heights.iter().sum::<f32>();
    let controls = (label_widths.0.ceil() + 6., label_widths.1.ceil() + 6.);
    Ok(ProposalReviewLayout {
        width,
        height,
        lines: rows.to_vec(),
        row_heights,
        control_widths: Some(controls),
        fits: width <= viewport_width - 28.
            && height <= viewport_height - 28. - bottom_clearance
            && controls.0.is_finite()
            && controls.1.is_finite()
            && controls.0 + 8. + controls.1 <= content_width,
    })
}

// Hard breaks become separate text runs, including empty/trailing paragraphs.
// Paired CR/LF sequences occupy one break, matching the native text painter.
fn physical_rows(text: &str) -> Vec<String> {
    let mut rows = vec![String::new()];
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' || ch == '\n' {
            let paired = if ch == '\r' { '\n' } else { '\r' };
            if chars.peek() == Some(&paired) {
                chars.next();
            }
            rows.push(String::new());
        } else {
            rows.last_mut().unwrap().push(ch);
        }
    }
    rows
}

/// Existing focused-pane console geometry, in caller pixel coordinates.
/// Shared with the console painter so review fit never guesses its occlusion.
pub struct ConsoleGeometry {
    pub bottom: f32,
    pub row_height: f32,
    pub pad_y: f32,
    pub strip_height: f32,
    pub history_gap: f32,
    pub history_height: f32,
    pub occupied_height: f32,
}
pub fn console_geometry(
    width: f32,
    height: f32,
    scale: f32,
    record: bool,
    history: bool,
) -> Option<ConsoleGeometry> {
    let bottom = 10. * scale;
    let pad_y = 5. * scale;
    let row_height = 16. * scale;
    let strip_height = row_height + pad_y * 2.;
    if (!record && !history) || width < 96. * scale || height < strip_height + bottom {
        return None;
    }
    let strip_height = if record { strip_height } else { 0. };
    let history_gap = if history { 6. * scale } else { 0. };
    let history_height = if history {
        (170. * scale).min((height - bottom - strip_height - history_gap).max(1.))
    } else {
        0.
    };
    Some(ConsoleGeometry {
        bottom,
        pad_y,
        row_height,
        strip_height,
        history_gap,
        history_height,
        occupied_height: bottom + strip_height + history_gap + history_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hard_breaks_empty_and_trailing_paragraphs_each_reserve_painter_rows() {
        for (input, expected) in [
            ("a\nb\nc", vec!["a", "b", "c"]),
            ("a\r\nb\n\rc\r", vec!["a", "b", "c", ""]),
            ("a\n\nb", vec!["a", "", "b"]),
        ] {
            let layout = measure(&[input.into()], "Tab", 800., 800., 0.);
            assert_eq!(layout.lines, expected);
            assert_eq!(layout.height, 56. + expected.len() as f32 * 17.);
            assert!(layout.lines.iter().all(|line| !line.contains(['\r', '\n'])));
        }
    }
    #[test]
    fn console_occlusion_tracks_fade_history_and_scale_without_phantom_strip() {
        for scale in [1., 1.25, 2.] {
            let geometry = |record, history| {
                console_geometry(800. * scale, 600. * scale, scale, record, history)
            };
            assert!(geometry(false, false).is_none());
            let strip = geometry(true, false).unwrap();
            assert_eq!(strip.occupied_height, 36. * scale);
            let expanded = geometry(true, true).unwrap();
            assert_eq!(expanded.occupied_height, 212. * scale);
            let faded = geometry(false, true).unwrap();
            assert_eq!(faded.strip_height, 0.);
            assert_eq!(faded.occupied_height, 186. * scale);
            assert!(console_geometry(95. * scale, 600. * scale, scale, true, true).is_none());
            assert!(console_geometry(800. * scale, 35. * scale, scale, true, true).is_none());
        }
    }
}
