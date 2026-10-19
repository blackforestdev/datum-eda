//! Shared AT-SPI state bitmap and wire encoding for every accessible surface.
use super::*;

pub(super) fn state_body(
    terminal: bool,
    focused: bool,
    preference: Option<(bool, bool)>,
) -> Vec<u8> {
    words_body(state_words(terminal, focused, preference))
}

/// Toggle state is semantic, independent of its current keyboard focus.
pub(super) fn preference_state_body(node: &GlobalPreferencesAccessibleNode) -> Vec<u8> {
    let mut words = state_words(false, false, Some((node.available, node.focused)));
    if node.role == GlobalPreferencesAccessibleRole::CheckBox
        && node.value.as_deref() == Some("true")
    {
        // ATSPI_STATE_CHECKED from the platform's atspi-constants.h enumeration.
        words[0] |= 1 << 4;
    }
    if node.id == "folder-navigation-browse" {
        // ATSPI_STATE_EXPANDABLE (9), EXPANDED (10), COLLAPSED (5).
        words[0] |= 1 << 9;
        words[0] |= 1
            << if node.value.as_deref() == Some("expanded") {
                10
            } else {
                5
            };
    }
    words_body(words)
}

pub(super) fn state_words(
    terminal: bool,
    focused: bool,
    preference: Option<(bool, bool)>,
) -> [u32; 2] {
    let mut words = [0_u32; 2];
    let states = if terminal {
        &[
            STATE_EDITABLE,
            STATE_ENABLED,
            STATE_FOCUSABLE,
            STATE_MULTI_LINE,
            STATE_SENSITIVE,
            STATE_SHOWING,
            STATE_VISIBLE,
            STATE_SELECTABLE_TEXT,
        ][..]
    } else if preference.is_some() {
        &[STATE_FOCUSABLE, STATE_SHOWING, STATE_VISIBLE][..]
    } else {
        &[STATE_ENABLED, STATE_SENSITIVE, STATE_SHOWING, STATE_VISIBLE][..]
    };
    let available = preference.map(|(available, _)| available).unwrap_or(true);
    let focused = focused || preference.map(|(_, focused)| focused).unwrap_or(false);
    for state in states
        .iter()
        .copied()
        .chain(available.then_some(STATE_ENABLED))
        .chain(available.then_some(STATE_SENSITIVE))
        .chain(focused.then_some(STATE_FOCUSED))
    {
        words[(state / 32) as usize] |= 1 << (state % 32);
    }
    words
}

pub(super) fn words_body(words: [u32; 2]) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.array(4, |body| {
        body.u32(words[0]);
        body.u32(words[1]);
    });
    body.finish()
}
