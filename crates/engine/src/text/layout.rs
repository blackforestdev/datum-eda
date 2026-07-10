use crate::board::BoardText;
use crate::export::ExportError;
use crate::ir::geometry::Point;

use super::backend::default_backend_for_attributes;
use super::geometry::{TextAttributes, TextGeometryPrimitive, TextStroke};

pub fn layout_text_strokes(
    text: &str,
    attrs: &TextAttributes,
) -> Result<Vec<TextStroke>, ExportError> {
    default_backend_for_attributes(attrs).shape_text(text, attrs)
}

pub fn layout_text_geometry(
    text: &str,
    attrs: &TextAttributes,
) -> Result<Vec<TextGeometryPrimitive>, ExportError> {
    default_backend_for_attributes(attrs).shape_text_geometry(text, attrs)
}

pub(crate) fn layout_text_strokes_with_glyph_lookup<F>(
    text: &str,
    attrs: &TextAttributes,
    glyph_lookup: F,
) -> Result<Vec<TextStroke>, ExportError>
where
    F: Fn(char) -> Option<&'static str>,
{
    if attrs.height_nm <= 0 {
        return Err(ExportError::InvalidTextHeight);
    }
    if attrs.stroke_width_nm <= 0 {
        return Err(ExportError::InvalidTextStrokeWidth);
    }

    let attrs = normalize_text_attributes(attrs.clone());
    let decoded_lines = text
        .split('\n')
        .map(|line| decode_line(line, &glyph_lookup))
        .collect::<Result<Vec<_>, ExportError>>()?;
    let scale_nm = attrs.height_nm as f64 / GLYPH_HEIGHT_UNITS as f64;
    let line_pitch_nm = (attrs.height_nm as i128 * attrs.line_spacing_ratio_ppm as i128
        / 1_000_000_i128) as i64
        + attrs.stroke_width_nm;

    let mut block_min_x_nm = 0_i64;
    let mut block_max_x_nm = 0_i64;
    let mut block_min_y_nm = 0_i64;
    let mut block_max_y_nm = 0_i64;

    for (line_index, line) in decoded_lines.iter().enumerate() {
        let baseline_y_nm = line_baseline_y_nm(line_index, line_pitch_nm, attrs.mirrored);
        let line_min_x_nm = scale_units_to_nm(line.min_x_units, scale_nm);
        let line_max_x_nm = scale_units_to_nm(line.max_x_units, scale_nm);
        let line_min_y_nm = baseline_y_nm + scale_units_to_nm(line.min_y_units, scale_nm);
        let line_max_y_nm = baseline_y_nm + scale_units_to_nm(line.max_y_units, scale_nm);

        if line_index == 0 {
            block_min_x_nm = line_min_x_nm;
            block_max_x_nm = line_max_x_nm;
            block_min_y_nm = line_min_y_nm;
            block_max_y_nm = line_max_y_nm;
        } else {
            block_min_x_nm = block_min_x_nm.min(line_min_x_nm);
            block_max_x_nm = block_max_x_nm.max(line_max_x_nm);
            block_min_y_nm = block_min_y_nm.min(line_min_y_nm);
            block_max_y_nm = block_max_y_nm.max(line_max_y_nm);
        }
    }

    let anchor_shift_x_nm = anchor_shift(attrs.h_align, block_min_x_nm, block_max_x_nm);
    let anchor_shift_y_nm = anchor_shift(attrs.v_align, block_min_y_nm, block_max_y_nm);
    let mut strokes = Vec::new();

    for (line_index, line) in decoded_lines.iter().enumerate() {
        let baseline_y_nm = line_baseline_y_nm(line_index, line_pitch_nm, attrs.mirrored);
        for segment in &line.segments {
            let from = transform_text_point(
                &attrs,
                scale_units_to_nm(segment.from.0, scale_nm) + anchor_shift_x_nm,
                baseline_y_nm + scale_units_to_nm(segment.from.1, scale_nm) + anchor_shift_y_nm,
            );
            let to = transform_text_point(
                &attrs,
                scale_units_to_nm(segment.to.0, scale_nm) + anchor_shift_x_nm,
                baseline_y_nm + scale_units_to_nm(segment.to.1, scale_nm) + anchor_shift_y_nm,
            );
            strokes.push(TextStroke {
                from,
                to,
                width_nm: attrs.stroke_width_nm,
            });
        }
    }

    Ok(strokes)
}

