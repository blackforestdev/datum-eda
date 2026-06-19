mod backend;
mod determinism;
mod geometry;
mod layout;
mod mesh;
mod newstroke_data;
mod outline;
mod registry;
mod semantic;
mod stroke;

pub use backend::{GlyphBackend, GlyphBackendKind, default_backend_for_attributes};
pub use determinism::{
    FlattenedGlyphFixture, FlattenedOutlineContour, FlattenedOutlinePoint,
    OutlineDeterminismFixture, canonical_outline_fixture_json, golden_text_fixture_path,
    vendored_font_asset_path,
};
pub use geometry::{
    TextAttributes, TextContourRing, TextContourSet, TextFillRule, TextFilledRegion,
    TextGeometryPrimitive, TextHAlign, TextPolygon, TextResolvedFill, TextStroke, TextVAlign,
    default_stroke_width_nm,
};
pub use layout::{
    layout_text_geometry, layout_text_geometry_from_board_text, layout_text_strokes,
    layout_text_strokes_from_board_text,
};
pub use mesh::{
    Affine2DFixed, GlyphMeshAsset, GlyphMeshHandle, MeshRectEm, MeshVertexEm, TextGeometryBatch,
    TextGlyphInstance, TextMeshScene, layout_text_mesh, layout_text_mesh_from_board_text,
};
pub use outline::{OutlineError, flatten_glyph_from_font_bytes};
pub use registry::{
    FAMILY_DEV_DEJAVU_SANS, FAMILY_IBM_PLEX_SANS_CONDENSED, FAMILY_INTER, FAMILY_INTER_DISPLAY,
    FAMILY_JETBRAINS_MONO, FAMILY_NEWSTROKE, FontFamilyEntry, STYLE_REGULAR,
    default_family_for_intent, default_style_for_family, family_asset_is_vendored,
    family_backend_kind, family_entry, resolve_family_and_style, vendored_asset_path_for_family,
};
pub use semantic::{TextFamilyId, TextFamilySource, TextRenderIntent, TextStyleId};
