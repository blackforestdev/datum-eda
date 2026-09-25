//! Prepared-scene integration for the output-only Datum Console.

use super::{
    ConsoleOverlayLayout, HitRegion, PreparedScene, Quad, ReviewWorkspaceState, ShellLayout,
    TextRun, Vertex, datum_console, quads_to_vertices,
};

pub(super) fn prepare(
    state: &ReviewWorkspaceState,
    shell: &ShellLayout,
    scale: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> anyhow::Result<(Vec<Vertex>, Option<ConsoleOverlayLayout>)> {
    Ok({
        let mut quads = Vec::<Quad>::new();
        let layout = datum_console::render_datum_console(
            state,
            shell,
            scale,
            &mut quads,
            text_runs,
            hit_regions,
        )?;
        (quads_to_vertices(&quads), layout)
    })
}

impl PreparedScene {
    pub(super) fn console_overlay_vertices(&self) -> &[Vertex] {
        &self.console_overlay_vertices
    }

    pub fn console_overlay_layout(&self) -> Option<ConsoleOverlayLayout> {
        self.console_overlay_layout
    }
}
