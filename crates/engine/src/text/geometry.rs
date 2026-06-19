use super::{
    TextFamilyId, TextFamilySource, TextRenderIntent, TextStyleId, resolve_family_and_style,
};
use crate::board::BoardText;
use crate::ir::geometry::Point;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextHAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextVAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextAttributes {
    pub position: Point,
    pub rotation_degrees: i32,
    pub height_nm: i64,
    pub stroke_width_nm: i64,
    pub h_align: TextHAlign,
    pub v_align: TextVAlign,
    pub mirrored: bool,
    pub keep_upright: bool,
    pub line_spacing_ratio_ppm: i32,
    pub render_intent: TextRenderIntent,
    pub family: TextFamilyId,
    pub family_source: TextFamilySource,
    pub style: TextStyleId,
    pub italic: bool,
    pub bold: bool,
    pub style_class: Option<String>,
}

impl TextAttributes {
    pub fn from_board_text(text: &BoardText) -> Self {
        let (family, style) = resolve_family_and_style(
            text.render_intent,
            text.family_source,
            &text.family,
            &text.style,
        );
        Self {
            position: text.position,
            rotation_degrees: text.rotation,
            height_nm: text.height_nm,
            stroke_width_nm: if text.stroke_width_nm > 0 {
                text.stroke_width_nm
            } else {
                default_stroke_width_nm(text.height_nm)
            },
            h_align: text.h_align,
            v_align: text.v_align,
            mirrored: text.mirrored,
            keep_upright: text.keep_upright,
            line_spacing_ratio_ppm: text.line_spacing_ratio_ppm,
            render_intent: text.render_intent,
            family,
            family_source: text.family_source,
            style,
            italic: text.italic,
            bold: text.bold,
            style_class: text.style_class.clone(),
        }
    }
}

pub fn default_stroke_width_nm(height_nm: i64) -> i64 {
    ((height_nm as i128 * 152_000_i128) / 1_000_000_i128) as i64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextStroke {
    pub from: Point,
    pub to: Point,
    pub width_nm: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFillRule {
    NonZero,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextContourRing {
    pub points: Vec<Point>,
    pub signed_area_nm2: i128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextContourSet {
    pub fill_rule: TextFillRule,
    pub rings: Vec<TextContourRing>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFilledRegion {
    pub outer: Vec<Point>,
    pub holes: Vec<Vec<Point>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextResolvedFill {
    pub regions: Vec<TextFilledRegion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPolygon {
    pub outer: Vec<Point>,
    pub holes: Vec<Vec<Point>>,
}

impl From<TextFilledRegion> for TextPolygon {
    fn from(region: TextFilledRegion) -> Self {
        Self {
            outer: region.outer,
            holes: region.holes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextGeometryPrimitive {
    Stroke(TextStroke),
    FilledPolygon(TextPolygon),
}

impl TextGeometryPrimitive {
    pub fn stroke_fallback(self, width_nm: i64) -> Vec<TextStroke> {
        match self {
            Self::Stroke(stroke) => vec![stroke],
            Self::FilledPolygon(polygon) => {
                let mut strokes = polygon_ring_to_strokes(&polygon.outer, width_nm);
                for hole in polygon.holes {
                    strokes.extend(polygon_ring_to_strokes(&hole, width_nm));
                }
                strokes
            }
        }
    }
}

impl TextResolvedFill {
    pub fn into_primitives(self) -> Vec<TextGeometryPrimitive> {
        self.regions
            .into_iter()
            .map(|region| TextGeometryPrimitive::FilledPolygon(region.into()))
            .collect()
    }
}

fn polygon_ring_to_strokes(ring: &[Point], width_nm: i64) -> Vec<TextStroke> {
    if ring.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for index in 0..ring.len() {
        let from = ring[index];
        let to = ring[(index + 1) % ring.len()];
        if from == to {
            continue;
        }
        out.push(TextStroke { from, to, width_nm });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BoardText;
    use crate::ir::geometry::Point;
    use crate::text::{
        FAMILY_INTER, FAMILY_NEWSTROKE, TextFamilySource, TextHAlign, TextRenderIntent,
        TextStyleId, TextVAlign,
    };
    use uuid::Uuid;

    fn sample_board_text(render_intent: TextRenderIntent, family: TextFamilyId) -> BoardText {
        BoardText {
            uuid: Uuid::nil(),
            text: "TEST".to_string(),
            position: Point::zero(),
            rotation: 0,
            layer: 37,
            render_intent,
            family,
            family_source: TextFamilySource::ImplicitDefault,
            style: TextStyleId::default(),
            height_nm: 1_000_000,
            stroke_width_nm: 152_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            italic: false,
            bold: false,
            style_class: None,
        }
    }

    #[test]
    fn from_board_text_promotes_annotation_legacy_defaults_to_inter() {
        let attrs = TextAttributes::from_board_text(&sample_board_text(
            TextRenderIntent::Annotation,
            TextFamilyId(FAMILY_NEWSTROKE.to_string()),
        ));
        assert_eq!(attrs.family.0, FAMILY_INTER);
        assert_eq!(attrs.family_source, TextFamilySource::ImplicitDefault);
    }

    #[test]
    fn from_board_text_promotes_manufacturing_legacy_defaults_to_inter() {
        let attrs = TextAttributes::from_board_text(&sample_board_text(
            TextRenderIntent::Manufacturing,
            TextFamilyId(FAMILY_NEWSTROKE.to_string()),
        ));
        assert_eq!(attrs.family.0, FAMILY_INTER);
    }

    #[test]
    fn from_board_text_preserves_explicit_newstroke_family() {
        let mut text = sample_board_text(
            TextRenderIntent::Manufacturing,
            TextFamilyId(FAMILY_NEWSTROKE.to_string()),
        );
        text.family_source = TextFamilySource::Explicit;
        let attrs = TextAttributes::from_board_text(&text);

        assert_eq!(attrs.family.0, FAMILY_NEWSTROKE);
        assert_eq!(attrs.family_source, TextFamilySource::Explicit);
    }
}
