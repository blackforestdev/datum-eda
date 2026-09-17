use super::*;
use datum_gui_protocol::{
    GlobalPreferenceControlUi, GlobalPreferenceRowUi, GlobalPreferencesFocus,
};

pub(super) fn state_with_preferences_open() -> datum_gui_protocol::ReviewWorkspaceState {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.global_preferences.open = true;
    state.ui.global_preferences.focus = GlobalPreferencesFocus::SectionNavigation;
    state.ui.global_preferences.rows = vec![
        row(
            "datum.console.feedback_duration",
            "Console feedback duration",
            GlobalPreferenceControlUi::SingleChoice {
                value: "6s".to_owned(),
                choices: vec![
                    ("4s".to_owned(), "4 s".to_owned()),
                    ("6s".to_owned(), "6 s".to_owned()),
                    ("10s".to_owned(), "10 s".to_owned()),
                    ("never".to_owned(), "Never hide".to_owned()),
                ],
            },
        ),
        row(
            "datum.accessibility.reduced_motion",
            "Reduced motion",
            GlobalPreferenceControlUi::Boolean {
                value: false,
                off_label: "Off".to_owned(),
                on_label: "On".to_owned(),
            },
        ),
        row(
            "datum.accessibility.high_contrast_noncolor",
            "High contrast and non-color cues",
            GlobalPreferenceControlUi::Boolean {
                value: false,
                off_label: "Off".to_owned(),
                on_label: "On".to_owned(),
            },
        ),
    ];
    state
}

fn row(key: &str, label: &str, control: GlobalPreferenceControlUi) -> GlobalPreferenceRowUi {
    let description = match key {
        "datum.console.feedback_duration" => {
            "How long Console feedback stays visible before auto-hiding."
        }
        "datum.accessibility.reduced_motion" => {
            "Disables non-essential interface animation while preserving non-color state cues."
        }
        "datum.accessibility.high_contrast_noncolor" => {
            "Uses high-contrast rendering with non-color state cues across GUI and terminal."
        }
        _ => unreachable!("fixture key is canonical"),
    };
    GlobalPreferenceRowUi {
        key: key.to_owned(),
        section_id: "appearance".to_owned(),
        label: label.to_owned(),
        description: description.to_owned(),
        aliases: Vec::new(),
        scope: "Global · this device".to_owned(),
        provenance: "Factory default · Global · this device".to_owned(),
        explanation_lines: vec![format!("Descriptor  {key}")],
        control,
        changed: false,
        writable: true,
        unavailable_reason: None,
        reset_description: "Removes the User contribution and resolves again.".to_owned(),
    }
}

fn prepared(
    state: &datum_gui_protocol::ReviewWorkspaceState,
    width: u32,
    height: u32,
) -> PreparedScene {
    let retained = RetainedScene::from_workspace(state, width, height);
    PreparedScene::from_workspace_with_terminal_renderer(
        state,
        width,
        height,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
        &[],
        None,
        true,
    )
}

