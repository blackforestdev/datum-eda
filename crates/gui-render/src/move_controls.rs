//! Captured Move prompt uses common controls, independent of the target editor.
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter, button};
use crate::*;
use datum_gui_protocol::ReviewWorkspaceState;
use eda_engine::{
    api::native_write::project::project_display_units,
    ir::units::{LengthEditSession, Nanometers},
};

pub(crate) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    let gesture = &state.authoring.gesture;
    if gesture.tool != datum_gui_protocol::WorkspaceTool::Move || !gesture.is_active() {
        return Ok(());
    }
    let Some(basis) = &gesture.view_basis else {
        return Ok(());
    };
    let panes = layout.viewport_panes(&state.ui.layout);
    let Some(pane) = panes.panes.iter().find(|pane| pane.id == basis.pane) else {
        return Ok(());
    };
    let viewport = pane.rect.scene;
    let width = 460f32.min((viewport.width - 16.).max(0.));
    if width < 80. || viewport.height < 16. {
        return Ok(());
    }
    let panel = RectPx {
        x: viewport.x + (viewport.width - width) / 2.,
        y: viewport.y + 8.,
        width,
        height: 86.,
    };
    let starts = (quads.len(), text.len(), hits.len());
    let owner_available = state.ui.active_menu.is_none()
        && !state.ui.new_project.open
        && !state.ui.project_open.open
        && state.library.acquisition.is_none();
    if owner_available {
        hits.push(HitRegion {
            target: HitTarget::SchematicPicker,
            rect: panel,
        });
    }

    quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
    let target = gesture.target_object_id.as_deref().unwrap_or("");
    let group = target.strip_prefix("schematic-symbols:");
    let title = group
        .map(|count| format!("Move {count} symbols · R: rotate · X/Y: flip"))
        .unwrap_or_else(|| {
            if target.starts_with("schematic-wiring-") {
                "Move wiring · R: rotate · X/Y: flip".into()
            } else if target.starts_with("schematic-text:") {
                "Move Text · R: rotate".into()
            } else if target.starts_with("schematic-symbol:") {
                "Move symbol · R: rotate · X/Y: flip horiz/vert".into()
            } else {
                "Move".into()
            }
        });
    draw_text_clipped(
        &title,
        panel.x + 10.,
        panel.y + 10.,
        12.,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        panel,
        text,
    );
    let hint = RectPx {
        x: panel.x + 10.,
        y: panel.y + 36.,
        width: (width - 150.).max(1.),
        height: 42.,
    };
    draw_text_clipped(
        if group.is_some() {
            "Cross: pickup reference · Click apply · Esc cancel"
        } else {
            "Click: apply · Esc: cancel"
        },
        hint.x,
        hint.y,
        11.,
        TEXT_MUTED,
        TextFace::Ui,
        hint,
        text,
    );
    let enabled = owner_available && state.ui.layout.focused == basis.pane;
    if enabled && state.ui.focus == datum_gui_protocol::ApplicationFocus::Editor(basis.pane) {
        draw_text_clipped(
            &format!(
                "Canvas arrows: grid step {}",
                grid_step_token(state, basis)?
            ),
            hint.x,
            hint.y + 20.,
            11.,
            TEXT_MUTED,
            TextFace::Ui,
            hint,
            text,
        );
    }
    let button_width = 122f32.min(width - 24.);
    let rect = RectPx {
        x: panel.x + width - button_width - 12.,
        y: panel.y + 42.,
        width: button_width,
        height: 29.,
    };
    let mut cache = ControlMeshCache::default();
    button(
        "Coordinates…",
        rect,
        false,
        enabled,
        &mut ControlPainter::new(quads, &mut cache, 1.),
        text,
    );
    if enabled {
        hits.push(HitRegion {
            target: HitTarget::MoveCoordinates,
            rect,
        });
    }
    hit_clipping::clip_content(quads, text, hits, starts.0, starts.1, starts.2, viewport);
    Ok(())
}

