//! Properties and footprint choice share the placement configuration renderer.
use super::*;
#[allow(clippy::too_many_arguments)]
pub(super) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    _controls: &mut ControlMeshCache,
    _scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> bool {
    let placement = state
        .library
        .acquisition
        .as_ref()
        .is_some_and(|a| a.component_picker);
    if !placement
        && !state
            .library
            .configuration
            .as_ref()
            .is_some_and(|c| c.target.is_some() || c.footprint_chooser.is_some())
    {
        return false;
    }
    let start = (quads.len(), text.len(), hits.len());
    let scale = crate::text_presentation::chrome_scale::Scale::for_layout(layout);
    let logical = scale.logical_layout(layout.clone());
    crate::side_panels::library_surface::render_component_picker(
        state, &logical, quads, text, hits,
    );
    scale.quads(&mut quads[start.0..]);
    scale.text_geometry(&mut text[start.1..]);
    scale.hits(&mut hits[start.2..]);
    true
}