#[test]
fn main_workspace_never_draws_the_native_preferences_window_or_backdrop() {
    let state = state_with_preferences_open();
    let retained = RetainedScene::from_workspace(&state, 1300, 760);
    let prepared = PreparedScene::from_workspace(
        &state,
        1300,
        760,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    assert!(!prepared.hit_regions.iter().any(|region| matches!(
        region.target,
        HitTarget::GlobalPreferencesModal
            | HitTarget::GlobalPreferencesSection(_)
            | HitTarget::GlobalPreferencesSearch
            | HitTarget::GlobalPreferencesSettingName(_)
            | HitTarget::GlobalPreferencesControl(_)
            | HitTarget::GlobalPreferencesChoice { .. }
            | HitTarget::GlobalPreferencesReset(_)
            | HitTarget::GlobalPreferencesExplanationClose
    )));
}

#[test]
fn dialog_renders_exact_three_canonical_controls_and_blocks_workspace_hits() {
    let state = state_with_preferences_open();
    let prepared = prepared(&state, 1300, 760);
    let controls: Vec<_> = prepared
        .hit_regions
        .iter()
        .filter_map(|region| match &region.target {
            HitTarget::GlobalPreferencesControl(key) => Some(key.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        controls,
        vec![
            "datum.console.feedback_duration",
            "datum.accessibility.reduced_motion",
            "datum.accessibility.high_contrast_noncolor",
        ]
    );
    assert_eq!(
        prepared.hit_test(2.0, 300.0),
        Some(&HitTarget::GlobalPreferencesModal)
    );
    let section = prepared
        .hit_regions
        .iter()
        .find(|region| {
            region.target == HitTarget::GlobalPreferencesSection("appearance".to_owned())
        })
        .expect("Appearance must have one row-sized navigation target");
    assert_eq!(section.rect.height, 32.0);
    assert!(section.rect.width < 210.0);
    assert_eq!(
        prepared.hit_test(section.rect.x + 20.0, section.rect.y + 16.0),
        Some(&HitTarget::GlobalPreferencesSection(
            "appearance".to_owned()
        ))
    );
    let labels: Vec<_> = prepared
        .menu_overlay_text_runs
        .iter()
        .map(|run| run.text.as_str())
        .collect();
    assert!(labels.contains(&"Global · this device"));
    assert!(labels.contains(&"saves immediately"));
    assert!(!labels.contains(&"Close"));
    assert!(!labels.contains(&"Apply"));
}

#[test]
fn focused_search_draws_one_text_caret_for_empty_and_nonempty_queries() {
    let mut unfocused_state = state_with_preferences_open();
    unfocused_state.ui.global_preferences.focus = GlobalPreferencesFocus::SectionNavigation;
    let unfocused = prepared(&unfocused_state, 1300, 760);

    let mut focused_empty_state = unfocused_state.clone();
    focused_empty_state.ui.global_preferences.focus = GlobalPreferencesFocus::Search;
    let focused_empty = prepared(&focused_empty_state, 1300, 760);
    let caret_vertices = |scene: &PreparedScene| {
        let search = scene
            .hit_regions
            .iter()
            .find(|region| region.target == HitTarget::GlobalPreferencesSearch)
            .unwrap()
            .rect;
        scene
            .menu_overlay_vertices()
            .iter()
            .filter(|vertex| {
                vertex.color == TEXT_PRIMARY
                    && vertex.pos[0] > search.x
                    && vertex.pos[0] < search.x + search.width
                    && vertex.pos[1] > search.y
                    && vertex.pos[1] < search.y + search.height
            })
            .count()
    };
    assert_eq!(caret_vertices(&unfocused), 0);
    assert_eq!(caret_vertices(&focused_empty), 6);

    focused_empty_state.ui.global_preferences.search_query = "units".to_owned();
    let mut unfocused_nonempty_state = focused_empty_state.clone();
    unfocused_nonempty_state.ui.global_preferences.focus =
        GlobalPreferencesFocus::SectionNavigation;
    let unfocused_nonempty = prepared(&unfocused_nonempty_state, 1300, 760);
    let focused_nonempty = prepared(&focused_empty_state, 1300, 760);
    assert_eq!(caret_vertices(&unfocused_nonempty), 0);
    assert_eq!(caret_vertices(&focused_nonempty), 6);
}

#[test]
fn search_field_uses_the_protected_six_pixel_rounded_outline() {
    let rect = RectPx {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 38.0,
    };
    let points =
        global_preferences_primitives::rounded_rect_points(rect, design_tokens::radius::MD);

    for square_corner in [
        (rect.x, rect.y),
        (rect.x + rect.width, rect.y),
        (rect.x + rect.width, rect.y + rect.height),
        (rect.x, rect.y + rect.height),
    ] {
        assert!(
            !points.iter().any(|point| {
                (point.0 - square_corner.0).abs() < 0.001
                    && (point.1 - square_corner.1).abs() < 0.001
            }),
            "rounded search geometry must not fill square corner {square_corner:?}"
        );
    }
    let radius = design_tokens::radius::MD;
    for edge_tangent in [
        (rect.x + radius, rect.y),
        (rect.x + rect.width - radius, rect.y),
        (rect.x + rect.width, rect.y + radius),
        (rect.x + rect.width, rect.y + rect.height - radius),
        (rect.x + rect.width - radius, rect.y + rect.height),
        (rect.x + radius, rect.y + rect.height),
        (rect.x, rect.y + rect.height - radius),
        (rect.x, rect.y + radius),
    ] {
        assert!(
            points.iter().any(|point| {
                (point.0 - edge_tangent.0).abs() < 0.001 && (point.1 - edge_tangent.1).abs() < 0.001
            }),
            "rounded search geometry must contain edge tangent {edge_tangent:?}"
        );
    }
}

#[test]
fn narrow_dialog_keeps_every_preference_hit_inside_the_window() {
    let state = state_with_preferences_open();
    let prepared = prepared(&state, 720, 760);
    for region in &prepared.hit_regions {
        if matches!(
            region.target,
            HitTarget::GlobalPreferencesModal
                | HitTarget::GlobalPreferencesSection(_)
                | HitTarget::GlobalPreferencesSearch
                | HitTarget::GlobalPreferencesSettingName(_)
                | HitTarget::GlobalPreferencesControl(_)
                | HitTarget::GlobalPreferencesChoice { .. }
                | HitTarget::GlobalPreferencesReset(_)
                | HitTarget::GlobalPreferencesExplanationClose
        ) {
            assert!(region.rect.x >= 0.0 && region.rect.y >= 0.0);
            assert!(region.rect.x + region.rect.width <= 720.0);
            assert!(region.rect.y + region.rect.height <= 760.0);
        }
    }
}

#[test]
fn canonical_descriptions_render_without_ellipsis() {
    let state = state_with_preferences_open();
    let prepared = prepared(&state, 700, 780);
    for description in state
        .ui
        .global_preferences
        .rows
        .iter()
        .map(|row| row.description.as_str())
    {
        assert!(
            prepared
                .menu_overlay_text_runs
                .iter()
                .any(|run| run.text == description),
            "canonical description was elided: {description}"
        );
    }
}

#[test]
fn preserved_unreadable_rows_keep_values_but_have_no_control_targets() {
    let mut state = state_with_preferences_open();
    state.ui.global_preferences.rows[0].changed = true;
    for row in &mut state.ui.global_preferences.rows {
        row.writable = false;
        row.provenance = "Preserved · Global · this device".to_owned();
    }
    state.ui.global_preferences.notice = Some(
        datum_gui_protocol::GlobalPreferencesNoticeUi::PreservedUnreadable(
            "Preferences could not be read; exact data remains preserved.".to_owned(),
        ),
    );
    let prepared = prepared(&state, 1300, 760);
    assert!(!prepared.hit_regions.iter().any(|region| matches!(
        region.target,
        HitTarget::GlobalPreferencesControl(_) | HitTarget::GlobalPreferencesReset(_)
    )));
    let labels: Vec<_> = prepared
        .menu_overlay_text_runs
        .iter()
        .map(|run| run.text.as_str())
        .collect();
    assert!(labels.contains(&"6 s · unavailable"));
    assert_eq!(
        labels
            .iter()
            .filter(|label| **label == "Off · unavailable")
            .count(),
        2
    );
    assert!(
        labels.contains(
            &"Preferences could not be read. Factory defaults are active for this session."
        )
    );
    assert!(labels.contains(&"Damaged data remains preserved; controls are unavailable."));
}

#[test]
fn reduced_motion_changes_no_information_or_keyboard_target() {
    let state = state_with_preferences_open();
    let ordinary = prepared(&state, 1300, 760);
    let mut reduced_state = state.clone();
    reduced_state.ui.global_preferences.reduced_motion = true;
    let reduced = prepared(&reduced_state, 1300, 760);
    assert_eq!(ordinary.hit_regions, reduced.hit_regions);
    assert_eq!(
        ordinary.menu_overlay_text_runs,
        reduced.menu_overlay_text_runs
    );
}

#[test]
fn high_contrast_mode_keeps_word_and_shape_state_cues() {
    let mut state = state_with_preferences_open();
    state.ui.global_preferences.high_contrast_noncolor = true;
    state.ui.global_preferences.rows[2].control = GlobalPreferenceControlUi::Boolean {
        value: true,
        off_label: "Off".to_owned(),
        on_label: "On".to_owned(),
    };
    let prepared = prepared(&state, 1300, 760);
    let labels: Vec<_> = prepared
        .menu_overlay_text_runs
        .iter()
        .map(|run| run.text.as_str())
        .collect();
    assert!(labels.contains(&"On"));
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text == "[HC] HIGH CONTRAST + NON-COLOR CUES")
    );
}
