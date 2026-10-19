use super::*;

#[test]
fn shared_manifest_activation_requires_exact_regular_entry_in_ready_open_without_inner_action() {
    let entry = BrowserEntry {
        name: "project.json".into(),
        kind: BrowserEntryKind::File,
        identity: Some((1, 2)),
    };
    assert!(entry.is_project_manifest() && !entry.navigable());
    let mut state = ready(FolderBrowserMode::Open);
    state.entries = vec![entry.clone()];
    assert!(
        !state.opens_selected_manifest(),
        "passive listing must not activate"
    );
    state.selected = Some(0);
    assert!(state.opens_selected_manifest());
    let nodes = state.accessibility_nodes(Some(FolderBrowserFocus::Folders), None, true);
    let row = nodes.iter().find(|n| n.name == "project.json").unwrap();
    assert!(row.value.as_deref().unwrap().starts_with("file,"));
    assert!(
        row.description
            .contains("Enter opens this folder’s Project")
    );
    assert_eq!(state.destination(None).as_deref(), Some("/parent"));
    for kind in [
        BrowserEntryKind::Directory,
        BrowserEntryKind::Link { directory: false },
        BrowserEntryKind::Link { directory: true },
        BrowserEntryKind::Other,
    ] {
        state.entries[0].kind = kind;
        assert!(!state.opens_selected_manifest());
    }
    state.entries[0] = entry.clone();
    for name in ["Project.json", "other.json", "project.json.backup"] {
        state.entries[0].name = name.into();
        assert!(!state.opens_selected_manifest());
    }
    state.entries[0] = entry;
    state.mode = FolderBrowserMode::New;
    assert!(!state.opens_selected_manifest());
    state.mode = FolderBrowserMode::Open;
    state.loading = true;
    assert!(!state.opens_selected_manifest());
    state.loading = false;
    state.path_input = "/different".into();
    assert!(!state.opens_selected_manifest());
    state.path_input = state.directory.clone();
    state.expanded = false;
    assert!(!state.opens_selected_manifest());
    state.expanded = true;
    state.naming = Some(NamingDraft::new(state.directory.clone(), (1, 1), 1, None));
    assert!(!state.opens_selected_manifest());
    state.naming = None;
    state.menu = Some(crate::popup_menu::PopupMenu::new((0.0, 0.0), vec![]));
    assert!(!state.opens_selected_manifest());
}
fn ready(mode: FolderBrowserMode) -> FolderBrowserState {
    let mut state = FolderBrowserState::new(mode, "/parent".into());
    state.directory = "/parent".into();
    state.expanded = true;
    state.loading = false;
    state
}
#[test]
fn inline_new_uses_current_parent_and_exact_leaf_not_highlight() {
    let mut state = ready(FolderBrowserMode::New);
    state.entries = vec!["different parent".into()];
    state.selected = Some(0);
    assert_eq!(
        state.destination(Some(" exact λ ")).as_deref(),
        Some("/parent/ exact λ ")
    );
    for invalid in ["", ".", "..", "a/b", "/absolute", "a\0b"] {
        assert!(state.destination(Some(invalid)).is_none());
        if !invalid.is_empty() {
            assert!(
                state
                    .refusal(Some(invalid))
                    .unwrap()
                    .contains("override Folder name")
            );
        }
    }
    state.occupied.insert("occupied".into());
    assert!(state.destination(Some("occupied")).is_none());
    assert_eq!(
        state.preview(Some("occupied")).as_deref(),
        Some("/parent/occupied")
    );
    state.loading = true;
    assert_eq!(
        state.destination(Some("new")).as_deref(),
        Some("/parent/new")
    );
    state.loading = false;
    state.path_input = "/unread parent".into();
    assert_eq!(
        state.destination(Some("new")).as_deref(),
        Some("/unread parent/new")
    );
}
#[test]
fn inline_open_uses_highlight_or_current_with_visible_accessible_rows() {
    let mut state = ready(FolderBrowserMode::Open);
    state.entries = vec!["a".into(), "b λ ".into()];
    assert_eq!(state.destination(None).as_deref(), Some("/parent"));
    state.move_selection(1);
    state.move_selection(1);
    assert_eq!(state.destination(None).as_deref(), Some("/parent/b λ "));
    state.visible_rows = vec![0];
    let nodes = state.accessibility_nodes(Some(FolderBrowserFocus::Folders), None, true);
    for name in ["a", "b λ "] {
        assert!(nodes.iter().any(|n| n.name == name));
    }
    assert!(
        !nodes
            .iter()
            .any(|n| n.role == crate::GlobalPreferencesAccessibleRole::Dialog)
    );
    assert!(
        !nodes
            .iter()
            .any(|n| matches!(n.name.as_str(), "Select" | "Use Location" | "Cancel"))
    );
    let disabled = state.accessibility_nodes(None, None, false);
    assert!(disabled.iter().all(|n| !n.available));
}
#[test]
fn inline_new_name_default_override_and_restore_are_explicit() {
    use crate::{NewProjectDialogState, NewProjectFocus};
    let mut state = NewProjectDialogState::default();
    assert!(state.use_project_name);
    state.project_name = " initial λ ".into();
    assert_eq!(state.effective_folder_name(), " initial λ ");
    state.set_use_project_name(false);
    assert_eq!(state.folder_name, " initial λ ");
    state.folder_name = "custom ".into();
    state.project_name = "renamed".into();
    assert_eq!(state.effective_folder_name(), "custom ");
    assert!(
        state
            .focus_order(false)
            .contains(&NewProjectFocus::FolderName)
    );
    state.set_use_project_name(true);
    assert_eq!(state.effective_folder_name(), "renamed");
    assert!(
        !state
            .focus_order(false)
            .contains(&NewProjectFocus::FolderName)
    );
    state.project_name = "invalid/name".into();
    assert!(
        ready(FolderBrowserMode::New)
            .destination(Some(state.effective_folder_name()))
            .is_none()
    );
    state.set_use_project_name(false);
    state.folder_name = "correct override".into();
    assert_eq!(
        ready(FolderBrowserMode::New)
            .destination(Some(state.effective_folder_name()))
            .as_deref(),
        Some("/parent/correct override")
    );
}
#[test]
fn inline_large_list_exposes_visible_rows_plus_offscreen_selection() {
    let mut state = ready(FolderBrowserMode::Open);
    state.entries = (0..10000)
        .map(|i| BrowserEntry::directory(format!("folder-{i:05}")))
        .collect();
    state.move_selection(-1);
    state.visible_rows = vec![300, 301];
    let nodes = state.accessibility_nodes(Some(FolderBrowserFocus::Folders), None, true);
    assert!(nodes.len() < 100);
    for name in ["folder-00300", "folder-00301", "folder-09999"] {
        assert!(nodes.iter().any(|n| n.name == name));
    }
}

