use std::path::PathBuf;

use super::backend::GlyphBackendKind;
use super::determinism::vendored_font_asset_path;
use super::semantic::{TextFamilyId, TextFamilySource, TextRenderIntent, TextStyleId};

pub const FAMILY_NEWSTROKE: &str = "newstroke";
pub const FAMILY_INTER: &str = "inter";
pub const FAMILY_INTER_DISPLAY: &str = "inter_display";
pub const FAMILY_IBM_PLEX_SANS_CONDENSED: &str = "ibm_plex_sans_condensed";
pub const FAMILY_IBM_PLEX_SANS_CONDENSED_MEDIUM: &str = "ibm_plex_sans_condensed_medium";
pub const FAMILY_IBM_PLEX_SANS_CONDENSED_SEMIBOLD: &str = "ibm_plex_sans_condensed_semibold";
pub const FAMILY_IBM_PLEX_MONO: &str = "ibm_plex_mono";
pub const FAMILY_IBM_PLEX_MONO_MEDIUM: &str = "ibm_plex_mono_medium";
pub const FAMILY_JETBRAINS_MONO: &str = "jetbrains_mono";
pub const FAMILY_DEV_DEJAVU_SANS: &str = "dev_dejavu_sans";

pub const STYLE_REGULAR: &str = "regular";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontFamilyEntry {
    pub family_id: &'static str,
    pub default_style_id: &'static str,
    pub backend_kind: GlyphBackendKind,
    pub asset_relpath: Option<&'static str>,
}

const FONT_FAMILIES: &[FontFamilyEntry] = &[
    FontFamilyEntry {
        family_id: FAMILY_NEWSTROKE,
        default_style_id: STYLE_REGULAR,
        backend_kind: GlyphBackendKind::Stroke,
        asset_relpath: None,
    },
    FontFamilyEntry {
        family_id: FAMILY_INTER,
        default_style_id: STYLE_REGULAR,
        backend_kind: GlyphBackendKind::Outline,
        asset_relpath: Some("inter/InterVariable.ttf"),
    },
    FontFamilyEntry {
        family_id: FAMILY_INTER_DISPLAY,
        default_style_id: STYLE_REGULAR,
        backend_kind: GlyphBackendKind::Outline,
        asset_relpath: Some("inter/InterVariable.ttf"),
    },
    FontFamilyEntry {
        family_id: FAMILY_IBM_PLEX_SANS_CONDENSED,
        default_style_id: STYLE_REGULAR,
        backend_kind: GlyphBackendKind::Outline,
        asset_relpath: Some("ibm_plex_sans_condensed/IBMPlexSansCondensed-Regular.ttf"),
    },
    FontFamilyEntry {
        family_id: FAMILY_IBM_PLEX_SANS_CONDENSED_MEDIUM,
        default_style_id: STYLE_REGULAR,
        backend_kind: GlyphBackendKind::Outline,
        asset_relpath: Some("ibm_plex_sans_condensed/IBMPlexSansCondensed-Medium.ttf"),
    },
    FontFamilyEntry {
        family_id: FAMILY_IBM_PLEX_SANS_CONDENSED_SEMIBOLD,
        default_style_id: STYLE_REGULAR,
        backend_kind: GlyphBackendKind::Outline,
        asset_relpath: Some("ibm_plex_sans_condensed/IBMPlexSansCondensed-SemiBold.ttf"),
    },
    FontFamilyEntry {
        family_id: FAMILY_IBM_PLEX_MONO,
        default_style_id: STYLE_REGULAR,
        backend_kind: GlyphBackendKind::Outline,
        asset_relpath: Some("ibm_plex_mono/IBMPlexMono-Regular.ttf"),
    },
    FontFamilyEntry {
        family_id: FAMILY_IBM_PLEX_MONO_MEDIUM,
        default_style_id: STYLE_REGULAR,
        backend_kind: GlyphBackendKind::Outline,
        asset_relpath: Some("ibm_plex_mono/IBMPlexMono-Medium.ttf"),
    },
    FontFamilyEntry {
        family_id: FAMILY_JETBRAINS_MONO,
        default_style_id: STYLE_REGULAR,
        backend_kind: GlyphBackendKind::Outline,
        asset_relpath: Some("jetbrains_mono/JetBrainsMono-Regular.ttf"),
    },
    FontFamilyEntry {
        family_id: FAMILY_DEV_DEJAVU_SANS,
        default_style_id: STYLE_REGULAR,
        backend_kind: GlyphBackendKind::Outline,
        asset_relpath: Some("dev/DejaVuSans.ttf"),
    },
];

