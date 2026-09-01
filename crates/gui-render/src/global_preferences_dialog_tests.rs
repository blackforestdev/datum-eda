use super::*;
use datum_gui_protocol::{
    GlobalPreferenceControlUi, GlobalPreferenceRowUi, GlobalPreferencesFocus,
};

fn state_with_preferences_open() -> datum_gui_protocol::ReviewWorkspaceState {
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
        label: label.to_owned(),
        description: description.to_owned(),
        aliases: Vec::new(),
        scope: "Global · this device".to_owned(),
        provenance: "Factory default · Global · this device".to_owned(),
        explanation_lines: vec![format!("Descriptor  {key}")],
        control,
        changed: false,
        writable: true,
    }
}

fn prepared(
    state: &datum_gui_protocol::ReviewWorkspaceState,
    width: u32,
    height: u32,
) -> PreparedScene {
    let retained = RetainedScene::from_workspace(state, width, height);
    PreparedScene::from_workspace(
        state,
        width,
        height,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
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
}

#[test]
fn narrow_dialog_keeps_every_preference_hit_inside_the_window() {
    let state = state_with_preferences_open();
    let prepared = prepared(&state, 720, 760);
    for region in &prepared.hit_regions {
        if matches!(
            region.target,
            HitTarget::GlobalPreferencesModal
                | HitTarget::GlobalPreferencesSection
                | HitTarget::GlobalPreferencesSearch
                | HitTarget::GlobalPreferencesSettingName(_)
                | HitTarget::GlobalPreferencesControl(_)
                | HitTarget::GlobalPreferencesChoice { .. }
                | HitTarget::GlobalPreferencesReset(_)
                | HitTarget::GlobalPreferencesExplanationClose
                | HitTarget::GlobalPreferencesClose
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
    assert!(
        !prepared
            .hit_regions
            .iter()
            .any(|region| matches!(region.target, HitTarget::GlobalPreferencesControl(_)))
    );
    let labels: Vec<_> = prepared
        .menu_overlay_text_runs
        .iter()
        .map(|run| run.text.as_str())
        .collect();
    assert!(labels.contains(&"6 s  v -- unavailable"));
    assert_eq!(
        labels
            .iter()
            .filter(|label| **label == "[ ]  Off -- unavailable")
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
    assert!(labels.contains(&"[x]  On"));
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text == "[HC] HIGH CONTRAST + NON-COLOR CUES")
    );
}
