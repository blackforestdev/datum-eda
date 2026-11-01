//! Library text input over the actual logical key variants delivered by winit.
use super::*;
use datum_gui_protocol::library_browser::LibraryBrowser;

/// Printable keys over a result list start a search instead of disappearing.
/// An explicitly focused source-path editor retains its normal text ownership.
pub(super) fn focus_acquisition_search(
    browser: &mut LibraryBrowser,
    key: &Key,
    modifiers: winit::keyboard::ModifiersState,
) -> bool {
    let Some(a) = &mut browser.acquisition else {
        return false;
    };
    if modifiers.alt_key() || modifiers.super_key() {
        return false;
    }
    if modifiers.control_key() {
        if matches!(key, Key::Character(s) if s.eq_ignore_ascii_case("f")) {
            browser.focus = Some(C::AcquireSearch);
            a.search_editor.select_all(&a.search);
            browser.reveal_focus = true;
            return true;
        }
        return false;
    }
    if matches!(key, Key::Character(s) if !s.chars().any(char::is_control))
        && !matches!(browser.focus, Some(C::AcquirePath | C::AcquireSearch))
    {
        browser.focus = Some(C::AcquireSearch);
        a.search_editor.select_all(&a.search);
        browser.reveal_focus = true;
    }
    false
}

pub(super) fn apply(browser: &mut LibraryBrowser, key: &Key) -> bool {
    let text = match key {
        Key::Character(text) => Some(text.as_str()),
        Key::Named(NamedKey::Backspace) => None,
        Key::Named(NamedKey::Space) => {
            let text_focus = match browser.focus {
                Some(C::Search) => true,
                Some(C::Field(_)) => browser
                    .draft
                    .as_ref()
                    .is_some_and(|d| !d.fields[d.focused].selector),
                _ => false,
            };
            if !text_focus {
                return false;
            }
            Some(" ")
        }
        _ => return false,
    };
    match browser.focus {
        Some(C::Search) => {
            if let Some(text) = text {
                browser.search.push_str(text);
            } else {
                browser.search.pop();
            }
            browser.result_offset = 0;
        }
        Some(C::Field(_)) => {
            if let Some(d) = &mut browser.draft
                && !d.fields[d.focused].selector
            {
                let mut value = d.fields[d.focused].text.clone();
                if let Some(text) = text {
                    value.push_str(text);
                } else {
                    value.pop();
                }
                d.change(value);
            }
        }
        _ => {}
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::library_authoring::LibraryDraft;

    #[test]
    fn acquisition_typing_from_a_result_starts_a_new_search_but_keeps_source_path_editing() {
        use datum_gui_protocol::library_browser::acquisition::LibraryAcquisition;
        let mut browser = LibraryBrowser::default();
        let mut acquisition = LibraryAcquisition::begin("rev".into(), None);
        acquisition.search = "resistor".into();
        browser.acquisition = Some(acquisition);
        browser.focus = Some(C::AcquireResult(0));
        assert!(!focus_acquisition_search(
            &mut browser,
            &Key::Character("c".into()),
            Default::default()
        ));
        assert_eq!(browser.focus, Some(C::AcquireSearch));
        let a = browser.acquisition.as_ref().unwrap();
        assert_eq!(a.search_editor.selected_text(&a.search), "resistor");
        browser.focus = Some(C::AcquirePath);
        assert!(!focus_acquisition_search(
            &mut browser,
            &Key::Character("r".into()),
            Default::default()
        ));
        assert_eq!(browser.focus, Some(C::AcquirePath));
        assert!(focus_acquisition_search(
            &mut browser,
            &Key::Character("f".into()),
            winit::keyboard::ModifiersState::CONTROL
        ));
        assert_eq!(browser.focus, Some(C::AcquireSearch));
    }

    #[test]
    fn library_named_space_preserves_name_and_search_without_editing_selectors() {
        let mut browser = LibraryBrowser::default();
        browser.draft =
            Some(LibraryDraft::begin(&browser, None, "units", "revision".into(), None).unwrap());
        browser.focus = Some(C::Field(0));
        for key in [
            Key::Character("Manual".into()),
            Key::Named(NamedKey::Space),
            Key::Character("Logic".into()),
            Key::Named(NamedKey::Space),
            Key::Character("9gtd".into()),
        ] {
            assert!(apply(&mut browser, &key));
        }
        assert_eq!(
            browser.draft.as_ref().unwrap().value("name"),
            "Manual Logic 9gtd"
        );
        assert!(apply(&mut browser, &Key::Character(" ".into())));
        assert!(apply(&mut browser, &Key::Named(NamedKey::Backspace)));
        assert_eq!(
            browser.draft.as_ref().unwrap().value("name"),
            "Manual Logic 9gtd"
        );
        browser.focus = Some(C::Search);
        browser.result_offset = 8;
        for key in [
            Key::Character("Manual".into()),
            Key::Named(NamedKey::Space),
            Key::Character("Logic".into()),
        ] {
            assert!(apply(&mut browser, &key));
        }
        assert_eq!(browser.search, "Manual Logic");
        assert_eq!(browser.result_offset, 0);
        let draft = browser.draft.as_mut().unwrap();
        draft.focused = draft
            .fields
            .iter()
            .position(|f| f.key == "pin_type")
            .unwrap();
        draft.change("Input".into());
        browser.focus = Some(C::Field(draft.focused));
        assert!(!apply(&mut browser, &Key::Named(NamedKey::Space)));
        assert_eq!(browser.draft.as_ref().unwrap().value("pin_type"), "Input");
        browser.focus = Some(C::AddPin);
        assert!(!apply(&mut browser, &Key::Named(NamedKey::Space)));
    }
}
