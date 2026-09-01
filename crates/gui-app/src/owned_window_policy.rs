//! Shared focus and stacking policy for application-owned native windows.
//!
//! Winit does not expose portable transient-parent metadata on every Linux
//! backend. Datum therefore enforces the observable ownership contract at its
//! application focus boundary instead of making these windows globally
//! always-on-top.

use std::sync::Arc;
use winit::window::{UserAttentionType, Window};

pub(super) const fn owner_activation_raises_owned_window(
    owner_focused: bool,
    owned_window_open: bool,
) -> bool {
    owner_focused && owned_window_open
}

pub(super) fn redirect_owner_activation(
    owner_focused: bool,
    owned_window: Option<&Arc<Window>>,
) -> bool {
    if !owner_activation_raises_owned_window(owner_focused, owned_window.is_some()) {
        return false;
    }
    let window = owned_window.expect("open owned window checked above");
    raise_owned_window(window);
    true
}

pub(super) fn show_owned_window(window: &Window) {
    window.set_visible(true);
    window.focus_window();
    window.request_redraw();
}

pub(super) fn raise_owned_window(window: &Window) {
    show_owned_window(window);
    window.request_user_attention(Some(UserAttentionType::Informational));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_owner_activation_with_an_open_owned_window_redirects_focus() {
        assert!(owner_activation_raises_owned_window(true, true));
        assert!(!owner_activation_raises_owned_window(false, true));
        assert!(!owner_activation_raises_owned_window(true, false));
        assert!(!owner_activation_raises_owned_window(false, false));
    }
}
