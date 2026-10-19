use super::*;

#[test]
fn shared_editor_replaces_selected_unicode_without_trimming_spaces() {
    let mut text = " old λ🙂 name ".to_owned();
    let mut editor = SingleLineEditor::at_end(&text);
    editor.select_all(&text);
    assert_eq!(editor.selected_text(&text), text);
    assert!(editor.replace(&mut text, " exact é🙂 ").unwrap());
    assert_eq!(text, " exact é🙂 ");
    editor.left(&text, false);
    editor.left(&text, true);
    assert_eq!(editor.selected_text(&text), "🙂");
    assert!(editor.backspace(&mut text));
    assert_eq!(text, " exact é ");
    editor.home(&text, false);
    assert!(editor.delete(&mut text));
    assert_eq!(text, "exact é ");
    editor.end(&text, true);
    assert_eq!(editor.selected_text(&text), text);
}

#[test]
fn shared_editor_pointer_ranges_and_external_value_changes_stay_utf8_safe() {
    let mut text = "aéλ🙂z".to_owned();
    let mut editor = SingleLineEditor::default();
    editor.position(&text, 4, false); // Inside λ resolves to its starting boundary.
    assert_eq!(editor.caret, 3);
    editor.position(&text, 9, true);
    assert_eq!(editor.selected_text(&text), "λ🙂");
    editor.left(&text, false);
    assert_eq!(editor.caret, 3);
    editor.right(&text, true);
    assert_eq!(editor.selected_text(&text), "λ");
    editor.replace(&mut text, " ").unwrap();
    assert_eq!(text, "aé 🙂z");
    text = "é".into();
    assert_eq!(editor.selection(&text), 2..2);
    assert!(!editor.delete(&mut text));
    assert!(editor.backspace(&mut text));
    assert!(text.is_empty());
}

#[test]
fn shared_editor_refuses_multiline_paste_without_changing_value_or_selection() {
    let mut text = "keep".to_owned();
    let mut editor = SingleLineEditor::at_end(&text);
    editor.select_all(&text);
    let before = editor;
    for value in ["a\nb", "a\rb", "a\0b", "a\tb"] {
        assert!(editor.replace(&mut text, value).is_err());
        assert_eq!(text, "keep");
        assert_eq!(editor, before);
    }
}
