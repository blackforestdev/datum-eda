//! Board-text field value vocabulary: cycle order and validation for
//! alignment, render intent, and font family choices.

use super::*;

pub(crate) fn next_h_align(current: &str) -> Result<&'static str> {
    validate_h_align(current)?;
    match current {
        "left" => Ok("center"),
        "center" => Ok("right"),
        "right" => Ok("left"),
        _ => unreachable!("validated horizontal alignment should cycle"),
    }
}

pub(crate) fn next_v_align(current: &str) -> Result<&'static str> {
    validate_v_align(current)?;
    match current {
        "top" => Ok("center"),
        "center" => Ok("bottom"),
        "bottom" => Ok("top"),
        _ => unreachable!("validated vertical alignment should cycle"),
    }
}

pub(crate) fn next_render_intent(current: &str) -> Result<&'static str> {
    validate_render_intent(current)?;
    match current {
        "manufacturing" => Ok("annotation"),
        "annotation" => Ok("branding"),
        "branding" => Ok("documentation"),
        "documentation" => Ok("ui_preview"),
        "ui_preview" => Ok("manufacturing"),
        _ => unreachable!("validated render intent should cycle"),
    }
}

pub(crate) fn next_font_family(current: &str) -> Result<&'static str> {
    validate_font_family(current)?;
    match current {
        FAMILY_NEWSTROKE => Ok(FAMILY_INTER),
        FAMILY_INTER => Ok(FAMILY_INTER_DISPLAY),
        FAMILY_INTER_DISPLAY => Ok(FAMILY_IBM_PLEX_SANS_CONDENSED),
        FAMILY_IBM_PLEX_SANS_CONDENSED => Ok(FAMILY_JETBRAINS_MONO),
        FAMILY_JETBRAINS_MONO => Ok(FAMILY_NEWSTROKE),
        _ => unreachable!("validated font family should cycle"),
    }
}

pub(crate) fn validate_h_align(value: &str) -> Result<()> {
    match value {
        "left" | "center" | "right" => Ok(()),
        other => anyhow::bail!(
            "unsupported board text horizontal alignment '{}'; expected one of: left, center, right",
            other
        ),
    }
}

pub(crate) fn validate_v_align(value: &str) -> Result<()> {
    match value {
        "top" | "center" | "bottom" => Ok(()),
        other => anyhow::bail!(
            "unsupported board text vertical alignment '{}'; expected one of: top, center, bottom",
            other
        ),
    }
}

pub(crate) fn validate_render_intent(value: &str) -> Result<()> {
    match value {
        "manufacturing" | "annotation" | "branding" | "documentation" | "ui_preview" => Ok(()),
        other => anyhow::bail!(
            "unsupported board text render intent '{}'; expected one of: manufacturing, annotation, branding, documentation, ui_preview",
            other
        ),
    }
}

pub(crate) fn validate_font_family(value: &str) -> Result<()> {
    match value {
        FAMILY_NEWSTROKE
        | FAMILY_INTER
        | FAMILY_INTER_DISPLAY
        | FAMILY_IBM_PLEX_SANS_CONDENSED
        | FAMILY_JETBRAINS_MONO => Ok(()),
        other => anyhow::bail!(
            "unsupported board text font family '{}'; expected one of: newstroke, inter, inter_display, ibm_plex_sans_condensed, jetbrains_mono",
            other
        ),
    }
}

pub(crate) fn family_source_to_string(source: TextFamilySource) -> &'static str {
    match source {
        TextFamilySource::ImplicitDefault => "implicit_default",
        TextFamilySource::Explicit => "explicit",
    }
}