pub fn family_entry(family: &TextFamilyId) -> Option<FontFamilyEntry> {
    FONT_FAMILIES
        .iter()
        .copied()
        .find(|entry| entry.family_id == family.0)
}

pub fn family_backend_kind(family: &TextFamilyId) -> GlyphBackendKind {
    family_entry(family)
        .map(|entry| entry.backend_kind)
        .unwrap_or(GlyphBackendKind::Stroke)
}

pub fn default_family_for_intent(intent: TextRenderIntent) -> TextFamilyId {
    // Rendering Book (docs/gui/DATUM_RENDERING_BOOK.md §5): IBM Plex is the program
    // typeface. Sans Condensed for all text (silk, annotation, UI, docs); SemiBold
    // for branding/display. Mono (registered above) is reserved for aligned numeric
    // data and is selected explicitly, not as an intent default.
    match intent {
        TextRenderIntent::Manufacturing => {
            TextFamilyId(FAMILY_IBM_PLEX_SANS_CONDENSED.to_string())
        }
        TextRenderIntent::Annotation => TextFamilyId(FAMILY_IBM_PLEX_SANS_CONDENSED.to_string()),
        TextRenderIntent::Branding => {
            TextFamilyId(FAMILY_IBM_PLEX_SANS_CONDENSED_SEMIBOLD.to_string())
        }
        TextRenderIntent::Documentation => {
            TextFamilyId(FAMILY_IBM_PLEX_SANS_CONDENSED.to_string())
        }
        TextRenderIntent::UiPreview => TextFamilyId(FAMILY_IBM_PLEX_SANS_CONDENSED.to_string()),
    }
}

pub fn default_style_for_family(family: &TextFamilyId) -> TextStyleId {
    family_entry(family)
        .map(|entry| TextStyleId(entry.default_style_id.to_string()))
        .unwrap_or_else(|| TextStyleId(STYLE_REGULAR.to_string()))
}

pub fn resolve_family_and_style(
    intent: TextRenderIntent,
    family_source: TextFamilySource,
    family: &TextFamilyId,
    style: &TextStyleId,
) -> (TextFamilyId, TextStyleId) {
    let using_legacy_defaults = family_source == TextFamilySource::ImplicitDefault
        && family.0 == FAMILY_NEWSTROKE
        && style.0 == STYLE_REGULAR;
    if using_legacy_defaults {
        let resolved_family = default_family_for_intent(intent);
        let resolved_style = default_style_for_family(&resolved_family);
        return (resolved_family, resolved_style);
    }
    (family.clone(), style.clone())
}

pub fn vendored_asset_path_for_family(family: &TextFamilyId) -> Option<PathBuf> {
    family_entry(family)
        .and_then(|entry| entry.asset_relpath)
        .map(vendored_font_asset_path)
}

