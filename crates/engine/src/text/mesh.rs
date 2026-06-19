use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use lyon_path::Path as LyonPath;
use lyon_path::math::point;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex, FillVertexConstructor,
    VertexBuffers,
};
use ttf_parser::{Face, GlyphId, OutlineBuilder, Tag};

use crate::board::BoardText;
use crate::export::ExportError;
use crate::ir::geometry::LayerId;

use super::geometry::TextAttributes;
use super::layout::{anchor_shift, line_baseline_y_nm, normalize_text_attributes};
use super::registry::vendored_asset_path_for_family;

const EM_NM: i64 = 1_000_000;

static GLYPH_MESH_CACHE: OnceLock<Mutex<BTreeMap<GlyphMeshHandle, GlyphMeshAsset>>> =
    OnceLock::new();
static FONT_BYTES_CACHE: OnceLock<Mutex<BTreeMap<PathBuf, Arc<Vec<u8>>>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GlyphMeshHandle {
    pub font_id: u32,
    pub glyph_id: u32,
    pub tolerance_class: u8,
    pub epoch: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshVertexEm {
    pub x_em_nm: i64,
    pub y_em_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphMeshAsset {
    pub handle: GlyphMeshHandle,
    pub vertices: Vec<MeshVertexEm>,
    pub indices: Vec<u32>,
    pub bbox_em_nm: MeshRectEm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshRectEm {
    pub min_x_em_nm: i64,
    pub min_y_em_nm: i64,
    pub max_x_em_nm: i64,
    pub max_y_em_nm: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Affine2DFixed {
    pub m11_ppm: i64,
    pub m12_ppm: i64,
    pub m21_ppm: i64,
    pub m22_ppm: i64,
    pub tx_nm: i64,
    pub ty_nm: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextGlyphInstance {
    pub glyph_handle: GlyphMeshHandle,
    pub origin_em_nm_x: i64,
    pub origin_em_nm_y: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextGeometryBatch {
    pub text_uuid: uuid::Uuid,
    pub layer: LayerId,
    pub world_transform: Affine2DFixed,
    pub block_bbox_em_nm: MeshRectEm,
    pub glyphs: Vec<TextGlyphInstance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextMeshScene {
    pub batch: TextGeometryBatch,
    pub glyph_mesh_assets: Vec<GlyphMeshAsset>,
}

#[derive(Debug, Clone)]
struct DecodedMeshLine {
    glyphs: Vec<DecodedGlyphInstance>,
    min_x_em_nm: i64,
    max_x_em_nm: i64,
    min_y_em_nm: i64,
    max_y_em_nm: i64,
}

#[derive(Debug, Clone, Copy)]
struct DecodedGlyphInstance {
    glyph_id: GlyphId,
    origin_x_em_nm: i64,
    origin_y_em_nm: i64,
}

pub fn layout_text_mesh_from_board_text(text: &BoardText) -> Result<TextMeshScene, ExportError> {
    layout_text_mesh(text, &TextAttributes::from_board_text(text))
}

pub fn layout_text_mesh(
    text: &BoardText,
    attrs: &TextAttributes,
) -> Result<TextMeshScene, ExportError> {
    let attrs = normalize_text_attributes(attrs.clone());
    if attrs.height_nm <= 0 {
        return Err(ExportError::InvalidTextHeight);
    }
    if attrs.stroke_width_nm <= 0 {
        return Err(ExportError::InvalidTextStrokeWidth);
    }

    let font_path =
        vendored_asset_path_for_family(&attrs.family).ok_or(ExportError::InvalidTextHeight)?;
    let font_bytes = cached_font_bytes(&font_path)?;
    let mut face =
        Face::parse(font_bytes.as_slice(), 0).map_err(|_| ExportError::InvalidTextHeight)?;
    apply_mesh_font_variations(&mut face, &attrs);
    let units_per_em = face.units_per_em();
    if units_per_em == 0 {
        return Err(ExportError::InvalidTextHeight);
    }

    let font_id = stable_font_id(&font_path, &attrs);
    let tolerance_class = tolerance_class_for_height(attrs.height_nm);
    let decoded_lines = text
        .text
        .split('\n')
        .map(|line| decode_mesh_line(line, &face))
        .collect::<Result<Vec<_>, ExportError>>()?;
    let line_pitch_em_nm = (EM_NM as i128 * attrs.line_spacing_ratio_ppm as i128 / 1_000_000_i128)
        as i64
        + (attrs.stroke_width_nm as i128 * EM_NM as i128 / attrs.height_nm as i128) as i64;

    let block_bbox = block_bbox_em(&decoded_lines, line_pitch_em_nm, attrs.mirrored);
    let anchor_shift_x_em_nm = anchor_shift(
        attrs.h_align,
        block_bbox.min_x_em_nm,
        block_bbox.max_x_em_nm,
    );
    let anchor_shift_y_em_nm = anchor_shift(
        attrs.v_align,
        block_bbox.min_y_em_nm,
        block_bbox.max_y_em_nm,
    );

    let mut asset_by_handle = BTreeMap::new();
    let mut glyphs = Vec::new();
    for (line_index, line) in decoded_lines.iter().enumerate() {
        let baseline_y_em_nm = line_baseline_y_nm(line_index, line_pitch_em_nm, attrs.mirrored);
        for glyph in &line.glyphs {
            let handle = GlyphMeshHandle {
                font_id,
                glyph_id: glyph.glyph_id.0.into(),
                tolerance_class,
                epoch: 0,
            };
            if !asset_by_handle.contains_key(&handle) {
                let asset = cached_glyph_mesh_asset(&face, glyph.glyph_id, handle, units_per_em)?;
                asset_by_handle.insert(handle, asset);
            }
            glyphs.push(TextGlyphInstance {
                glyph_handle: handle,
                origin_em_nm_x: glyph.origin_x_em_nm + anchor_shift_x_em_nm,
                origin_em_nm_y: baseline_y_em_nm + glyph.origin_y_em_nm + anchor_shift_y_em_nm,
            });
        }
    }

    Ok(TextMeshScene {
        batch: TextGeometryBatch {
            text_uuid: text.uuid,
            layer: text.layer,
            world_transform: text_world_transform(&attrs),
            block_bbox_em_nm: block_bbox,
            glyphs,
        },
        glyph_mesh_assets: asset_by_handle.into_values().collect(),
    })
}

fn cached_font_bytes(path: &Path) -> Result<Arc<Vec<u8>>, ExportError> {
    let cache = FONT_BYTES_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Some(bytes) = cache
        .lock()
        .expect("font bytes cache poisoned")
        .get(path)
        .cloned()
    {
        return Ok(bytes);
    }

    let bytes = Arc::new(std::fs::read(path).map_err(|_| ExportError::InvalidTextHeight)?);
    let mut cache = cache.lock().expect("font bytes cache poisoned");
    Ok(cache
        .entry(path.to_path_buf())
        .or_insert_with(|| bytes.clone())
        .clone())
}

fn decode_mesh_line(line: &str, face: &Face<'_>) -> Result<DecodedMeshLine, ExportError> {
    let mut glyphs = Vec::new();
    let mut cursor_x_em_nm = 0_i64;
    let mut min_x_em_nm = 0_i64;
    let mut max_x_em_nm = 0_i64;
    let mut min_y_em_nm = 0_i64;
    let mut max_y_em_nm = 0_i64;
    let mut has_bounds = false;

    for ch in line.chars() {
        let glyph_id = face
            .glyph_index(ch)
            .ok_or(ExportError::UnsupportedSilkscreenTextCharacter(ch))?;
        if let Some(bbox) = face.glyph_bounding_box(glyph_id) {
            include_bbox(
                &mut min_x_em_nm,
                &mut max_x_em_nm,
                &mut min_y_em_nm,
                &mut max_y_em_nm,
                &mut has_bounds,
                cursor_x_em_nm + font_units_to_em_nm(i64::from(bbox.x_min), face.units_per_em()),
                -font_units_to_em_nm(i64::from(bbox.y_max), face.units_per_em()),
                cursor_x_em_nm + font_units_to_em_nm(i64::from(bbox.x_max), face.units_per_em()),
                -font_units_to_em_nm(i64::from(bbox.y_min), face.units_per_em()),
            );
        }
        glyphs.push(DecodedGlyphInstance {
            glyph_id,
            origin_x_em_nm: cursor_x_em_nm,
            origin_y_em_nm: 0,
        });
        cursor_x_em_nm += glyph_advance_em_nm(face, glyph_id);
    }

    if !has_bounds {
        max_x_em_nm = cursor_x_em_nm;
    }

    Ok(DecodedMeshLine {
        glyphs,
        min_x_em_nm,
        max_x_em_nm,
        min_y_em_nm,
        max_y_em_nm,
    })
}

fn cached_glyph_mesh_asset(
    face: &Face<'_>,
    glyph_id: GlyphId,
    handle: GlyphMeshHandle,
    units_per_em: u16,
) -> Result<GlyphMeshAsset, ExportError> {
    let cache = GLYPH_MESH_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Some(asset) = cache
        .lock()
        .expect("glyph mesh cache mutex should not be poisoned")
        .get(&handle)
        .cloned()
    {
        return Ok(asset);
    }

    let asset = glyph_mesh_asset(face, glyph_id, handle, units_per_em)?;
    Ok(cache
        .lock()
        .expect("glyph mesh cache mutex should not be poisoned")
        .entry(handle)
        .or_insert_with(|| asset.clone())
        .clone())
}

fn glyph_mesh_asset(
    face: &Face<'_>,
    glyph_id: GlyphId,
    handle: GlyphMeshHandle,
    units_per_em: u16,
) -> Result<GlyphMeshAsset, ExportError> {
    let mut builder = LyonOutlineBuilder::default();
    if face.outline_glyph(glyph_id, &mut builder).is_none() {
        return Ok(empty_glyph_mesh_asset(handle));
    }
    let path = builder.finish();
    let mut tessellator = FillTessellator::new();
    let mut buffers: VertexBuffers<MeshVertexEm, u32> = VertexBuffers::new();
    tessellator
        .tessellate_path(
            &path,
            &FillOptions::default().with_fill_rule(FillRule::NonZero),
            &mut BuffersBuilder::new(
                &mut buffers,
                MeshVertexConstructor {
                    units_per_em: i64::from(units_per_em),
                },
            ),
        )
        .map_err(|_| ExportError::TextMeshTessellationFailed)?;
    let bbox = mesh_bbox(&buffers.vertices).unwrap_or(MeshRectEm {
        min_x_em_nm: 0,
        min_y_em_nm: 0,
        max_x_em_nm: 0,
        max_y_em_nm: 0,
    });
    Ok(GlyphMeshAsset {
        handle,
        vertices: buffers.vertices,
        indices: buffers.indices,
        bbox_em_nm: bbox,
    })
}

#[derive(Default)]
struct LyonOutlineBuilder {
    builder: lyon_path::path::Builder,
    has_open_contour: bool,
}

impl LyonOutlineBuilder {
    fn finish(mut self) -> LyonPath {
        if self.has_open_contour {
            self.builder.close();
        }
        self.builder.build()
    }
}

impl OutlineBuilder for LyonOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        if self.has_open_contour {
            self.builder.close();
        }
        self.builder.begin(point(x, -y));
        self.has_open_contour = true;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(point(x, -y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.builder
            .quadratic_bezier_to(point(x1, -y1), point(x, -y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.builder
            .cubic_bezier_to(point(x1, -y1), point(x2, -y2), point(x, -y));
    }

    fn close(&mut self) {
        if self.has_open_contour {
            self.builder.close();
            self.has_open_contour = false;
        }
    }
}

struct MeshVertexConstructor {
    units_per_em: i64,
}

impl FillVertexConstructor<MeshVertexEm> for MeshVertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex<'_>) -> MeshVertexEm {
        let position = vertex.position();
        MeshVertexEm {
            x_em_nm: font_units_to_em_nm(position.x.round() as i64, self.units_per_em as u16),
            y_em_nm: font_units_to_em_nm(position.y.round() as i64, self.units_per_em as u16),
        }
    }
}

fn text_world_transform(attrs: &TextAttributes) -> Affine2DFixed {
    let scale_ppm = attrs.height_nm;
    let angle = (attrs.rotation_degrees as f64).to_radians();
    let mirror = if attrs.mirrored { -1.0 } else { 1.0 };
    Affine2DFixed {
        m11_ppm: (angle.cos() * mirror * scale_ppm as f64).round() as i64,
        m12_ppm: (-angle.sin() * scale_ppm as f64).round() as i64,
        m21_ppm: (angle.sin() * mirror * scale_ppm as f64).round() as i64,
        m22_ppm: (angle.cos() * scale_ppm as f64).round() as i64,
        tx_nm: attrs.position.x,
        ty_nm: attrs.position.y,
    }
}

fn block_bbox_em(
    decoded_lines: &[DecodedMeshLine],
    line_pitch_em_nm: i64,
    mirrored: bool,
) -> MeshRectEm {
    let mut bbox = MeshRectEm {
        min_x_em_nm: 0,
        min_y_em_nm: 0,
        max_x_em_nm: 0,
        max_y_em_nm: 0,
    };
    for (line_index, line) in decoded_lines.iter().enumerate() {
        let baseline_y_em_nm = line_baseline_y_nm(line_index, line_pitch_em_nm, mirrored);
        let line_bbox = MeshRectEm {
            min_x_em_nm: line.min_x_em_nm,
            max_x_em_nm: line.max_x_em_nm,
            min_y_em_nm: baseline_y_em_nm + line.min_y_em_nm,
            max_y_em_nm: baseline_y_em_nm + line.max_y_em_nm,
        };
        if line_index == 0 {
            bbox = line_bbox;
        } else {
            bbox.min_x_em_nm = bbox.min_x_em_nm.min(line_bbox.min_x_em_nm);
            bbox.max_x_em_nm = bbox.max_x_em_nm.max(line_bbox.max_x_em_nm);
            bbox.min_y_em_nm = bbox.min_y_em_nm.min(line_bbox.min_y_em_nm);
            bbox.max_y_em_nm = bbox.max_y_em_nm.max(line_bbox.max_y_em_nm);
        }
    }
    bbox
}

fn apply_mesh_font_variations(face: &mut Face<'_>, attrs: &TextAttributes) {
    if !face.is_variable() {
        return;
    }
    if attrs.family.0 == super::FAMILY_INTER || attrs.family.0 == super::FAMILY_INTER_DISPLAY {
        let mut optical_size = optical_size_points(attrs.height_nm);
        if attrs.family.0 == super::FAMILY_INTER_DISPLAY {
            optical_size = optical_size.max(32.0);
        }
        let _ = face.set_variation(Tag::from_bytes(b"opsz"), optical_size);
    }
    if attrs.bold {
        let _ = face.set_variation(Tag::from_bytes(b"wght"), 700.0);
    } else if attrs.family.0 == super::FAMILY_INTER_DISPLAY {
        let _ = face.set_variation(Tag::from_bytes(b"wght"), 500.0);
    }
}

fn optical_size_points(height_nm: i64) -> f32 {
    const NM_PER_POINT: f64 = 352_778.0;
    (height_nm as f64 / NM_PER_POINT).clamp(14.0, 32.0) as f32
}

fn stable_font_id(path: &Path, attrs: &TextAttributes) -> u32 {
    let mut hash = 2_166_136_261_u32;
    for byte in path
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .chain(attrs.family.0.as_bytes())
        .chain(attrs.style.0.as_bytes())
        .chain(&mesh_font_variation_signature(attrs))
    {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    hash
}

fn mesh_font_variation_signature(attrs: &TextAttributes) -> [u8; 6] {
    let optical_size_centipoints =
        if attrs.family.0 == super::FAMILY_INTER || attrs.family.0 == super::FAMILY_INTER_DISPLAY {
            let mut optical_size = optical_size_points(attrs.height_nm);
            if attrs.family.0 == super::FAMILY_INTER_DISPLAY {
                optical_size = optical_size.max(32.0);
            }
            (optical_size * 100.0).round() as u16
        } else {
            0
        };
    let weight = if attrs.bold {
        700_u16
    } else if attrs.family.0 == super::FAMILY_INTER_DISPLAY {
        500_u16
    } else {
        400_u16
    };
    let style_flags = u16::from(attrs.italic);
    let mut signature = [0_u8; 6];
    signature[0..2].copy_from_slice(&optical_size_centipoints.to_le_bytes());
    signature[2..4].copy_from_slice(&weight.to_le_bytes());
    signature[4..6].copy_from_slice(&style_flags.to_le_bytes());
    signature
}

fn tolerance_class_for_height(height_nm: i64) -> u8 {
    match height_nm {
        ..=999_999 => 0,
        1_000_000..=9_999_999 => 1,
        _ => 2,
    }
}

fn glyph_advance_em_nm(face: &Face<'_>, glyph_id: GlyphId) -> i64 {
    let advance_units = face
        .glyph_hor_advance(glyph_id)
        .map(i64::from)
        .or_else(|| {
            face.glyph_bounding_box(glyph_id)
                .map(|bbox| i64::from(bbox.x_max) - i64::from(bbox.x_min))
        })
        .unwrap_or_default();
    font_units_to_em_nm(advance_units, face.units_per_em())
}

fn font_units_to_em_nm(value: i64, units_per_em: u16) -> i64 {
    ((value as i128 * EM_NM as i128) / i128::from(units_per_em)) as i64
}

fn include_bbox(
    min_x: &mut i64,
    max_x: &mut i64,
    min_y: &mut i64,
    max_y: &mut i64,
    has_bounds: &mut bool,
    x0: i64,
    y0: i64,
    x1: i64,
    y1: i64,
) {
    if !*has_bounds {
        *min_x = x0;
        *max_x = x1;
        *min_y = y0;
        *max_y = y1;
        *has_bounds = true;
    } else {
        *min_x = (*min_x).min(x0);
        *max_x = (*max_x).max(x1);
        *min_y = (*min_y).min(y0);
        *max_y = (*max_y).max(y1);
    }
}

fn mesh_bbox(vertices: &[MeshVertexEm]) -> Option<MeshRectEm> {
    let first = vertices.first()?;
    let mut bbox = MeshRectEm {
        min_x_em_nm: first.x_em_nm,
        max_x_em_nm: first.x_em_nm,
        min_y_em_nm: first.y_em_nm,
        max_y_em_nm: first.y_em_nm,
    };
    for vertex in &vertices[1..] {
        bbox.min_x_em_nm = bbox.min_x_em_nm.min(vertex.x_em_nm);
        bbox.max_x_em_nm = bbox.max_x_em_nm.max(vertex.x_em_nm);
        bbox.min_y_em_nm = bbox.min_y_em_nm.min(vertex.y_em_nm);
        bbox.max_y_em_nm = bbox.max_y_em_nm.max(vertex.y_em_nm);
    }
    Some(bbox)
}

fn empty_glyph_mesh_asset(handle: GlyphMeshHandle) -> GlyphMeshAsset {
    GlyphMeshAsset {
        handle,
        vertices: Vec::new(),
        indices: Vec::new(),
        bbox_em_nm: MeshRectEm {
            min_x_em_nm: 0,
            min_y_em_nm: 0,
            max_x_em_nm: 0,
            max_y_em_nm: 0,
        },
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::ir::geometry::Point;
    use crate::text::{
        FAMILY_INTER, TextFamilyId, TextFamilySource, TextHAlign, TextRenderIntent, TextStyleId,
        TextVAlign,
    };

    fn outline_text(content: &str) -> BoardText {
        BoardText {
            uuid: Uuid::from_u128(1),
            text: content.to_string(),
            position: Point { x: 10, y: 20 },
            rotation: 0,
            layer: 37,
            render_intent: TextRenderIntent::Annotation,
            family: TextFamilyId(FAMILY_INTER.to_string()),
            family_source: TextFamilySource::ImplicitDefault,
            style: TextStyleId::default(),
            height_nm: 2_000_000,
            stroke_width_nm: 200_000,
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
    fn repeated_outline_glyphs_share_mesh_assets() {
        let scene = layout_text_mesh_from_board_text(&outline_text("OODO"))
            .expect("outline mesh layout should succeed");

        assert_eq!(scene.batch.glyphs.len(), 4);
        assert_eq!(scene.glyph_mesh_assets.len(), 2);
        assert!(
            scene
                .glyph_mesh_assets
                .iter()
                .all(|asset| !asset.vertices.is_empty() && !asset.indices.is_empty())
        );
        let first_o = scene.batch.glyphs[0].glyph_handle;
        assert_eq!(scene.batch.glyphs[1].glyph_handle, first_o);
        assert_eq!(scene.batch.glyphs[3].glyph_handle, first_o);
    }

    #[test]
    fn space_glyph_uses_intentional_empty_mesh_asset() {
        let scene = layout_text_mesh_from_board_text(&outline_text(" "))
            .expect("space glyph mesh layout should succeed");

        assert_eq!(scene.batch.glyphs.len(), 1);
        assert_eq!(scene.glyph_mesh_assets.len(), 1);
        assert!(scene.glyph_mesh_assets[0].vertices.is_empty());
        assert!(scene.glyph_mesh_assets[0].indices.is_empty());
    }

    #[test]
    fn mesh_handles_encode_bold_variation() {
        let normal = layout_text_mesh_from_board_text(&outline_text("O"))
            .expect("normal mesh layout should succeed");
        let mut bold_text = outline_text("O");
        bold_text.bold = true;
        let bold =
            layout_text_mesh_from_board_text(&bold_text).expect("bold mesh layout should succeed");

        assert_ne!(
            normal.batch.glyphs[0].glyph_handle.font_id, bold.batch.glyphs[0].glyph_handle.font_id,
            "variable-font weight must be part of the glyph mesh cache identity"
        );
    }

    #[test]
    fn mesh_handles_encode_optical_size_variation() {
        let mut small_text = outline_text("O");
        small_text.height_nm = 5_000_000;
        let small = layout_text_mesh_from_board_text(&small_text)
            .expect("small mesh layout should succeed");
        let mut large_text = outline_text("O");
        large_text.height_nm = 9_000_000;
        let large = layout_text_mesh_from_board_text(&large_text)
            .expect("large mesh layout should succeed");

        assert_eq!(
            small.batch.glyphs[0].glyph_handle.tolerance_class,
            large.batch.glyphs[0].glyph_handle.tolerance_class,
            "test heights should isolate font variation identity from tolerance class"
        );
        assert_ne!(
            small.batch.glyphs[0].glyph_handle.font_id, large.batch.glyphs[0].glyph_handle.font_id,
            "variable-font optical size must be part of the glyph mesh cache identity"
        );
    }

    #[test]
    fn mesh_multiline_spacing_honors_line_spacing_ratio() {
        let mut text = outline_text("O\nO");
        text.line_spacing_ratio_ppm = 1_350_000;
        let scene =
            layout_text_mesh_from_board_text(&text).expect("multiline mesh layout should succeed");

        assert_eq!(scene.batch.glyphs.len(), 2);
        let pitch = scene.batch.glyphs[0].origin_em_nm_y - scene.batch.glyphs[1].origin_em_nm_y;
        assert_eq!(
            pitch, 1_450_000,
            "mesh baseline pitch should include line spacing ratio plus stroke allowance"
        );
    }
}
