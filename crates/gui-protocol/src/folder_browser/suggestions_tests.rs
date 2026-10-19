use super::*;
fn query(input: &str, names: &[&str]) -> FolderBrowserState {
    let mut s = FolderBrowserState::new(FolderBrowserMode::Open, input.into());
    s.expanded = true;
    s.directory = "/parent".into();
    s.suggestions = Some(FolderSuggestions {
        input: input.into(),
        parent: "/parent".into(),
        fragment: input.rsplit('/').next().unwrap().into(),
    });
    s.entries = names.iter().map(|n| BrowserEntry::directory(*n)).collect();
    s
}
#[test]
fn path_assistance_selection_preserves_exact_candidate_and_requires_current_query() {
    let mut s = query(
        "~/d",
        &["Desktop", "Documents", "Downloads", "kicad_projects"],
    );
    assert!(s.listing_ready() && !s.ready() && !s.opens_selected_manifest());
    s.move_selection(1);
    s.move_selection(1);
    assert_eq!(s.preview(None).as_deref(), Some("~/d"));
    assert_eq!(s.preview(Some("new")).as_deref(), Some("~/d"));
    assert_eq!(s.tab_suggestion().as_deref(), Some("~/Documents/"));
    let nodes = s.accessibility_nodes(Some(FolderBrowserFocus::Path), None, true);
    assert!(
        nodes
            .iter()
            .any(|n| n.name == "Documents" && n.value.as_deref().unwrap().contains("selected"))
    );
    s.path_input = "~/do".into();
    assert!(!s.suggestions_ready());
    assert!(s.tab_suggestion().is_none());
}
#[test]
fn path_assistance_completion_preserves_spelling_focus_escape_options_and_ambiguous_boundaries() {
    let s = query("relative/ca", &["CaSe λ "]);
    assert_eq!(s.tab_suggestion().as_deref(), Some("relative/CaSe λ /"));
    let mut s = query("~/Doc", &["Documents", "Documents-old"]);
    assert_eq!(
        s.tab_suggestion().as_deref(),
        Some("~/Documents"),
        "unsuffixed common start remains a query, even when a folder has that name"
    );
    s.entries = vec![
        BrowserEntry::directory("Documents-alpha"),
        BrowserEntry::directory("Documents-beta"),
    ];
    assert_eq!(s.tab_suggestion().as_deref(), Some("~/Documents-"));
    s.path_editor.home(&s.path_input, false);
    assert!(s.tab_suggestion().is_none());
    s.path_editor.select_all(&s.path_input);
    assert!(s.tab_suggestion().is_none());
    let s = query("~/d", &["Desktop", "kicad_projects"]);
    assert_eq!(
        s.tab_suggestion().as_deref(),
        Some("~/Desktop/"),
        "unique prefix wins over incidental substring"
    );
    let s = query("~/missing", &[]);
    assert!(s.tab_suggestion().is_none());
    assert!(s.refusal(None).is_none());
    assert!(
        s.suggestion_header()
            .unwrap()
            .contains("no matching folders")
    );
    assert_eq!(folder_match_span("CaSe λ /", "case"), Some(0..4));
    assert_eq!(folder_match_span("İstanbul/", "i"), Some(0..2));
}

#[test]
fn successive_completion_keeps_exact_named_siblings_and_explicit_loaded_tab_never_chooses_child() {
    let s = query(
        "~/Documents/datum-ed",
        &["datum-eda", "datum-eda-backups", "datum-eda-workspaces"],
    );
    assert_eq!(s.tab_suggestion().as_deref(), Some("~/Documents/datum-eda"));
    let mut s = query(
        "~/Documents/datum-eda",
        &["datum-eda", "datum-eda-backups", "datum-eda-workspaces"],
    );
    assert!(s.tab_suggestion().is_none());
    assert!(!s.ready());
    assert_eq!(s.preview(None).as_deref(), Some("~/Documents/datum-eda"));
    s.move_selection(1);
    assert_eq!(
        s.tab_suggestion().as_deref(),
        Some("~/Documents/datum-eda/")
    );
    s.suggestions = None;
    s.directory = s.path_input.clone();
    s.entries = vec![BrowserEntry::directory("a-child")];
    s.selected = Some(0);
    assert_eq!(
        s.tab_suggestion().as_deref(),
        Some("~/Documents/datum-eda/"),
        "explicit loaded path accepts itself, never child"
    );
    s.path_input.push('/');
    s.directory = s.path_input.clone();
    s.path_editor = crate::single_line_editor::SingleLineEditor::at_end(&s.path_input);
    assert!(s.tab_suggestion().is_none());
}

#[test]
fn prefix_group_completes_before_substrings_without_hiding_rows_or_overriding_selection() {
    let names = [
        "datum-eda",
        "datum-eda-backups",
        "datum-eda-workspaces",
        "datum_projects",
        "plugdata",
    ];
    let mut s = query("~/Documents/dat", &names);
    assert_eq!(s.tab_suggestion().as_deref(), Some("~/Documents/datum"));
    assert_eq!(s.entries.len(), 5);
    assert_eq!(s.preview(None).as_deref(), Some("~/Documents/dat"));
    s.selected = Some(4);
    assert_eq!(
        s.tab_suggestion().as_deref(),
        Some("~/Documents/plugdata/"),
        "explicit substring selection wins"
    );
    let s = query("~/foo", &["FooBar", "afoo", "bfoo"]);
    assert_eq!(
        s.tab_suggestion().as_deref(),
        Some("~/FooBar/"),
        "case-insensitive unique prefix retains exact spelling"
    );
    let s = query("~/obar", &["FooBar"]);
    assert_eq!(
        s.tab_suggestion().as_deref(),
        Some("~/FooBar/"),
        "no-prefix unique substring fallback"
    );
    let s = query("~/dat", &["Datum-one", "Datum-two", "plugdata"]);
    assert!(
        s.tab_suggestion().is_none(),
        "case folding cannot synthesize multiple-name prefix spelling"
    );
}