#[test]
fn inline_readonly_folder_field_skips_tab_but_is_inspectable_after_pointer_focus() {
    use crate::{NewProjectDialogState, NewProjectFocus};
    let mut state = NewProjectDialogState {
        open: true,
        project_name: "Visible default".into(),
        focus: NewProjectFocus::FolderName,
        ..Default::default()
    };
    let nodes = state.accessibility_nodes();
    let field = nodes.iter().find(|n| n.name == "Folder name").unwrap();
    assert!(field.available && field.focused);
    assert!(field.description.contains("Read-only"));
    assert_eq!(field.value.as_deref(), Some("Visible default"));
    state.move_focus(true, false);
    assert_eq!(state.focus, NewProjectFocus::ProjectName);
    state.focus = NewProjectFocus::FolderName;
    state.move_focus(false, false);
    assert_eq!(state.focus, NewProjectFocus::UseProjectName);
}

#[test]
fn folder_browser_compact_and_stale_listings_use_current_typed_location() {
    for mode in [FolderBrowserMode::New, FolderBrowserMode::Open] {
        let mut state = FolderBrowserState::new(mode, "/typed λ ".into());
        let expected = if mode == FolderBrowserMode::New {
            "/typed λ /leaf"
        } else {
            "/typed λ "
        };
        assert_eq!(state.destination(Some("leaf")).as_deref(), Some(expected));
        let nodes = state.accessibility_nodes(None, Some("leaf"), true);
        assert!(nodes.iter().any(|n| n.name == "Location"));
        assert!(nodes.iter().any(|n| n.name == "Browse for Location" && n.value.as_deref() == Some("collapsed")));
        assert!(!nodes.iter().any(|n| matches!(
            n.role,
            crate::GlobalPreferencesAccessibleRole::List
                | crate::GlobalPreferencesAccessibleRole::ListItem
        )));
        for name in ["Up one folder", "Home folder"] {
            assert!(!nodes.iter().any(|n| n.name == name));
        }
        state.expanded = true;
        state.directory = "/old".into();
        state.entries = vec!["stale child".into()];
        state.selected = Some(0);
        for (loading, error) in [
            (true, None),
            (false, Some("Listing permission denied".into())),
        ] {
            state.loading = loading;
            state.error = error;
            assert_eq!(state.destination(Some("leaf")).as_deref(), Some(expected));
        }
        state.path_input.clear();
        assert!(state.destination(Some("leaf")).is_none());
        state.path_input = "a\0b".into();
        assert!(state.destination(Some("leaf")).is_none());
    }
    assert_eq!(
        folder_navigation_focus_order(false),
        vec![
            FolderBrowserFocus::Path,
            FolderBrowserFocus::Browse,
            FolderBrowserFocus::Preview
        ]
    );
    assert!(folder_navigation_focus_order(true).contains(&FolderBrowserFocus::Folders));
}