pub fn family_asset_is_vendored(family: &TextFamilyId) -> bool {
    vendored_asset_path_for_family(family)
        .map(|path| path.exists())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manufacturing_defaults_to_ibm_plex_sans_condensed() {
        assert_eq!(
            default_family_for_intent(TextRenderIntent::Manufacturing).0,
            FAMILY_IBM_PLEX_SANS_CONDENSED
        );
    }

    #[test]
    fn annotation_defaults_to_ibm_plex_sans_condensed() {
        assert_eq!(
            default_family_for_intent(TextRenderIntent::Annotation).0,
            FAMILY_IBM_PLEX_SANS_CONDENSED
        );
    }

    #[test]
    fn branding_defaults_to_ibm_plex_semibold() {
        assert_eq!(
            default_family_for_intent(TextRenderIntent::Branding).0,
            FAMILY_IBM_PLEX_SANS_CONDENSED_SEMIBOLD
        );
    }

    #[test]
    fn dev_dejavu_asset_is_present() {
        assert!(family_asset_is_vendored(&TextFamilyId(
            FAMILY_DEV_DEJAVU_SANS.to_string()
        )));
    }

    #[test]
    fn researched_product_assets_are_vendored() {
        assert!(family_asset_is_vendored(&TextFamilyId(
            FAMILY_INTER.to_string()
        )));
        assert!(family_asset_is_vendored(&TextFamilyId(
            FAMILY_INTER_DISPLAY.to_string()
        )));
        assert!(family_asset_is_vendored(&TextFamilyId(
            FAMILY_IBM_PLEX_SANS_CONDENSED.to_string()
        )));
        assert!(family_asset_is_vendored(&TextFamilyId(
            FAMILY_IBM_PLEX_SANS_CONDENSED_MEDIUM.to_string()
        )));
        assert!(family_asset_is_vendored(&TextFamilyId(
            FAMILY_IBM_PLEX_SANS_CONDENSED_SEMIBOLD.to_string()
        )));
        assert!(family_asset_is_vendored(&TextFamilyId(
            FAMILY_IBM_PLEX_MONO.to_string()
        )));
        assert!(family_asset_is_vendored(&TextFamilyId(
            FAMILY_IBM_PLEX_MONO_MEDIUM.to_string()
        )));
        assert!(family_asset_is_vendored(&TextFamilyId(
            FAMILY_JETBRAINS_MONO.to_string()
        )));
    }

    #[test]
    fn resolve_family_and_style_promotes_annotation_legacy_defaults() {
        let (family, style) = resolve_family_and_style(
            TextRenderIntent::Annotation,
            TextFamilySource::ImplicitDefault,
            &TextFamilyId(FAMILY_NEWSTROKE.to_string()),
            &TextStyleId(STYLE_REGULAR.to_string()),
        );
        assert_eq!(family.0, FAMILY_IBM_PLEX_SANS_CONDENSED);
        assert_eq!(style.0, STYLE_REGULAR);
    }

    #[test]
    fn resolve_family_and_style_promotes_manufacturing_legacy_defaults() {
        let (family, style) = resolve_family_and_style(
            TextRenderIntent::Manufacturing,
            TextFamilySource::ImplicitDefault,
            &TextFamilyId(FAMILY_NEWSTROKE.to_string()),
            &TextStyleId(STYLE_REGULAR.to_string()),
        );
        assert_eq!(family.0, FAMILY_IBM_PLEX_SANS_CONDENSED);
        assert_eq!(style.0, STYLE_REGULAR);
    }

    #[test]
    fn resolve_family_and_style_preserves_explicit_family_choice() {
        let (family, style) = resolve_family_and_style(
            TextRenderIntent::Annotation,
            TextFamilySource::Explicit,
            &TextFamilyId(FAMILY_JETBRAINS_MONO.to_string()),
            &TextStyleId(STYLE_REGULAR.to_string()),
        );
        assert_eq!(family.0, FAMILY_JETBRAINS_MONO);
        assert_eq!(style.0, STYLE_REGULAR);
    }

    #[test]
    fn resolve_family_and_style_preserves_explicit_newstroke_choice() {
        let (family, style) = resolve_family_and_style(
            TextRenderIntent::Manufacturing,
            TextFamilySource::Explicit,
            &TextFamilyId(FAMILY_NEWSTROKE.to_string()),
            &TextStyleId(STYLE_REGULAR.to_string()),
        );
        assert_eq!(family.0, FAMILY_NEWSTROKE);
        assert_eq!(style.0, STYLE_REGULAR);
    }
}