pub fn layout_text_strokes_from_board_text(
    text: &BoardText,
) -> Result<Vec<TextStroke>, ExportError> {
    layout_text_strokes(&text.text, &TextAttributes::from_board_text(text))
}

pub fn layout_text_geometry_from_board_text(
    text: &BoardText,
) -> Result<Vec<TextGeometryPrimitive>, ExportError> {
    layout_text_geometry(&text.text, &TextAttributes::from_board_text(text))
}

const GLYPH_HEIGHT_UNITS: i32 = 21;
const HERSHEY_ORIGIN_Y_UNITS: i32 = 9;
const INTER_CHAR_UNITS: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GlyphSegmentUnits {
    from: (i32, i32),
    to: (i32, i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecodedLine {
    segments: Vec<GlyphSegmentUnits>,
    min_x_units: i32,
    max_x_units: i32,
    min_y_units: i32,
    max_y_units: i32,
}

fn decode_line<F>(line: &str, glyph_lookup: &F) -> Result<DecodedLine, ExportError>
where
    F: Fn(char) -> Option<&'static str>,
{
    let mut segments = Vec::new();
    let mut cursor_x_units = 0_i32;
    let mut min_x_units = 0_i32;
    let mut max_x_units = 0_i32;
    let mut min_y_units = 0_i32;
    let mut max_y_units = 0_i32;
    let mut has_bounds = false;
    let mut overbar_start_units: Option<i32> = None;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '~' {
            if chars.peek() == Some(&'~') {
                chars.next();
            } else {
                if let Some(start_units) = overbar_start_units.take() {
                    push_segment_with_bounds(
                        &mut segments,
                        &mut min_x_units,
                        &mut max_x_units,
                        &mut min_y_units,
                        &mut max_y_units,
                        &mut has_bounds,
                        (start_units, OVERBAR_Y_UNITS),
                        (cursor_x_units, OVERBAR_Y_UNITS),
                    );
                } else {
                    overbar_start_units = Some(cursor_x_units);
                }
                continue;
            }
        }

        let glyph = glyph_lookup(ch).ok_or(ExportError::UnsupportedSilkscreenTextCharacter(ch))?;
        let bytes = glyph.as_bytes();
        if bytes.len() < 2 {
            continue;
        }

        let left = hershey_coord(bytes[0]);
        let right = hershey_coord(bytes[1]);
        let x_shift_units = -left;
        let mut pen_down = false;
        let mut previous = (0_i32, 0_i32);
        let mut index = 2;

        while index + 1 < bytes.len() {
            if bytes[index] == b' ' {
                pen_down = false;
                index += 1;
                continue;
            }

            let x = hershey_coord(bytes[index]) + x_shift_units + cursor_x_units;
            let y = hershey_coord(bytes[index + 1]) + HERSHEY_ORIGIN_Y_UNITS;

            if pen_down {
                push_segment_with_bounds(
                    &mut segments,
                    &mut min_x_units,
                    &mut max_x_units,
                    &mut min_y_units,
                    &mut max_y_units,
                    &mut has_bounds,
                    previous,
                    (x, y),
                );
            } else {
                include_point_bounds(
                    &mut min_x_units,
                    &mut max_x_units,
                    &mut min_y_units,
                    &mut max_y_units,
                    &mut has_bounds,
                    x,
                    y,
                );
            }
            previous = (x, y);
            pen_down = true;
            index += 2;
        }

        cursor_x_units += right - left;
        if has_remaining_visible_glyph(chars.clone()) {
            cursor_x_units += INTER_CHAR_UNITS;
        }
    }

    if let Some(start_units) = overbar_start_units.take() {
        push_segment_with_bounds(
            &mut segments,
            &mut min_x_units,
            &mut max_x_units,
            &mut min_y_units,
            &mut max_y_units,
            &mut has_bounds,
            (start_units, OVERBAR_Y_UNITS),
            (cursor_x_units, OVERBAR_Y_UNITS),
        );
    }

    if !has_bounds {
        max_x_units = cursor_x_units;
    }

    Ok(DecodedLine {
        segments,
        min_x_units,
        max_x_units,
        min_y_units,
        max_y_units,
    })
}

const OVERBAR_Y_UNITS: i32 = -6;

fn include_point_bounds(
    min_x_units: &mut i32,
    max_x_units: &mut i32,
    min_y_units: &mut i32,
    max_y_units: &mut i32,
    has_bounds: &mut bool,
    x: i32,
    y: i32,
) {
    if !*has_bounds {
        *min_x_units = x;
        *max_x_units = x;
        *min_y_units = y;
        *max_y_units = y;
        *has_bounds = true;
    } else {
        *min_x_units = (*min_x_units).min(x);
        *max_x_units = (*max_x_units).max(x);
        *min_y_units = (*min_y_units).min(y);
        *max_y_units = (*max_y_units).max(y);
    }
}

// Text layout threads many glyph/run/metrics parameters.
#[allow(clippy::too_many_arguments)]
fn push_segment_with_bounds(
    segments: &mut Vec<GlyphSegmentUnits>,
    min_x_units: &mut i32,
    max_x_units: &mut i32,
    min_y_units: &mut i32,
    max_y_units: &mut i32,
    has_bounds: &mut bool,
    from: (i32, i32),
    to: (i32, i32),
) {
    include_point_bounds(
        min_x_units,
        max_x_units,
        min_y_units,
        max_y_units,
        has_bounds,
        from.0,
        from.1,
    );
    include_point_bounds(
        min_x_units,
        max_x_units,
        min_y_units,
        max_y_units,
        has_bounds,
        to.0,
        to.1,
    );
    segments.push(GlyphSegmentUnits { from, to });
}

fn hershey_coord(byte: u8) -> i32 {
    byte as i32 - 'R' as i32
}

fn scale_units_to_nm(units: i32, scale_nm: f64) -> i64 {
    ((units as f64) * scale_nm).round() as i64
}

fn has_remaining_visible_glyph(mut chars: std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    while let Some(ch) = chars.next() {
        if ch == '~' {
            if chars.peek() == Some(&'~') {
                chars.next();
                return true;
            }
            continue;
        }
        return true;
    }
    false
}

pub(crate) fn anchor_shift(align: impl Into<AnchorAlign>, min_nm: i64, max_nm: i64) -> i64 {
    match align.into() {
        AnchorAlign::Start => -min_nm,
        AnchorAlign::Center => -((min_nm + max_nm) / 2),
        AnchorAlign::End => -max_nm,
    }
}

pub(crate) fn normalize_text_attributes(mut attrs: TextAttributes) -> TextAttributes {
    if attrs.mirrored {
        attrs.h_align = flip_h_align(attrs.h_align);
    }
    if attrs.keep_upright && should_flip_upright(attrs.rotation_degrees) {
        attrs.rotation_degrees += 180;
        attrs.h_align = flip_h_align(attrs.h_align);
        attrs.v_align = flip_v_align(attrs.v_align);
    }
    attrs
}

fn should_flip_upright(rotation_degrees: i32) -> bool {
    let wrapped = rotation_degrees.rem_euclid(360);
    wrapped > 90 && wrapped < 270
}

pub(crate) fn line_baseline_y_nm(line_index: usize, line_pitch_nm: i64, mirrored: bool) -> i64 {
    let direction = if mirrored { 1 } else { -1 };
    direction * (line_index as i64) * line_pitch_nm
}

fn flip_h_align(align: crate::text::TextHAlign) -> crate::text::TextHAlign {
    match align {
        crate::text::TextHAlign::Left => crate::text::TextHAlign::Right,
        crate::text::TextHAlign::Center => crate::text::TextHAlign::Center,
        crate::text::TextHAlign::Right => crate::text::TextHAlign::Left,
    }
}

fn flip_v_align(align: crate::text::TextVAlign) -> crate::text::TextVAlign {
    match align {
        crate::text::TextVAlign::Top => crate::text::TextVAlign::Bottom,
        crate::text::TextVAlign::Center => crate::text::TextVAlign::Center,
        crate::text::TextVAlign::Bottom => crate::text::TextVAlign::Top,
    }
}

pub(crate) enum AnchorAlign {
    Start,
    Center,
    End,
}

impl From<crate::text::TextHAlign> for AnchorAlign {
    fn from(value: crate::text::TextHAlign) -> Self {
        match value {
            crate::text::TextHAlign::Left => Self::Start,
            crate::text::TextHAlign::Center => Self::Center,
            crate::text::TextHAlign::Right => Self::End,
        }
    }
}

impl From<crate::text::TextVAlign> for AnchorAlign {
    fn from(value: crate::text::TextVAlign) -> Self {
        match value {
            crate::text::TextVAlign::Top => Self::Start,
            crate::text::TextVAlign::Center => Self::Center,
            crate::text::TextVAlign::Bottom => Self::End,
        }
    }
}

pub(crate) fn transform_text_point(attrs: &TextAttributes, mut x_nm: i64, y_nm: i64) -> Point {
    if attrs.mirrored {
        x_nm = -x_nm;
    }
    rotate_text_point(attrs.position, attrs.rotation_degrees, x_nm, y_nm)
}

fn rotate_text_point(origin: Point, rotation_deg: i32, x_nm: i64, y_nm: i64) -> Point {
    let radians = f64::from(rotation_deg).to_radians();
    let x = x_nm as f64;
    let y = y_nm as f64;
    let rotated_x = x * radians.cos() - y * radians.sin();
    let rotated_y = x * radians.sin() + y * radians.cos();
    Point {
        x: origin.x + rotated_x.round() as i64,
        y: origin.y + rotated_y.round() as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::geometry::Point;
    use crate::text::{TextHAlign, TextVAlign};

    #[test]
    fn layout_rejects_non_positive_height() {
        let attrs = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 0,
            stroke_width_nm: 100_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let err = layout_text_strokes("A", &attrs).expect_err("height must be positive");
        assert!(matches!(err, ExportError::InvalidTextHeight));
    }

    #[test]
    fn layout_supports_lowercase_glyphs() {
        let attrs = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 1_000_000,
            stroke_width_nm: 100_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let lower = layout_text_strokes("a", &attrs).expect("lowercase should layout");
        assert!(!lower.is_empty());
    }

    #[test]
    fn layout_includes_inter_character_spacing() {
        let attrs = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 1_000_000,
            stroke_width_nm: 100_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let single = layout_text_strokes("A", &attrs).expect("single glyph should layout");
        let pair = layout_text_strokes("AA", &attrs).expect("pair should layout");
        let single_max_x = single
            .iter()
            .flat_map(|stroke| [stroke.from.x, stroke.to.x])
            .max()
            .expect("single glyph should produce geometry");
        let pair_max_x = pair
            .iter()
            .flat_map(|stroke| [stroke.from.x, stroke.to.x])
            .max()
            .expect("pair should produce geometry");
        assert!(pair_max_x > single_max_x * 2);
    }

    #[test]
    fn layout_supports_common_eda_symbols() {
        let attrs = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 1_000_000,
            stroke_width_nm: 100_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let strokes =
            layout_text_strokes("10µ 25°C 10kΩ ±1% 2×", &attrs).expect("EDA symbols should layout");
        assert!(!strokes.is_empty());
    }

    #[test]
    fn layout_supports_overbar_markup_and_tilde_escape() {
        let attrs = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 1_000_000,
            stroke_width_nm: 100_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let overbar = layout_text_strokes("~RST~", &attrs).expect("overbar markup should layout");
        let escaped = layout_text_strokes("~~", &attrs).expect("escaped tilde should layout");
        assert!(!overbar.is_empty());
        assert!(!escaped.is_empty());
    }

    #[test]
    fn layout_closes_unmatched_overbar_at_line_end() {
        let attrs = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 1_000_000,
            stroke_width_nm: 100_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let strokes = layout_text_strokes("~RST", &attrs).expect("trailing overbar should layout");
        let overbar_y = strokes
            .iter()
            .flat_map(|stroke| [stroke.from.y, stroke.to.y])
            .min()
            .expect("layout should produce geometry");
        assert!(overbar_y < 0);
    }

    #[test]
    fn layout_centers_multiline_text_block() {
        let attrs = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 2_100_000,
            stroke_width_nm: 100_000,
            h_align: TextHAlign::Center,
            v_align: TextVAlign::Center,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_350_000,
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let strokes = layout_text_strokes("A\nA", &attrs).expect("multiline text should layout");
        let min_x = strokes
            .iter()
            .flat_map(|stroke| [stroke.from.x, stroke.to.x])
            .min()
            .expect("layout should produce geometry");
        let max_x = strokes
            .iter()
            .flat_map(|stroke| [stroke.from.x, stroke.to.x])
            .max()
            .expect("layout should produce geometry");
        let min_y = strokes
            .iter()
            .flat_map(|stroke| [stroke.from.y, stroke.to.y])
            .min()
            .expect("layout should produce geometry");
        let max_y = strokes
            .iter()
            .flat_map(|stroke| [stroke.from.y, stroke.to.y])
            .max()
            .expect("layout should produce geometry");

        assert!((min_x + max_x).abs() <= 2);
        assert!((min_y + max_y).abs() <= 2);
    }

    #[test]
    fn layout_multiline_spacing_includes_stroke_width() {
        let narrow = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 1_000_000,
            stroke_width_nm: 50_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_350_000,
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let mut wide = narrow.clone();
        wide.stroke_width_nm = 250_000;

        let narrow_strokes =
            layout_text_strokes("A\nA", &narrow).expect("narrow multiline text should layout");
        let wide_strokes =
            layout_text_strokes("A\nA", &wide).expect("wide multiline text should layout");

        let narrow_min_y = narrow_strokes
            .iter()
            .flat_map(|stroke| [stroke.from.y, stroke.to.y])
            .min()
            .expect("layout should produce geometry");
        let wide_min_y = wide_strokes
            .iter()
            .flat_map(|stroke| [stroke.from.y, stroke.to.y])
            .min()
            .expect("layout should produce geometry");

        assert!(wide_min_y < narrow_min_y);
    }

    #[test]
    fn layout_rotation_zero_is_upright_in_y_down_board_space() {
        let attrs = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 1_000_000,
            stroke_width_nm: 100_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let strokes = layout_text_strokes("T", &attrs).expect("glyph should layout");
        let top_bar_y = strokes
            .iter()
            .filter(|stroke| stroke.from.y == stroke.to.y)
            .map(|stroke| stroke.from.y)
            .min()
            .expect("T should contain a top bar");
        let stem_bottom_y = strokes
            .iter()
            .filter(|stroke| stroke.from.x == stroke.to.x)
            .flat_map(|stroke| [stroke.from.y, stroke.to.y])
            .max()
            .expect("T should contain a vertical stem");
        assert!(top_bar_y < stem_bottom_y);
    }

    #[test]
    fn keep_upright_flips_rotation_and_alignment() {
        let attrs = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 180,
            height_nm: 1_000_000,
            stroke_width_nm: 100_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Top,
            mirrored: false,
            keep_upright: true,
            line_spacing_ratio_ppm: 1_000_000,
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let normalized = normalize_text_attributes(attrs);
        assert_eq!(normalized.rotation_degrees, 360);
        assert_eq!(normalized.h_align, TextHAlign::Right);
        assert_eq!(normalized.v_align, TextVAlign::Bottom);
    }

    #[test]
    fn mirrored_text_swaps_horizontal_alignment() {
        let attrs = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 1_000_000,
            stroke_width_nm: 100_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: true,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            render_intent: crate::text::TextRenderIntent::Manufacturing,
            family: crate::text::TextFamilyId::default(),
            family_source: crate::text::TextFamilySource::ImplicitDefault,
            style: crate::text::TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let normalized = normalize_text_attributes(attrs);
        assert_eq!(normalized.h_align, TextHAlign::Right);
    }

    #[test]
    fn mirrored_multiline_text_inverts_line_stacking_direction() {
        let pitch = 1_450_000;
        assert_eq!(line_baseline_y_nm(0, pitch, false), 0);
        assert_eq!(line_baseline_y_nm(1, pitch, false), -pitch);
        assert_eq!(line_baseline_y_nm(0, pitch, true), 0);
        assert_eq!(line_baseline_y_nm(1, pitch, true), pitch);
    }
}
