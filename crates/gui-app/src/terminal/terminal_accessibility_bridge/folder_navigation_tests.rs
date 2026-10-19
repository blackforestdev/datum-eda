//! Focused inline Project preview accessibility binding.
use super::*;

#[test]
fn folder_browser_inline_preview_binding_exposes_complete_form_refusal_without_changing_path() {
    use datum_gui_protocol::{
        FolderBrowserFocus as Focus, FolderBrowserMode as Mode, FolderBrowserState,
    };
    let mut ui = datum_gui_protocol::load_fixture_workspace_state().ui;
    let reason = format!(
        "Permission denied: {} COMPLETE-REFUSAL-TAIL",
        "long details ".repeat(100)
    );
    ui.new_project.project_name = " exact leaf λ ".into();
    ui.new_project.refusal = Some(reason.clone());
    ui.project_open.refusal = Some(reason.clone());
    for (mode, expanded) in [
        (Mode::New, false),
        (Mode::New, true),
        (Mode::Open, false),
        (Mode::Open, true),
    ] {
        let mut browser = FolderBrowserState::new(mode, "/exact parent λ".into());
        browser.loading = false;
        browser.expanded = expanded;
        browser.directory = "/exact parent λ".into();
        let path = browser.preview((mode == Mode::New).then_some(" exact leaf λ "));
        assert_eq!(
            path.as_deref(),
            Some(if mode == Mode::New {
                "/exact parent λ/ exact leaf λ "
            } else {
                "/exact parent λ"
            })
        );
        let nodes = project_navigation_accessibility(&ui, &browser, Some(Focus::Preview));
        let preview = nodes
            .iter()
            .find(|n| n.id == "folder-navigation-preview")
            .unwrap();
        assert!(preview.focused);
        assert!(
            preview
                .description
                .ends_with(&format!("Full refusal details\n{reason}"))
        );
        assert_eq!(
            preview.value,
            path.clone().map(|p| if mode == Mode::Open && expanded {
                format!("{p} (this folder)")
            } else {
                p
            })
        );
        assert_eq!(
            browser.preview((mode == Mode::New).then_some(" exact leaf λ ")),
            path
        );
        browser.error = Some("Distinct folder read reason".into());
        let nodes = project_navigation_accessibility(&ui, &browser, Some(Focus::Preview));
        let preview = nodes
            .iter()
            .find(|n| n.id == "folder-navigation-preview")
            .unwrap();
        assert!(preview.description.contains(&reason));
        assert!(preview.description.contains("Distinct folder read reason"));
    }
}
