//! Field state integration: safe correction, transient movement and row rebinding.
use super::*;

fn draft(kind: &str) -> LibraryDraft {
    LibraryDraft::begin(&LibraryBrowser::default(), None, kind, "basis".into(), None).unwrap()
}
fn edit(
    draft: &mut LibraryDraft,
    action: impl FnOnce(&mut crate::single_line_editor::SingleLineEditor, &mut String),
) {
    let field = &draft.fields[draft.focused];
    let (mut text, mut editor) = (field.text.clone(), field.editor);
    action(&mut editor, &mut text);
    if text != draft.fields[draft.focused].text {
        draft.change(text);
    }
    draft.fields[draft.focused].editor = editor;
}

#[test]
fn unicode_selection_and_numeric_correction_use_field_state_and_invalidate_only_edits() {
    let mut d = draft("padstacks");
    d.focused = d.fields.iter().position(|f| f.key == "name").unwrap();
    d.change("AλZ".into());
    d.dirty.clear();
    d.preview = Some(json!({"preview":"retained"}));
    edit(&mut d, |e, t| {
        e.home(t, false);
        e.right(t, false);
        e.right(t, true);
    });
    assert_eq!(
        d.fields[d.focused].editor.selected_text(d.value("name")),
        "λ"
    );
    assert!(d.dirty.is_empty());
    assert!(d.preview.is_some());
    edit(&mut d, |e, t| {
        e.replace(t, "µ").unwrap();
    });
    assert_eq!(d.value("name"), "AµZ");
    assert!(d.dirty.contains("name"));
    assert!(d.preview.is_none());
    edit(&mut d, |e, t| {
        e.left(t, false);
        e.delete(t);
    });
    assert_eq!(d.value("name"), "AZ");
    d.focused = d.fields.iter().position(|f| f.key == "width").unwrap();
    d.change("1500000".into());
    edit(&mut d, |e, t| {
        e.home(t, false);
        e.right(t, false);
        e.right(t, true);
        e.replace(t, "6").unwrap();
    });
    assert_eq!(d.value("width"), "1600000");
}

#[test]
fn field_switches_keep_selection_but_rebound_rows_reset_it_and_empty_choices_stay_selectors() {
    let mut d = draft("units");
    let name = d.fields.iter().position(|f| f.key == "name").unwrap();
    d.focused = name;
    d.change("Persistent".into());
    edit(&mut d, |e, t| {
        e.home(t, false);
        e.right(t, true);
    });
    let selected = d.fields[name].editor;
    let pin = d.fields.iter().position(|f| f.key == "pin_name").unwrap();
    d.focused = pin;
    d.change("First pin".into());
    assert_eq!(d.fields[name].editor, selected);
    let choice = d.fields.iter().position(|f| f.key == "pin_type").unwrap();
    d.focused = choice;
    d.cycle(true);
    d.focused = pin;
    edit(&mut d, |e, t| e.select_all(t));
    d.switch_pin(None, false).unwrap();
    assert_eq!(d.value("pin_name"), "");
    assert_eq!(d.fields[pin].editor, Default::default());
    d.switch_pin(Some(0), false).unwrap();
    assert_eq!(d.value("pin_name"), "First pin");
    assert_eq!(d.fields[pin].editor.caret, "First pin".len());
    assert_eq!(d.fields[pin].editor.anchor, d.fields[pin].editor.caret);
    let choice = d.fields.iter().position(|f| f.key == "pin_type").unwrap();
    d.focused = choice;
    assert!(d.fields[choice].selector);
    d.fields[choice].choices.clear();
    let before = d.clone();
    d.cycle(true);
    assert_eq!(d, before);
}