fn grid_step_token(
    state: &ReviewWorkspaceState,
    basis: &datum_gui_protocol::AuthoringViewBasis,
) -> anyhow::Result<String> {
    let step = state.authoring.snap.grid_nm;
    let Some(model) = state
        .library
        .basis
        .as_ref()
        .filter(|model| model.project.project_id == basis.project)
    else {
        // Synthetic/render-only state has no captured Project unit authority.
        return Ok(format!("{step}nm"));
    };
    let profile = project_display_units(model)?
        .resolve()
        .map_err(|error| anyhow::anyhow!("Project units unavailable: {error:?}"))?;
    let unit = match basis.content {
        datum_gui_protocol::PaneContent::Board => profile.board.unit,
        datum_gui_protocol::PaneContent::Schematic => profile.schematic.unit,
        _ => return Ok(format!("{step}nm")),
    };
    Ok(
        LengthEditSession::begin(Nanometers::new(step), unit, profile.system)
            .buffer()
            .to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captured_move_offers_coordinates_and_canvas_only_nudge_guidance() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.library.basis = None;
        state.authoring.snap.grid_nm = 100_000;
        let layout = ShellLayout::for_window(1344, 840, None);
        let panes = layout.viewport_panes(&state.ui.layout);
        let pane = panes
            .panes
            .iter()
            .find(|pane| pane.content == datum_gui_protocol::PaneContent::Schematic)
            .unwrap();
        state.ui.layout.focused = pane.id;
        state.ui.focus = datum_gui_protocol::ApplicationFocus::Editor(pane.id);
        state.authoring.gesture.tool = datum_gui_protocol::WorkspaceTool::Move;
        state.authoring.gesture.target_object_id = Some("schematic-wiring-object:fixture".into());
        state.authoring.gesture.view_basis = Some(datum_gui_protocol::AuthoringViewBasis {
            project: uuid::Uuid::nil(),
            revision: "render-test".into(),
            pane: pane.id,
            content: pane.content,
            source: uuid::Uuid::nil(),
            document: None,
        });
        let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
        render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
        assert!(
            hits.iter()
                .any(|hit| hit.target == HitTarget::MoveCoordinates)
        );
        assert!(text.iter().any(|run| run.text == "Coordinates…"));
        let body = hits
            .iter()
            .position(|hit| hit.target == HitTarget::SchematicPicker)
            .unwrap();
        let opener = hits
            .iter()
            .position(|hit| hit.target == HitTarget::MoveCoordinates)
            .unwrap();
        assert!(
            body < opener,
            "opaque backdrop consumes canvas clicks without hiding the opener"
        );

        assert!(
            text.iter()
                .any(|run| run.text == "Canvas arrows: grid step 100000nm")
        );
        state.authoring.snap.grid_nm = 25_401;
        quads.clear();
        text.clear();
        hits.clear();
        render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
        assert!(
            text.iter()
                .any(|run| run.text == "Canvas arrows: grid step 25401nm")
        );

        for owner in 0..4 {
            state.ui.active_menu = (owner == 0).then(|| "View".into());
            state.ui.new_project.open = owner == 1;
            state.ui.project_open.open = owner == 2;
            state.library.acquisition = (owner == 3).then(|| {
                datum_gui_protocol::library_browser::acquisition::LibraryAcquisition::begin(
                    "render-test".into(),
                    None,
                )
            });
            quads.clear();
            text.clear();
            hits.clear();
            render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
            assert!(
                hits.is_empty(),
                "menu/modal owner {owner} retains prompt backdrop and button input"
            );
            assert!(text.iter().any(|run| run.text == "Coordinates…"));
        }
        state.ui.active_menu = None;
        state.ui.new_project.open = false;
        state.ui.project_open.open = false;
        state.library.acquisition = None;
        state.ui.focus = datum_gui_protocol::ApplicationFocus::Terminal;
        quads.clear();
        text.clear();
        hits.clear();
        render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
        assert!(
            !text
                .iter()
                .any(|run| run.text.starts_with("Canvas arrows: grid step"))
        );
        state.authoring.gesture.view_basis = None;
        quads.clear();
        text.clear();
        hits.clear();
        render(&state, &layout, &mut quads, &mut text, &mut hits).unwrap();
        assert!(hits.is_empty(), "no unqualified opener fallback");
    }
}
