//! Menu-overlay contract tests, co-located with the menu chrome they lock
//! (extracted from `render_contract_tests.rs` under source-size governance /
//! decision 022): dropdown compositing into the dedicated overlay sink,
//! title-aligned drop position, content-fitted card width, and the closed-menu
//! parity guarantee.

use crate::{
    CameraState, HitTarget, PreparedScene, RetainedScene, measured_text_run_width_px,
};

#[test]
fn menu_dropdown_composites_into_menu_overlay_under_its_title() {
    // Bugs B + C: when a menu is open, its dropdown body is emitted into the
    // dedicated menu-overlay sink (composited AFTER the viewport passes), NOT into
    // panel_vertices; and it drops directly under its own title (left-aligned to
    // the active title's rect.x), not from a fixed far-left offset.
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let model =
        datum_gui_protocol::load_default_gui_menu_model().expect("default menu model should load");
    // Use the FIRST (leftmost) title so the dropdown is not right-edge clamped.
    let active_title = model.menubar[0].menu.clone();
    state.ui.active_menu = Some(active_title.clone());

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    // Bug B: overlay sink is non-empty when a menu is active.
    let overlay = prepared.menu_overlay_vertices();
    assert!(
        !overlay.is_empty(),
        "menu overlay must carry the dropdown body when a menu is active"
    );

    // Bug C: the dropdown's left edge equals the active title's rect.x.
    let title_x = prepared
        .hit_regions
        .iter()
        .find_map(|region| match &region.target {
            HitTarget::MenuTitle(name) if *name == active_title => Some(region.rect.x),
            _ => None,
        })
        .expect("active menu title must have a hit region");
    let dropdown_left = overlay
        .iter()
        .map(|v| v.pos[0])
        .fold(f32::INFINITY, f32::min);
    assert!(
        (dropdown_left - title_x).abs() < 0.5,
        "dropdown left {dropdown_left:.2} must align under its title x {title_x:.2}"
    );

    // Bug B (text occlusion): the dropdown's OWN item labels must live in the
    // dedicated menu-overlay text sink (drawn LAST, on top of the card) and NOT
    // in the main text_runs (drawn before the card). If any item label appeared
    // in the main pass it would either be occluded by the card or, worse, other
    // main-pass text would bleed over the card. Locking the split guarantees the
    // card fully occludes the bleed while its own labels stay crisp.
    let active_menu = model
        .menubar
        .iter()
        .find(|m| m.menu == active_title)
        .expect("active menu exists in model");
    let overlay_labels: Vec<&str> = prepared
        .menu_overlay_text_runs()
        .iter()
        .map(|run| run.text.as_str())
        .collect();
    let main_labels: Vec<&str> = prepared
        .text_runs
        .iter()
        .map(|run| run.text.as_str())
        .collect();
    assert!(
        !prepared.menu_overlay_text_runs().is_empty(),
        "menu overlay text sink must carry the dropdown labels when a menu is open"
    );
    for item in &active_menu.items {
        assert!(
            overlay_labels.contains(&item.label.as_str()),
            "dropdown item '{}' must render in the menu-overlay text pass",
            item.label
        );
        assert!(
            !main_labels.contains(&item.label.as_str()),
            "dropdown item '{}' must NOT be in the main text pass (would bleed/occlude)",
            item.label
        );
    }
    // The menu-bar TITLE itself stays in the main pass (it lives in the bar and is
    // never occluded).
    assert!(
        main_labels.contains(&active_title.as_str()),
        "menu-bar title '{active_title}' must remain in the main text pass"
    );
}

#[test]
fn menu_overlay_is_empty_when_no_menu_open() {
    // Parity safety: default boot (no menu open) emits no overlay quads.
    let state = datum_gui_protocol::load_fixture_workspace_state();
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    assert!(
        prepared.menu_overlay_vertices().is_empty(),
        "no menu open -> no menu-overlay quads (default parity capture untouched)"
    );
}

#[test]
fn menu_dropdown_fits_its_content_no_spill() {
    // The dropdown card width scales to its content: every label and shortcut must
    // stay within the card's right edge. The retired fixed-width card (272px + a
    // fixed 74px shortcut reservation) clipped long labels and wide shortcuts like
    // "Ctrl+Shift+S".
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let model =
        datum_gui_protocol::load_default_gui_menu_model().expect("default menu model should load");
    // File has the widest shortcuts (Ctrl+Shift+…); fall back to the first menu.
    let menu_name = model
        .menubar
        .iter()
        .find(|m| m.menu == "File")
        .map(|m| m.menu.clone())
        .unwrap_or_else(|| model.menubar[0].menu.clone());
    state.ui.active_menu = Some(menu_name);

    let retained = RetainedScene::from_workspace(&state, 1680, 1050);
    let prepared = PreparedScene::from_workspace(
        &state,
        1680,
        1050,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let card_right = prepared
        .menu_overlay_vertices()
        .iter()
        .map(|v| v.pos[0])
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(card_right.is_finite(), "an open menu must emit an overlay card");
    for run in prepared.menu_overlay_text_runs() {
        let right_edge = run.x + measured_text_run_width_px(&run.text, run.size, run.face);
        assert!(
            right_edge <= card_right + 0.5,
            "menu text '{}' right edge {right_edge:.1} spills past the card right {card_right:.1}",
            run.text
        );
    }
}

