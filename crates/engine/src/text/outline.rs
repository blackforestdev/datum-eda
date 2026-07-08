use ttf_parser::{Face, GlyphId, OutlineBuilder, Tag};

use crate::export::ExportError;
use crate::text::backend::{GlyphBackend, GlyphBackendKind};
use crate::text::geometry::{
    TextAttributes, TextContourRing, TextContourSet, TextFillRule, TextFilledRegion,
    TextGeometryPrimitive, TextResolvedFill,
};
use crate::text::layout::{
    anchor_shift, line_baseline_y_nm, normalize_text_attributes, transform_text_point,
};
use crate::text::registry::vendored_asset_path_for_family;
use crate::text::{FlattenedGlyphFixture, FlattenedOutlineContour, FlattenedOutlinePoint};

#[derive(Debug, Default)]
pub struct VendoredOutlineBackend;

static VENDORED_OUTLINE_BACKEND: VendoredOutlineBackend = VendoredOutlineBackend;

pub fn vendored_outline_backend() -> &'static VendoredOutlineBackend {
    &VENDORED_OUTLINE_BACKEND
}

#[derive(Debug, thiserror::Error)]
pub enum OutlineError {
    #[error("failed to parse outline font")]
    InvalidFont,
    #[error("glyph not found for codepoint U+{0:04X}")]
    MissingGlyph(u32),
    #[error("font units-per-em is invalid")]
    InvalidUnitsPerEm,
}

impl GlyphBackend for VendoredOutlineBackend {
    fn kind(&self) -> GlyphBackendKind {
        GlyphBackendKind::Outline
    }

    fn shape_text_geometry(
        &self,
        text: &str,
        attrs: &TextAttributes,
    ) -> Result<Vec<TextGeometryPrimitive>, ExportError> {
        let font_path =
            vendored_asset_path_for_family(&attrs.family).ok_or(ExportError::InvalidTextHeight)?;
        let font_bytes = std::fs::read(font_path).map_err(|_| ExportError::InvalidTextHeight)?;
        layout_text_outline(text, attrs, &font_bytes)
    }
}

pub fn flatten_glyph_from_font_bytes(
    font_bytes: &[u8],
    codepoint: char,
    height_nm: i64,
    tolerance_nm: i64,
) -> Result<FlattenedGlyphFixture, OutlineError> {
    let face = Face::parse(font_bytes, 0).map_err(|_| OutlineError::InvalidFont)?;
    let glyph_id = face
        .glyph_index(codepoint)
        .ok_or(OutlineError::MissingGlyph(codepoint as u32))?;
    flatten_glyph_from_face(&face, glyph_id, codepoint, height_nm, tolerance_nm)
}

