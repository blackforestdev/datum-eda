use super::*;

#[test]
fn preference_notice_priority_distinguishes_status_from_refusal_and_recovery() {
    let (_, polite) = global_preferences_notice_announcement(GlobalPreferencesNoticeUi::Polite(
        "Saved".to_owned(),
    ));
    let (_, refusal) = global_preferences_notice_announcement(
        GlobalPreferencesNoticeUi::Assertive("Not saved".to_owned()),
    );
    let (_, recovery) = global_preferences_notice_announcement(
        GlobalPreferencesNoticeUi::PreservedUnreadable("Preserved".to_owned()),
    );
    assert_eq!(polite, AnnouncementPriority::Medium);
    assert_eq!(refusal, AnnouncementPriority::High);
    assert_eq!(recovery, AnnouncementPriority::High);
}

#[test]
fn projection_has_exact_three_rows_and_all_consumers_are_total() {
    let base = std::env::temp_dir().join(format!("datum-gp-ui-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let mut coordinator = GlobalPreferencesCoordinator {
        service: GlobalPreferencesService::open(
            base.join("repository"),
            &base.join("legacy.json"),
            "gui-test-writer",
            SCOPE,
        )
        .unwrap(),
        return_focus: ApplicationFocus::default(),
        terminal_theme_before_high_contrast: None,
    };
    let filters = datum_gui_protocol::WorkspaceFilterState {
        show_authored: true,
        show_proposed: true,
        show_unrouted: true,
        dim_unrelated: false,
        active_layer_id: None,
        layer_visibility: Default::default(),
        layer_scroll_offset: 0,
    };
    let mut ui = WorkspaceUiState::new(filters);
    let resolved_rows = coordinator.service.rows();
    coordinator.publish_projection(&mut ui);
    assert_eq!(ui.global_preferences.rows.len(), 3);
    assert_eq!(
        ui.global_preferences.rows[0].key,
        "datum.console.feedback_duration"
    );
    assert!(!ui.global_preferences.reduced_motion);
    assert!(!ui.global_preferences.high_contrast_noncolor);
    ui.global_preferences.open = true;
    let nodes = ui.global_preferences.accessibility_nodes();
    assert_eq!(nodes[0].name, "Global Preferences");
    assert_eq!(
        nodes
            .iter()
            .filter(|node| matches!(
                node.role,
                datum_gui_protocol::GlobalPreferencesAccessibleRole::Switch
                    | datum_gui_protocol::GlobalPreferencesAccessibleRole::ComboBox
            ))
            .count(),
        3
    );
    assert!(nodes.iter().all(|node| {
        !matches!(
            node.role,
            datum_gui_protocol::GlobalPreferencesAccessibleRole::Switch
                | datum_gui_protocol::GlobalPreferencesAccessibleRole::ComboBox
        ) || node.description.contains("Global · this device")
    }));
    for (projected, resolved) in ui.global_preferences.rows.iter().zip(&resolved_rows) {
        assert_eq!(projected.key, resolved.key.as_str());
        assert_eq!(projected.scope, resolved.explanation.machine_scope);
        assert!(
            projected
                .explanation_lines
                .iter()
                .any(|line| line.contains(resolved.key.as_str()))
        );
        for absence in &resolved.explanation.absent_sources {
            assert!(
                projected
                    .explanation_lines
                    .iter()
                    .any(|line| line.contains(&format!("{absence:?}")))
            );
        }
    }
    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn edits_and_resets_cannot_mutate_project_shards_or_journal() {
    let base = std::env::temp_dir().join(format!("datum-gp-zero-project-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let project = base.join("project");
    std::fs::create_dir_all(&project).unwrap();
    let shard = project.join("board.json");
    let journal = project.join("journal.jsonl");
    std::fs::write(&shard, b"project-shard-sentinel").unwrap();
    std::fs::write(&journal, b"journal-sentinel\n").unwrap();
    let before = (
        std::fs::read(&shard).unwrap(),
        std::fs::read(&journal).unwrap(),
    );
    let before_hash = project_tree_hash(&project);

    let mut coordinator = GlobalPreferencesCoordinator {
        service: GlobalPreferencesService::open(
            base.join("repository"),
            &base.join("legacy.json"),
            "zero-project-writer",
            SCOPE,
        )
        .unwrap(),
        return_focus: ApplicationFocus::default(),
        terminal_theme_before_high_contrast: None,
    };
    let filters = datum_gui_protocol::WorkspaceFilterState {
        show_authored: true,
        show_proposed: true,
        show_unrouted: true,
        dim_unrelated: false,
        active_layer_id: None,
        layer_visibility: Default::default(),
        layer_scroll_offset: 0,
    };
    let mut ui = WorkspaceUiState::new(filters);
    coordinator.publish_projection(&mut ui);
    coordinator
        .set_value(
            "datum.accessibility.reduced_motion",
            serde_json::Value::Bool(true),
            &mut ui,
        )
        .unwrap();
    coordinator
        .reset("datum.accessibility.reduced_motion", &mut ui)
        .unwrap();
    let invoker = ApplicationFocus::Editor(ui.layout.focused);
    assert!(coordinator.open_dialog(&mut ui, invoker));
    ui.global_preferences.search_query = "console_duration".to_owned();
    assert_eq!(ui.global_preferences.visible_rows().count(), 1);
    ui.global_preferences.explanation_key = Some("datum.console.feedback_duration".to_owned());
    assert!(
        ui.global_preferences
            .accessibility_nodes()
            .iter()
            .any(|node| node.id.ends_with("-explanation"))
    );
    let invalid = coordinator.set_value(
        "datum.accessibility.reduced_motion",
        serde_json::Value::String("invalid-draft".to_owned()),
        &mut ui,
    );
    assert!(invalid.is_err());
    assert!(coordinator.close_dialog(&mut ui).is_some());
    drop(coordinator);

    let mut restarted = GlobalPreferencesCoordinator {
        service: GlobalPreferencesService::open(
            base.join("repository"),
            &base.join("legacy.json"),
            "zero-project-restart-writer",
            SCOPE,
        )
        .unwrap(),
        return_focus: ApplicationFocus::default(),
        terminal_theme_before_high_contrast: None,
    };
    restarted.publish_projection(&mut ui);
    std::fs::write(base.join("repository/head.json"), b"{corrupt-head").unwrap();
    drop(restarted);
    let mut corrupt = GlobalPreferencesCoordinator {
        service: GlobalPreferencesService::open(
            base.join("repository"),
            &base.join("legacy.json"),
            "zero-project-corrupt-writer",
            SCOPE,
        )
        .unwrap(),
        return_focus: ApplicationFocus::default(),
        terminal_theme_before_high_contrast: None,
    };
    corrupt.publish_projection(&mut ui);
    assert!(ui.global_preferences.rows.iter().all(|row| !row.writable));

    assert_eq!(std::fs::read(&shard).unwrap(), before.0);
    assert_eq!(std::fs::read(&journal).unwrap(), before.1);
    assert_eq!(project_tree_hash(&project), before_hash);
    assert_eq!(std::fs::read_dir(&project).unwrap().count(), 2);
    let _ = std::fs::remove_dir_all(base);
}

fn project_tree_hash(project: &std::path::Path) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut entries: Vec<_> = std::fs::read_dir(project)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for path in entries {
        path.file_name().unwrap().hash(&mut hasher);
        std::fs::read(path).unwrap().hash(&mut hasher);
    }
    hasher.finish()
}

#[test]
fn every_visible_value_updates_its_declared_live_consumer() {
    let base = std::env::temp_dir().join(format!("datum-gp-consumers-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let mut coordinator = GlobalPreferencesCoordinator {
        service: GlobalPreferencesService::open(
            base.join("repository"),
            &base.join("legacy.json"),
            "consumer-test-writer",
            SCOPE,
        )
        .unwrap(),
        return_focus: ApplicationFocus::default(),
        terminal_theme_before_high_contrast: None,
    };
    let filters = datum_gui_protocol::WorkspaceFilterState {
        show_authored: true,
        show_proposed: true,
        show_unrouted: true,
        dim_unrelated: false,
        active_layer_id: None,
        layer_visibility: Default::default(),
        layer_scroll_offset: 0,
    };
    let mut ui = WorkspaceUiState::new(filters);
    coordinator.publish_projection(&mut ui);
    let original_terminal_theme = ui.terminal.theme;

    coordinator
        .set_value(
            "datum.console.feedback_duration",
            serde_json::Value::String("never".to_owned()),
            &mut ui,
        )
        .unwrap();
    assert_eq!(
        ui.console.duration_preference(),
        datum_gui_protocol::ConsoleFeedbackDuration::Never
    );

    coordinator
        .set_value(
            "datum.accessibility.reduced_motion",
            serde_json::Value::Bool(true),
            &mut ui,
        )
        .unwrap();
    assert!(ui.global_preferences.reduced_motion);

    coordinator
        .set_value(
            "datum.accessibility.high_contrast_noncolor",
            serde_json::Value::Bool(true),
            &mut ui,
        )
        .unwrap();
    assert!(ui.global_preferences.high_contrast_noncolor);
    assert_eq!(
        ui.terminal.theme,
        datum_gui_protocol::TerminalTheme::HighContrast
    );

    coordinator
        .reset("datum.accessibility.high_contrast_noncolor", &mut ui)
        .unwrap();
    assert!(!ui.global_preferences.high_contrast_noncolor);
    assert_eq!(ui.terminal.theme, original_terminal_theme);
    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn keyboard_focus_has_no_trap_and_escape_closes_innermost_first() {
    let base = std::env::temp_dir().join(format!("datum-gp-keyboard-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let mut coordinator = GlobalPreferencesCoordinator {
        service: GlobalPreferencesService::open(
            base.join("repository"),
            &base.join("legacy.json"),
            "keyboard-test-writer",
            SCOPE,
        )
        .unwrap(),
        return_focus: ApplicationFocus::default(),
        terminal_theme_before_high_contrast: None,
    };
    let filters = datum_gui_protocol::WorkspaceFilterState {
        show_authored: true,
        show_proposed: true,
        show_unrouted: true,
        dim_unrelated: false,
        active_layer_id: None,
        layer_visibility: Default::default(),
        layer_scroll_offset: 0,
    };
    let mut ui = WorkspaceUiState::new(filters);
    coordinator.publish_projection(&mut ui);
    ui.global_preferences.open = true;
    let first = ui.global_preferences.focus.clone();
    let mut visited = std::collections::BTreeSet::new();
    for _ in 0..8 {
        visited.insert(format!("{:?}", ui.global_preferences.focus));
        ui.global_preferences.advance_focus(false);
    }
    assert_eq!(ui.global_preferences.focus, first);
    assert_eq!(visited.len(), 8);
    ui.global_preferences.advance_focus(true);
    assert_eq!(
        ui.global_preferences.focus,
        datum_gui_protocol::GlobalPreferencesFocus::Control(
            "datum.accessibility.high_contrast_noncolor".to_owned()
        )
    );

    let key = "datum.console.feedback_duration".to_owned();
    ui.global_preferences.explanation_key = Some(key.clone());
    ui.global_preferences.open_choice_key = Some(key);
    assert_eq!(
        ui.global_preferences.dismiss_innermost(),
        datum_gui_protocol::GlobalPreferencesDismissal::ChoiceClosed
    );
    assert!(ui.global_preferences.explanation_key.is_some());
    assert_eq!(
        ui.global_preferences.dismiss_innermost(),
        datum_gui_protocol::GlobalPreferencesDismissal::ExplanationClosed
    );
    assert!(ui.global_preferences.open);
    assert_eq!(
        ui.global_preferences.dismiss_innermost(),
        datum_gui_protocol::GlobalPreferencesDismissal::DialogClosed
    );
    assert!(!ui.global_preferences.open);
    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn explanation_accessibility_is_complete_and_in_reading_order() {
    let mut ui = projected_default_ui("accessibility-explanation");
    ui.global_preferences.open = true;
    let row = &ui.global_preferences.rows[0];
    let key = row.key.clone();
    let expected = row.explanation_lines.join(". ");
    ui.global_preferences.explanation_key = Some(key.clone());
    let nodes = ui.global_preferences.accessibility_nodes();
    let explanation = nodes
        .iter()
        .find(|node| node.id == format!("{key}-explanation"))
        .unwrap();
    assert_eq!(explanation.value.as_deref(), Some(expected.as_str()));
    assert!(
        nodes
            .iter()
            .any(|node| node.id == format!("{key}-explanation-close"))
    );
}

#[test]
fn dialog_accessibility_states_that_changes_save_immediately() {
    let mut ui = projected_default_ui("accessibility-immediate-save");
    ui.global_preferences.open = true;
    let dialog = ui
        .global_preferences
        .accessibility_nodes()
        .into_iter()
        .find(|node| node.id == "global-preferences")
        .unwrap();
    assert!(dialog.description.contains("Changes save immediately"));
}

fn projected_default_ui(name: &str) -> WorkspaceUiState {
    let base =
        std::env::temp_dir().join(format!("datum-gp-projection-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let mut coordinator = GlobalPreferencesCoordinator {
        service: GlobalPreferencesService::open(
            base.join("repository"),
            &base.join("legacy.json"),
            "projection-test-writer",
            SCOPE,
        )
        .unwrap(),
        return_focus: ApplicationFocus::default(),
        terminal_theme_before_high_contrast: None,
    };
    let mut ui = WorkspaceUiState::new(datum_gui_protocol::WorkspaceFilterState {
        show_authored: true,
        show_proposed: true,
        show_unrouted: true,
        dim_unrelated: false,
        active_layer_id: None,
        layer_visibility: Default::default(),
        layer_scroll_offset: 0,
    });
    coordinator.publish_projection(&mut ui);
    let _ = std::fs::remove_dir_all(base);
    ui
}

#[test]
fn dialog_is_unique_and_restores_the_original_invoker_focus() {
    let base = std::env::temp_dir().join(format!("datum-gp-unique-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let mut coordinator = GlobalPreferencesCoordinator {
        service: GlobalPreferencesService::open(
            base.join("repository"),
            &base.join("legacy.json"),
            "unique-dialog-test-writer",
            SCOPE,
        )
        .unwrap(),
        return_focus: ApplicationFocus::default(),
        terminal_theme_before_high_contrast: None,
    };
    let mut ui = projected_default_ui("unique-dialog-state");
    let invoker = ApplicationFocus::Editor(ui.layout.focused);
    assert!(coordinator.open_dialog(&mut ui, invoker));
    assert!(ui.global_preferences.open);
    assert_eq!(ui.focus, ApplicationFocus::Overlay);
    assert!(!coordinator.open_dialog(&mut ui, ApplicationFocus::Terminal));
    assert_eq!(coordinator.close_dialog(&mut ui), Some(invoker));
    assert_eq!(coordinator.close_dialog(&mut ui), None);
    let _ = std::fs::remove_dir_all(base);
}
