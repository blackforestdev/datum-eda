//! New Project uses the same retained renderer and continuous scroll owner as Preferences.
use super::*;
use datum_gui_viewport::{ScreenRectPx, scroll::ScrollViewport};

impl Renderer {
    pub fn prepare_native_new_project(
        &mut self,
        dialog: &datum_gui_protocol::NewProjectDialogState,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> PreparedScene {
        self.prepare_native_new_project_scrolled(
            dialog,
            width,
            height,
            scale_factor,
            &mut ScrollViewport::default(),
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_native_new_project_scrolled(
        &mut self,
        dialog: &datum_gui_protocol::NewProjectDialogState,
        width: u32,
        height: u32,
        scale_factor: f32,
        scroll: &mut ScrollViewport,
        reveal_focus: bool,
    ) -> PreparedScene {
        let scale = scale_factor.max(0.01);
        let layout = PreparedScene::native_dialog_layout(width, height, scale);
        let (mut quads, mut text, mut hits) = (Vec::new(), Vec::new(), Vec::new());
        render_new_project_dialog_scrolled(
            dialog,
            &layout,
            true,
            &mut self.control_meshes,
            scale,
            &mut quads,
            &mut text,
            &mut hits,
            scroll,
            reveal_focus,
        );
        PreparedScene::from_dialog_parts(layout, quads, text, hits, scale, (width, height))
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_new_project_dialog(
    dialog: &datum_gui_protocol::NewProjectDialogState,
    layout: &ShellLayout,
    native_window: bool,
    controls: &mut ControlMeshCache,
    scale: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    render_new_project_dialog_scrolled(
        dialog,
        layout,
        native_window,
        controls,
        scale,
        quads,
        text,
        hits,
        &mut ScrollViewport::default(),
        false,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finish_body(
    dialog: &datum_gui_protocol::NewProjectDialogState,
    window: RectPx,
    top: f32,
    bottom: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
    starts: (usize, usize, usize),
    scroll: &mut ScrollViewport,
    reveal_focus: bool,
) {
    let viewport = ScreenRectPx {
        x: 0.0,
        y: top,
        width: window.width,
        height: (window.height - top).max(0.0),
    };
    let reflow = scroll.viewport != viewport || scroll.content_height != bottom - top;
    scroll.layout(viewport, bottom - top);
    if reveal_focus || reflow {
        let target = match dialog.focus {
            NewProjectFocus::ProjectName => HitTarget::NewProjectName,
            NewProjectFocus::Destination => HitTarget::NewProjectDestination,
            NewProjectFocus::UnitsChoice => HitTarget::NewProjectUnitsChoice(dialog.units_choice),
            NewProjectFocus::UnitsSummary => HitTarget::NewProjectUnitsSummary,
            NewProjectFocus::RetryGlobal => HitTarget::NewProjectRetryGlobal,
            NewProjectFocus::Cancel => HitTarget::NewProjectCancel,
            NewProjectFocus::Create => HitTarget::NewProjectCreate,
        };
        if let Some(hit) = hits[starts.2..].iter().find(|hit| hit.target == target) {
            scroll.reveal(hit.rect.y - top, hit.rect.y + hit.rect.height - top);
        }
    }
    if scroll.maximum() == 0.0 {
        return;
    }
    let dy = scroll.offset();
    for quad in &mut quads[starts.0..] {
        for point in &mut quad.points {
            point.1 -= dy;
        }
    }
    for run in &mut text[starts.1..] {
        run.y -= dy;
        if let Some(clip) = &mut run.clip_bounds {
            clip.y -= dy;
        }
    }
    for hit in &mut hits[starts.2..] {
        hit.rect.y -= dy;
    }
    crate::hit_clipping::clip_content(
        quads,
        text,
        hits,
        starts.0,
        starts.1,
        starts.2,
        RectPx {
            x: 0.0,
            y: top,
            width: (window.width - 11.0).max(0.0),
            height: viewport.height,
        },
    );
    crate::global_preferences_primitives::paint_scrollbar(scroll, quads);
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::{NewProjectDialogState, NewProjectUnitsSummaryRow};

    #[test]
    fn constrained_form_reveals_footer_and_returns_to_name_at_each_scale() {
        let mut dialog = NewProjectDialogState {
            open: true,
            units_summary: (0..8)
                .map(|n| NewProjectUnitsSummaryRow {
                    key: format!("unit{n}"),
                    label: format!("Unit {n}"),
                    value: "Metric".into(),
                })
                .collect(),
            ..Default::default()
        };
        for scale in [1.0, 1.5, 2.0] {
            dialog.focus = NewProjectFocus::ProjectName;
            let width = (960.0 * scale) as u32;
            let height = (400.0 * scale) as u32;
            let layout = PreparedScene::native_dialog_layout(width, height, scale);
            let legacy_layout = ShellLayout::for_surface(width, height, scale, None);
            let (mut legacy_quads, mut legacy_text, mut legacy_hits) =
                (Vec::new(), Vec::new(), Vec::new());
            render_new_project_dialog_scrolled(
                &dialog,
                &legacy_layout,
                true,
                &mut ControlMeshCache::default(),
                scale,
                &mut legacy_quads,
                &mut legacy_text,
                &mut legacy_hits,
                &mut ScrollViewport::default(),
                true,
            );
            let mut cache = ControlMeshCache::default();
            let mut scroll = ScrollViewport::default();
            let mut render =
                |dialog: &NewProjectDialogState, scroll: &mut ScrollViewport, reveal| {
                    let (mut q, mut t, mut h) = (Vec::new(), Vec::new(), Vec::new());
                    render_new_project_dialog_scrolled(
                        dialog, &layout, true, &mut cache, scale, &mut q, &mut t, &mut h, scroll,
                        reveal,
                    );
                    (q, t, h)
                };
            let first = render(&dialog, &mut scroll, true);
            assert_eq!(first, (legacy_quads, legacy_text, legacy_hits));
            assert!(scroll.maximum() > 0.0);
            assert!(
                !first
                    .2
                    .iter()
                    .any(|h| h.target == HitTarget::NewProjectCreate)
            );
            dialog.focus = NewProjectFocus::Create;
            let footer = render(&dialog, &mut scroll, true);
            assert!(scroll.offset() > 0.0);
            let create = footer
                .2
                .iter()
                .find(|h| h.target == HitTarget::NewProjectCreate)
                .unwrap()
                .rect;
            assert_eq!(create.height, 32.0 * scale);
            assert!(create.y >= 42.0 * scale && create.y + create.height <= 400.0 * scale);
            assert!(
                !footer
                    .2
                    .iter()
                    .any(|h| h.target == HitTarget::NewProjectName)
            );
            let header =
                |runs: &[TextRun]| runs.iter().find(|r| r.text == "New Project").unwrap().y;
            assert_eq!(header(&first.1), header(&footer.1));
            assert!(!scroll.wheel(0.0));
            let retained = scroll.offset();
            let _ = render(&dialog, &mut scroll, false);
            assert_eq!(scroll.offset(), retained);
            dialog.focus = NewProjectFocus::ProjectName;
            let top = render(&dialog, &mut scroll, true);
            let name = top
                .2
                .iter()
                .find(|h| h.target == HitTarget::NewProjectName)
                .unwrap()
                .rect;
            assert_eq!(name.height, 34.0 * scale);
            assert!(name.y >= 42.0 * scale);
            assert!(scroll.offset() < retained);
        }
    }
}