fn layout_text_outline(
    text: &str,
    attrs: &TextAttributes,
    font_bytes: &[u8],
) -> Result<Vec<TextGeometryPrimitive>, ExportError> {
    if attrs.height_nm <= 0 {
        return Err(ExportError::InvalidTextHeight);
    }
    if attrs.stroke_width_nm <= 0 {
        return Err(ExportError::InvalidTextStrokeWidth);
    }

    let attrs = normalize_text_attributes(attrs.clone());
    let mut face = Face::parse(font_bytes, 0).map_err(|_| ExportError::InvalidTextHeight)?;
    apply_font_variations(&mut face, &attrs);
    let units_per_em = face.units_per_em();
    if units_per_em == 0 {
        return Err(ExportError::InvalidTextHeight);
    }
    let scale_nm = attrs.height_nm as f64 / units_per_em as f64;
    let line_pitch_nm = (attrs.height_nm as i128 * attrs.line_spacing_ratio_ppm as i128
        / 1_000_000_i128) as i64
        + attrs.stroke_width_nm;
    let flatten_tolerance_nm = outline_flatten_tolerance_nm(&attrs);
    let decoded_lines = text
        .split('\n')
        .map(|line| decode_outline_line(line, &face, scale_nm, flatten_tolerance_nm))
        .collect::<Result<Vec<_>, ExportError>>()?;

    let mut block_min_x_nm = 0_i64;
    let mut block_max_x_nm = 0_i64;
    let mut block_min_y_nm = 0_i64;
    let mut block_max_y_nm = 0_i64;
    for (line_index, line) in decoded_lines.iter().enumerate() {
        let baseline_y_nm = line_baseline_y_nm(line_index, line_pitch_nm, attrs.mirrored);
        let line_min_y_nm = baseline_y_nm + line.min_y_nm;
        let line_max_y_nm = baseline_y_nm + line.max_y_nm;
        if line_index == 0 {
            block_min_x_nm = line.min_x_nm;
            block_max_x_nm = line.max_x_nm;
            block_min_y_nm = line_min_y_nm;
            block_max_y_nm = line_max_y_nm;
        } else {
            block_min_x_nm = block_min_x_nm.min(line.min_x_nm);
            block_max_x_nm = block_max_x_nm.max(line.max_x_nm);
            block_min_y_nm = block_min_y_nm.min(line_min_y_nm);
            block_max_y_nm = block_max_y_nm.max(line_max_y_nm);
        }
    }

    let anchor_shift_x_nm = anchor_shift(attrs.h_align, block_min_x_nm, block_max_x_nm);
    let anchor_shift_y_nm = anchor_shift(attrs.v_align, block_min_y_nm, block_max_y_nm);
    let mut geometries = Vec::new();
    for (line_index, line) in decoded_lines.iter().enumerate() {
        let baseline_y_nm = line_baseline_y_nm(line_index, line_pitch_nm, attrs.mirrored);
        for fill in &line.fills {
            let transformed_fill = TextResolvedFill {
                regions: fill
                    .regions
                    .iter()
                    .map(|region| TextFilledRegion {
                        outer: region
                            .outer
                            .iter()
                            .map(|point| {
                                transform_text_point(
                                    &attrs,
                                    point.x + anchor_shift_x_nm,
                                    baseline_y_nm + point.y + anchor_shift_y_nm,
                                )
                            })
                            .collect(),
                        holes: region
                            .holes
                            .iter()
                            .map(|ring| {
                                ring.iter()
                                    .map(|point| {
                                        transform_text_point(
                                            &attrs,
                                            point.x + anchor_shift_x_nm,
                                            baseline_y_nm + point.y + anchor_shift_y_nm,
                                        )
                                    })
                                    .collect()
                            })
                            .collect(),
                    })
                    .collect(),
            };
            geometries.extend(transformed_fill.into_primitives());
        }
    }

    Ok(geometries)
}

fn apply_font_variations(face: &mut Face<'_>, attrs: &TextAttributes) {
    if !face.is_variable() {
        return;
    }

    if attrs.family.0 == crate::text::FAMILY_INTER
        || attrs.family.0 == crate::text::FAMILY_INTER_DISPLAY
    {
        let mut optical_size = optical_size_points(attrs.height_nm);
        if attrs.family.0 == crate::text::FAMILY_INTER_DISPLAY {
            optical_size = optical_size.max(32.0);
        }
        let _ = face.set_variation(Tag::from_bytes(b"opsz"), optical_size);
    }

    if attrs.bold {
        let _ = face.set_variation(Tag::from_bytes(b"wght"), 700.0);
    } else if attrs.family.0 == crate::text::FAMILY_INTER_DISPLAY {
        let _ = face.set_variation(Tag::from_bytes(b"wght"), 500.0);
    }
}

