use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey},
};

pub(super) enum TerminalKeyAction {
    Write(Vec<u8>),
    Interrupt,
    RestartSession,
    TerminateSession,
    LetPasteShortcutHandle,
    LetCopyShortcutHandle,
    ConsumeRelease,
    Ignore,
}

pub(super) fn terminal_key_action(
    event: &KeyEvent,
    modifiers: ModifiersState,
) -> TerminalKeyAction {
    if event.state == ElementState::Released {
        return if consumes_release(event) {
            TerminalKeyAction::ConsumeRelease
        } else {
            TerminalKeyAction::Ignore
        };
    }
    if modifiers.control_key() {
        if modifiers.shift_key() && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyR)) {
            return TerminalKeyAction::RestartSession;
        }
        if modifiers.shift_key() && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyK)) {
            return TerminalKeyAction::TerminateSession;
        }
        if matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyC)) {
            return terminal_ctrl_c_action(modifiers);
        }
        if matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyV)) {
            return TerminalKeyAction::LetPasteShortcutHandle;
        }
    }
    match &event.logical_key {
        Key::Character(text) if !modifiers.control_key() => {
            TerminalKeyAction::Write(text.as_bytes().to_vec())
        }
        Key::Named(NamedKey::Space) => TerminalKeyAction::Write(b" ".to_vec()),
        Key::Named(NamedKey::Enter) => TerminalKeyAction::Write(b"\r".to_vec()),
        Key::Named(NamedKey::Backspace) => TerminalKeyAction::Write(b"\x7f".to_vec()),
        Key::Named(NamedKey::Tab) => TerminalKeyAction::Write(b"\t".to_vec()),
        Key::Named(NamedKey::ArrowLeft) => TerminalKeyAction::Write(b"\x1b[D".to_vec()),
        Key::Named(NamedKey::ArrowRight) => TerminalKeyAction::Write(b"\x1b[C".to_vec()),
        Key::Named(NamedKey::ArrowUp) => TerminalKeyAction::Write(b"\x1b[A".to_vec()),
        Key::Named(NamedKey::ArrowDown) => TerminalKeyAction::Write(b"\x1b[B".to_vec()),
        Key::Named(NamedKey::Home) => TerminalKeyAction::Write(b"\x1b[H".to_vec()),
        Key::Named(NamedKey::End) => TerminalKeyAction::Write(b"\x1b[F".to_vec()),
        Key::Named(NamedKey::Escape) => TerminalKeyAction::Write(b"\x1b".to_vec()),
        _ => TerminalKeyAction::Ignore,
    }
}

fn terminal_ctrl_c_action(modifiers: ModifiersState) -> TerminalKeyAction {
    if modifiers.shift_key() {
        TerminalKeyAction::LetCopyShortcutHandle
    } else {
        TerminalKeyAction::Interrupt
    }
}

fn consumes_release(event: &KeyEvent) -> bool {
    matches!(
        event.logical_key,
        Key::Named(
            NamedKey::Enter
                | NamedKey::Backspace
                | NamedKey::Tab
                | NamedKey::ArrowLeft
                | NamedKey::ArrowRight
                | NamedKey::ArrowUp
                | NamedKey::ArrowDown
                | NamedKey::Home
                | NamedKey::End
                | NamedKey::Escape
        )
    ) || matches!(
        event.physical_key,
        PhysicalKey::Code(KeyCode::KeyC | KeyCode::KeyV | KeyCode::KeyK | KeyCode::KeyR)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_c_interrupts_but_ctrl_shift_c_defers_to_copy() {
        let ctrl = ModifiersState::CONTROL;
        let ctrl_shift = ModifiersState::CONTROL | ModifiersState::SHIFT;

        assert!(matches!(
            terminal_ctrl_c_action(ctrl),
            TerminalKeyAction::Interrupt
        ));
        assert!(matches!(
            terminal_ctrl_c_action(ctrl_shift),
            TerminalKeyAction::LetCopyShortcutHandle
        ));
    }
}