fn optical_size_points(height_nm: i64) -> f32 {
    const NM_PER_POINT: f64 = 352_778.0;
    (height_nm as f64 / NM_PER_POINT).clamp(14.0, 32.0) as f32
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecodedOutlineLine {
    fills: Vec<TextResolvedFill>,
    min_x_nm: i64,
    max_x_nm: i64,
    min_y_nm: i64,
    max_y_nm: i64,
}

fn decode_outline_line(
    line: &str,
    face: &Face<'_>,
    scale_nm: f64,
    flatten_tolerance_nm: i64,
) -> Result<DecodedOutlineLine, ExportError> {
    let mut contours = Vec::new();
    let mut cursor_x_nm = 0_i64;
    let mut min_x_nm = 0_i64;
    let mut max_x_nm = 0_i64;
    let mut min_y_nm = 0_i64;
    let mut max_y_nm = 0_i64;
    let mut has_bounds = false;

    for ch in line.chars() {
        let glyph_id = face
            .glyph_index(ch)
            .ok_or(ExportError::UnsupportedSilkscreenTextCharacter(ch))?;
        let advance_nm = glyph_advance_nm(face, glyph_id, scale_nm);
        let glyph = flatten_glyph_from_face(
            face,
            glyph_id,
            ch,
            scaled_height_nm(scale_nm, face),
            flatten_tolerance_nm,
        )
        .map_err(|_| ExportError::UnsupportedSilkscreenTextCharacter(ch))?;

        for contour in glyph.contours {
            if contour.points.is_empty() {
                continue;
            }
            let shifted = contour
                .points
                .into_iter()
                .map(|point| FlattenedOutlinePoint {
                    x_nm: point.x_nm + cursor_x_nm,
                    y_nm: point.y_nm,
                })
                .collect::<Vec<_>>();
            for point in &shifted {
                include_bounds_nm(
                    &mut min_x_nm,
                    &mut max_x_nm,
                    &mut min_y_nm,
                    &mut max_y_nm,
                    &mut has_bounds,
                    point.x_nm,
                    point.y_nm,
                );
            }
            contours.push(shifted);
        }

        cursor_x_nm += advance_nm;
    }

    if !has_bounds {
        max_x_nm = cursor_x_nm;
    }

    Ok(DecodedOutlineLine {
        fills: flattened_contours_to_contour_set(contours)
            .into_iter()
            .map(resolve_contour_set_non_zero_scanline)
            .collect(),
        min_x_nm,
        max_x_nm,
        min_y_nm,
        max_y_nm,
    })
}

fn outline_flatten_tolerance_nm(attrs: &TextAttributes) -> i64 {
    // Datum's doctrine is high-fidelity text across all intents. Fabrication
    // concerns are handled by validation/policy, not by intentionally coarse
    // outline flattening in the renderer path.
    //
    // Use a size-relative geometric error budget instead of one hidden
    // constant. This keeps the approximation deterministic while scaling
    // quality with the authored text size.
    //
    // Ratio: 0.001 of height, bounded to keep both tiny and huge text sane.
    ((attrs.height_nm as i128) / 100_000_i128).clamp(50_i128, 250_i128) as i64
}

fn flattened_contours_to_contour_set(
    contours: Vec<Vec<FlattenedOutlinePoint>>,
) -> Option<TextContourSet> {
    let rings = contours
        .into_iter()
        .filter_map(|points| normalize_closed_ring(points))
        .map(|points| TextContourRing {
            signed_area_nm2: signed_area_nm2(&points),
            points: points
                .into_iter()
                .map(|point| crate::ir::geometry::Point {
                    x: point.x_nm,
                    y: point.y_nm,
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    if rings.is_empty() {
        return None;
    }
    Some(TextContourSet {
        fill_rule: TextFillRule::NonZero,
        rings,
    })
}

fn resolve_contour_set_non_zero_scanline(contours: TextContourSet) -> TextResolvedFill {
    debug_assert_eq!(contours.fill_rule, TextFillRule::NonZero);
    const EPS: f64 = 1e-6;

    let mut ys: Vec<f64> = contours
        .rings
        .iter()
        .flat_map(|ring| ring.points.iter().map(|point| point.y as f64))
        .collect();
    ys.sort_by(|a, b| a.total_cmp(b));
    ys.dedup_by(|a, b| (*a - *b).abs() <= EPS);
    if ys.len() < 2 {
        return TextResolvedFill {
            regions: Vec::new(),
        };
    }

    let mut regions = Vec::new();
    for band in ys.windows(2) {
        let y0 = band[0];
        let y1 = band[1];
        if y1 - y0 <= EPS {
            continue;
        }
        let y_mid = (y0 + y1) * 0.5;
        let mut crossings: Vec<(f64, f64, f64, i32)> = Vec::new();

        for ring in &contours.rings {
            for index in 0..ring.points.len() {
                let a = ring.points[index];
                let b = ring.points[(index + 1) % ring.points.len()];
                let ay = a.y as f64;
                let by = b.y as f64;
                if (ay - by).abs() <= EPS {
                    continue;
                }
                let min_y = ay.min(by);
                let max_y = ay.max(by);
                if y_mid < min_y || y_mid >= max_y {
                    continue;
                }
                let x_at = |y: f64| {
                    let t = (y - ay) / (by - ay);
                    a.x as f64 + (b.x as f64 - a.x as f64) * t
                };
                let winding_delta = if ay <= y_mid && by > y_mid { 1 } else { -1 };
                crossings.push((x_at(y_mid), x_at(y0), x_at(y1), winding_delta));
            }
        }

        crossings.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut winding = 0_i32;
        let mut span_start: Option<(f64, f64)> = None;

        for crossing in crossings {
            let previous_winding = winding;
            winding += crossing.3;
            if previous_winding == 0 && winding != 0 {
                span_start = Some((crossing.1, crossing.2));
            } else if previous_winding != 0 && winding == 0 {
                if let Some((left_x0, left_x1)) = span_start.take() {
                    if crossing.0 - left_x0 > EPS {
                        regions.push(TextFilledRegion {
                            outer: vec![
                                crate::ir::geometry::Point {
                                    x: left_x0.round() as i64,
                                    y: y0.round() as i64,
                                },
                                crate::ir::geometry::Point {
                                    x: crossing.1.round() as i64,
                                    y: y0.round() as i64,
                                },
                                crate::ir::geometry::Point {
                                    x: crossing.2.round() as i64,
                                    y: y1.round() as i64,
                                },
                                crate::ir::geometry::Point {
                                    x: left_x1.round() as i64,
                                    y: y1.round() as i64,
                                },
                            ],
                            holes: Vec::new(),
                        });
                    }
                }
            }
        }
    }

    TextResolvedFill { regions }
}

fn normalize_closed_ring(
    mut points: Vec<FlattenedOutlinePoint>,
) -> Option<Vec<FlattenedOutlinePoint>> {
    if points.len() < 3 {
        return None;
    }
    if points.first() == points.last() {
        points.pop();
    }
    if points.len() < 3 {
        return None;
    }
    Some(points)
}

fn signed_area_nm2(ring: &[FlattenedOutlinePoint]) -> i128 {
    let mut area = 0_i128;
    for index in 0..ring.len() {
        let a = &ring[index];
        let b = &ring[(index + 1) % ring.len()];
        area += i128::from(a.x_nm) * i128::from(b.y_nm) - i128::from(b.x_nm) * i128::from(a.y_nm);
    }
    area
}

fn scaled_height_nm(scale_nm: f64, face: &Face<'_>) -> i64 {
    (scale_nm * f64::from(face.units_per_em())).round() as i64
}

fn glyph_advance_nm(face: &Face<'_>, glyph_id: GlyphId, scale_nm: f64) -> i64 {
    let advance_units = face
        .glyph_hor_advance(glyph_id)
        .map(i32::from)
        .or_else(|| {
            face.glyph_bounding_box(glyph_id)
                .map(|bbox| i32::from(bbox.x_max) - i32::from(bbox.x_min))
        })
        .unwrap_or_default();
    (f64::from(advance_units) * scale_nm).round() as i64
}

fn include_bounds_nm(
    min_x_nm: &mut i64,
    max_x_nm: &mut i64,
    min_y_nm: &mut i64,
    max_y_nm: &mut i64,
    has_bounds: &mut bool,
    x_nm: i64,
    y_nm: i64,
) {
    if !*has_bounds {
        *min_x_nm = x_nm;
        *max_x_nm = x_nm;
        *min_y_nm = y_nm;
        *max_y_nm = y_nm;
        *has_bounds = true;
    } else {
        *min_x_nm = (*min_x_nm).min(x_nm);
        *max_x_nm = (*max_x_nm).max(x_nm);
        *min_y_nm = (*min_y_nm).min(y_nm);
        *max_y_nm = (*max_y_nm).max(y_nm);
    }
}

fn flatten_glyph_from_face(
    face: &Face<'_>,
    glyph_id: GlyphId,
    codepoint: char,
    height_nm: i64,
    tolerance_nm: i64,
) -> Result<FlattenedGlyphFixture, OutlineError> {
    let units_per_em = face.units_per_em();
    if units_per_em == 0 {
        return Err(OutlineError::InvalidUnitsPerEm);
    }
    let scale_nm = height_nm as f64 / units_per_em as f64;
    let tolerance_units = (tolerance_nm as f64 / scale_nm).max(0.25);
    let mut builder = FlatteningOutlineBuilder::new(scale_nm, tolerance_units);
    if face.outline_glyph(glyph_id, &mut builder).is_none() {
        return Ok(FlattenedGlyphFixture {
            codepoint: codepoint as u32,
            contours: Vec::new(),
        });
    }
    Ok(FlattenedGlyphFixture {
        codepoint: codepoint as u32,
        contours: builder.finish(),
    })
}

struct FlatteningOutlineBuilder {
    contours: Vec<FlattenedOutlineContour>,
    current_points: Vec<FlattenedOutlinePoint>,
    current: Option<(f64, f64)>,
    start: Option<(f64, f64)>,
    scale_nm: f64,
    tolerance_units: f64,
}

impl FlatteningOutlineBuilder {
    fn new(scale_nm: f64, tolerance_units: f64) -> Self {
        Self {
            contours: Vec::new(),
            current_points: Vec::new(),
            current: None,
            start: None,
            scale_nm,
            tolerance_units,
        }
    }

    fn finish(mut self) -> Vec<FlattenedOutlineContour> {
        self.flush_open_contour(false);
        self.contours
    }

    fn push_point(&mut self, x: f64, y: f64) {
        let point = FlattenedOutlinePoint {
            x_nm: (x * self.scale_nm).round() as i64,
            y_nm: (-y * self.scale_nm).round() as i64,
        };
        if self.current_points.last() != Some(&point) {
            self.current_points.push(point);
        }
        self.current = Some((x, y));
    }

    fn flush_open_contour(&mut self, close: bool) {
        if self.current_points.is_empty() {
            self.current = None;
            self.start = None;
            return;
        }
        if close
            && self.current_points.first() != self.current_points.last()
            && let Some(first) = self.current_points.first().cloned()
        {
            self.current_points.push(first);
        }
        self.contours.push(FlattenedOutlineContour {
            points: std::mem::take(&mut self.current_points),
        });
        self.current = None;
        self.start = None;
    }
}

impl OutlineBuilder for FlatteningOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.flush_open_contour(false);
        let point = (f64::from(x), f64::from(y));
        self.start = Some(point);
        self.push_point(point.0, point.1);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.push_point(f64::from(x), f64::from(y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let Some(p0) = self.current else {
            return;
        };
        let p1 = (f64::from(x1), f64::from(y1));
        let p2 = (f64::from(x), f64::from(y));
        flatten_quadratic(p0, p1, p2, self.tolerance_units, &mut |px, py| {
            self.push_point(px, py);
        });
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let Some(p0) = self.current else {
            return;
        };
        let p1 = (f64::from(x1), f64::from(y1));
        let p2 = (f64::from(x2), f64::from(y2));
        let p3 = (f64::from(x), f64::from(y));
        flatten_cubic(p0, p1, p2, p3, self.tolerance_units, &mut |px, py| {
            self.push_point(px, py);
        });
    }

    fn close(&mut self) {
        self.flush_open_contour(true);
    }
}

fn flatten_quadratic(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    tolerance: f64,
    push: &mut impl FnMut(f64, f64),
) {
    flatten_quadratic_recursive(p0, p1, p2, tolerance, 0, push);
}

fn flatten_quadratic_recursive(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    tolerance: f64,
    depth: u8,
    push: &mut impl FnMut(f64, f64),
) {
    if depth >= 16 || quadratic_is_flat_enough(p0, p1, p2, tolerance) {
        push(p2.0, p2.1);
        return;
    }
    let p01 = midpoint(p0, p1);
    let p12 = midpoint(p1, p2);
    let p012 = midpoint(p01, p12);
    flatten_quadratic_recursive(p0, p01, p012, tolerance, depth + 1, push);
    flatten_quadratic_recursive(p012, p12, p2, tolerance, depth + 1, push);
}

fn flatten_cubic(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    tolerance: f64,
    push: &mut impl FnMut(f64, f64),
) {
    flatten_cubic_recursive(p0, p1, p2, p3, tolerance, 0, push);
}

fn flatten_cubic_recursive(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    tolerance: f64,
    depth: u8,
    push: &mut impl FnMut(f64, f64),
) {
    if depth >= 16 || cubic_is_flat_enough(p0, p1, p2, p3, tolerance) {
        push(p3.0, p3.1);
        return;
    }
    let p01 = midpoint(p0, p1);
    let p12 = midpoint(p1, p2);
    let p23 = midpoint(p2, p3);
    let p012 = midpoint(p01, p12);
    let p123 = midpoint(p12, p23);
    let p0123 = midpoint(p012, p123);
    flatten_cubic_recursive(p0, p01, p012, p0123, tolerance, depth + 1, push);
    flatten_cubic_recursive(p0123, p123, p23, p3, tolerance, depth + 1, push);
}

fn midpoint(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    ((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5)
}

fn quadratic_is_flat_enough(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    tolerance: f64,
) -> bool {
    point_line_distance(p1, p0, p2) <= tolerance
}

fn cubic_is_flat_enough(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    tolerance: f64,
) -> bool {
    point_line_distance(p1, p0, p3) <= tolerance && point_line_distance(p2, p0, p3) <= tolerance
}

fn point_line_distance(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    if dx == 0.0 && dy == 0.0 {
        return ((p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)).sqrt();
    }
    let numerator = ((p.0 - a.0) * dy - (p.1 - a.1) * dx).abs();
    let denominator = (dx * dx + dy * dy).sqrt();
    numerator / denominator
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::geometry::Point;
    use crate::text::registry::FAMILY_DEV_DEJAVU_SANS;
    use crate::text::{TextFamilyId, TextHAlign, TextRenderIntent, TextStyleId, TextVAlign};

    #[test]
    fn flatten_glyph_from_vendored_dejavu_font_produces_contours() {
        let bytes = std::fs::read(crate::text::vendored_font_asset_path("dev/DejaVuSans.ttf"))
            .expect("vendored DejaVuSans.ttf should read");
        let glyph = flatten_glyph_from_font_bytes(&bytes, 'A', 1_000_000, 50_000)
            .expect("outline glyph should flatten");
        assert_eq!(glyph.codepoint, 'A' as u32);
        assert!(!glyph.contours.is_empty());
        assert!(glyph.contours.iter().any(|c| c.points.len() >= 3));
    }

    #[test]
    fn vendored_outline_backend_shapes_annotation_text() {
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
            render_intent: TextRenderIntent::Annotation,
            family: TextFamilyId(FAMILY_DEV_DEJAVU_SANS.to_string()),
            family_source: crate::text::TextFamilySource::Explicit,
            style: TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let strokes = vendored_outline_backend()
            .shape_text("TEST", &attrs)
            .expect("dev outline backend should shape text");
        assert!(!strokes.is_empty());
    }

    fn point_in_polygon(point: Point, ring: &[Point]) -> bool {
        let mut inside = false;
        let px = point.x as f64;
        let py = point.y as f64;
        for index in 0..ring.len() {
            let a = &ring[index];
            let b = &ring[(index + 1) % ring.len()];
            let ax = a.x as f64;
            let ay = a.y as f64;
            let bx = b.x as f64;
            let by = b.y as f64;
            let crosses = (ay > py) != (by > py);
            if !crosses {
                continue;
            }
            let x_intersection = ax + (py - ay) * (bx - ax) / (by - ay);
            if x_intersection > px {
                inside = !inside;
            }
        }
        inside
    }

    fn winding_number(point: Point, ring: &[Point]) -> i32 {
        let px = point.x as f64;
        let py = point.y as f64;
        let mut winding = 0_i32;
        for index in 0..ring.len() {
            let a = ring[index];
            let b = ring[(index + 1) % ring.len()];
            let ax = a.x as f64;
            let ay = a.y as f64;
            let bx = b.x as f64;
            let by = b.y as f64;
            if ay <= py {
                if by > py {
                    let is_left = (bx - ax) * (py - ay) - (px - ax) * (by - ay);
                    if is_left > 0.0 {
                        winding += 1;
                    }
                }
            } else if by <= py {
                let is_left = (bx - ax) * (py - ay) - (px - ax) * (by - ay);
                if is_left < 0.0 {
                    winding -= 1;
                }
            }
        }
        winding
    }

    fn resolved_fill_contains(point: Point, fill: &TextResolvedFill) -> bool {
        fill.regions
            .iter()
            .any(|region| point_in_polygon(point, &region.outer))
    }

    fn raw_non_zero_contains(point: Point, contour_set: &TextContourSet) -> bool {
        contour_set
            .rings
            .iter()
            .map(|ring| winding_number(point, &ring.points))
            .sum::<i32>()
            != 0
    }

    #[test]
    fn resolved_fill_matches_raw_non_zero_regression_corpus() {
        let corpus = [
            (crate::text::FAMILY_INTER, ['A', 'D', 'R', 'Q', 'O']),
            (
                crate::text::FAMILY_IBM_PLEX_SANS_CONDENSED,
                ['A', 'D', 'R', 'Q', 'O'],
            ),
            (
                crate::text::FAMILY_JETBRAINS_MONO,
                ['A', 'D', 'R', 'Q', 'O'],
            ),
        ];

        for (family, glyphs) in corpus {
            let font_path =
                crate::text::vendored_asset_path_for_family(&TextFamilyId(family.to_string()))
                    .expect("vendored asset path should exist");
            let bytes = std::fs::read(font_path).expect("vendored font should read");
            let face = Face::parse(&bytes, 0).expect("font should parse");
            let scale_nm = 1_000_000_f64 / f64::from(face.units_per_em());

            for glyph in glyphs {
                let glyph_id = face
                    .glyph_index(glyph)
                    .expect("glyph should exist in vendored font");
                let raw = flatten_glyph_from_face(
                    &face,
                    glyph_id,
                    glyph,
                    scaled_height_nm(scale_nm, &face),
                    50_000,
                )
                .expect("glyph should flatten");
                let contour_set = flattened_contours_to_contour_set(
                    raw.contours
                        .into_iter()
                        .map(|contour| contour.points)
                        .collect(),
                )
                .expect("glyph contour set should exist");
                let resolved = resolve_contour_set_non_zero_scanline(contour_set.clone());

                let mut min_x = i64::MAX;
                let mut max_x = i64::MIN;
                let mut min_y = i64::MAX;
                let mut max_y = i64::MIN;
                for ring in &contour_set.rings {
                    for point in &ring.points {
                        min_x = min_x.min(point.x);
                        max_x = max_x.max(point.x);
                        min_y = min_y.min(point.y);
                        max_y = max_y.max(point.y);
                    }
                }

                let step_x = ((max_x - min_x).max(1) / 24).max(1);
                let step_y = ((max_y - min_y).max(1) / 24).max(1);

                let mut y = min_y + step_y / 2;
                while y < max_y {
                    let mut x = min_x + step_x / 2;
                    while x < max_x {
                        let point = Point { x, y };
                        assert_eq!(
                            raw_non_zero_contains(point, &contour_set),
                            resolved_fill_contains(point, &resolved),
                            "non-zero mismatch for family={family} glyph={glyph} at ({x},{y})"
                        );
                        x += step_x;
                    }
                    y += step_y;
                }
            }
        }
    }

    #[test]
    fn engine_outline_policy_uses_finer_curve_flattening_than_50um_fixture() {
        let bytes = std::fs::read(
            crate::text::vendored_asset_path_for_family(&TextFamilyId(
                crate::text::FAMILY_INTER.to_string(),
            ))
            .expect("inter asset path should exist"),
        )
        .expect("inter font should read");

        let coarse = flatten_glyph_from_font_bytes(&bytes, 'O', 1_000_000, 50_000)
            .expect("coarse O should flatten");
        let fine = flatten_glyph_from_font_bytes(
            &bytes,
            'O',
            1_000_000,
            outline_flatten_tolerance_nm(&TextAttributes {
                position: Point::zero(),
                rotation_degrees: 0,
                height_nm: 1_000_000,
                stroke_width_nm: 100_000,
                h_align: TextHAlign::Left,
                v_align: TextVAlign::Bottom,
                mirrored: false,
                keep_upright: false,
                line_spacing_ratio_ppm: 1_000_000,
                render_intent: TextRenderIntent::Manufacturing,
                family: TextFamilyId(crate::text::FAMILY_INTER.to_string()),
                family_source: crate::text::TextFamilySource::Explicit,
                style: TextStyleId::default(),
                italic: false,
                bold: false,
                style_class: None,
            }),
        )
        .expect("fine O should flatten");

        let coarse_points: usize = coarse
            .contours
            .iter()
            .map(|contour| contour.points.len())
            .sum();
        let fine_points: usize = fine
            .contours
            .iter()
            .map(|contour| contour.points.len())
            .sum();
        assert!(
            fine_points > coarse_points,
            "finer outline policy should increase curved glyph resolution, coarse={coarse_points}, fine={fine_points}"
        );
    }

    #[test]
    fn outline_flatten_tolerance_scales_with_text_height() {
        let tiny = outline_flatten_tolerance_nm(&TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 2_500_000,
            stroke_width_nm: 380_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            render_intent: TextRenderIntent::Manufacturing,
            family: TextFamilyId(crate::text::FAMILY_INTER.to_string()),
            family_source: crate::text::TextFamilySource::Explicit,
            style: TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        });
        let hero = outline_flatten_tolerance_nm(&TextAttributes {
            height_nm: 18_000_000,
            stroke_width_nm: 2_736_000,
            ..TextAttributes {
                position: Point::zero(),
                rotation_degrees: 0,
                height_nm: 2_500_000,
                stroke_width_nm: 380_000,
                h_align: TextHAlign::Left,
                v_align: TextVAlign::Bottom,
                mirrored: false,
                keep_upright: false,
                line_spacing_ratio_ppm: 1_000_000,
                render_intent: TextRenderIntent::Manufacturing,
                family: TextFamilyId(crate::text::FAMILY_INTER.to_string()),
                family_source: crate::text::TextFamilySource::Explicit,
                style: TextStyleId::default(),
                italic: false,
                bold: false,
                style_class: None,
            }
        });
        assert!(tiny < hero);
        assert_eq!(tiny, 50);
        assert_eq!(hero, 180);
    }

    #[test]
    fn optical_size_policy_tracks_text_height() {
        assert_eq!(optical_size_points(2_500_000), 14.0);
        assert!(optical_size_points(6_000_000) > 14.0);
        assert_eq!(optical_size_points(18_000_000), 32.0);
    }

    #[test]
    fn vendored_outline_backend_shapes_inter_text() {
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
            render_intent: TextRenderIntent::Annotation,
            family: TextFamilyId(crate::text::FAMILY_INTER.to_string()),
            family_source: crate::text::TextFamilySource::Explicit,
            style: TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let strokes = vendored_outline_backend()
            .shape_text("TEST", &attrs)
            .expect("inter outline backend should shape text");
        assert!(!strokes.is_empty());
    }

    #[test]
    fn inter_display_uses_distinct_variable_font_instance() {
        let base = TextAttributes {
            position: Point::zero(),
            rotation_degrees: 0,
            height_nm: 1_000_000,
            stroke_width_nm: 100_000,
            h_align: TextHAlign::Left,
            v_align: TextVAlign::Bottom,
            mirrored: false,
            keep_upright: false,
            line_spacing_ratio_ppm: 1_000_000,
            render_intent: TextRenderIntent::Annotation,
            family: TextFamilyId(crate::text::FAMILY_INTER.to_string()),
            family_source: crate::text::TextFamilySource::Explicit,
            style: TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let display = TextAttributes {
            render_intent: TextRenderIntent::Branding,
            family: TextFamilyId(crate::text::FAMILY_INTER_DISPLAY.to_string()),
            ..base.clone()
        };

        let inter_geometry = vendored_outline_backend()
            .shape_text_geometry("BRAND", &base)
            .expect("inter geometry should shape");
        let display_geometry = vendored_outline_backend()
            .shape_text_geometry("BRAND", &display)
            .expect("inter display geometry should shape");

        assert_ne!(
            inter_geometry, display_geometry,
            "inter_display should not collapse to the same default geometry as inter"
        );
    }

    #[test]
    fn vendored_outline_backend_shapes_ibm_plex_text() {
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
            render_intent: TextRenderIntent::Documentation,
            family: TextFamilyId(crate::text::FAMILY_IBM_PLEX_SANS_CONDENSED.to_string()),
            family_source: crate::text::TextFamilySource::Explicit,
            style: TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let strokes = vendored_outline_backend()
            .shape_text("R7", &attrs)
            .expect("ibm plex condensed outline backend should shape text");
        assert!(!strokes.is_empty());
    }

    #[test]
    fn vendored_outline_backend_shapes_jetbrains_mono_text() {
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
            render_intent: TextRenderIntent::Documentation,
            family: TextFamilyId(crate::text::FAMILY_JETBRAINS_MONO.to_string()),
            family_source: crate::text::TextFamilySource::Explicit,
            style: TextStyleId::default(),
            italic: false,
            bold: false,
            style_class: None,
        };
        let strokes = vendored_outline_backend()
            .shape_text("v1.2.3", &attrs)
            .expect("jetbrains mono outline backend should shape text");
        assert!(!strokes.is_empty());
    }
}
