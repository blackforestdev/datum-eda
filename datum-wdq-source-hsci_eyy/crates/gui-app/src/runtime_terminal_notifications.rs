//! Desktop projection for bounded terminal notification events.
//!
//! TerminalCore owns parsing and payload bounds. This adapter performs only a
//! display handoff; notification text is never interpreted as a command or
//! persisted as a Datum operation.

use std::{
    process::Command,
    sync::mpsc::{SyncSender, TrySendError, sync_channel},
};

use crate::{
    Runtime, terminal_core_adapter::PRODUCTION_CORE_LIMIT_VALUES,
    terminal_session::TerminalNotificationRequest,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TerminalNotificationPolicy {
    Off,
    Unfocused,
    Always,
}

struct DesktopNotification {
    title: String,
    text: String,
}

pub(super) struct TerminalNotificationBridge {
    policy: TerminalNotificationPolicy,
    sender: SyncSender<DesktopNotification>,
}

impl TerminalNotificationPolicy {
    pub(super) fn from_environment() -> Self {
        match std::env::var("DATUM_TERMINAL_NOTIFICATIONS")
            .unwrap_or_else(|_| "unfocused".to_string())
            .to_ascii_lowercase()
            .as_str()
        {
            "off" => Self::Off,
            "always" => Self::Always,
            _ => Self::Unfocused,
        }
    }

    const fn should_deliver(self, window_focused: bool) -> bool {
        match self {
            Self::Off => false,
            Self::Unfocused => !window_focused,
            Self::Always => true,
        }
    }
}

impl TerminalNotificationBridge {
    pub(super) fn from_environment() -> Self {
        let policy = TerminalNotificationPolicy::from_environment();
        let (sender, receiver) =
            sync_channel::<DesktopNotification>(PRODUCTION_CORE_LIMIT_VALUES.pending_events);
        std::thread::Builder::new()
            .name("datum-terminal-notifications".to_string())
            .spawn(move || {
                while let Ok(notification) = receiver.recv() {
                    let _ = Command::new("/usr/bin/notify-send")
                        .args([
                            "--app-name=Datum EDA",
                            notification.title.as_str(),
                            notification.text.as_str(),
                        ])
                        .status();
                }
            })
            .expect("terminal notification worker should start");
        Self { policy, sender }
    }

    fn publish(&self, window_focused: bool, notification: DesktopNotification) -> bool {
        if !self.policy.should_deliver(window_focused) || notification.text.contains('\0') {
            return false;
        }
        match self.sender.try_send(notification) {
            Ok(()) => true,
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => false,
        }
    }
}

impl Runtime {
    pub(super) fn handle_terminal_notification(
        &mut self,
        notification: TerminalNotificationRequest,
    ) -> bool {
        let title = self
            .workspace()
            .ui
            .terminal
            .tabs
            .iter()
            .find(|tab| tab.session_id == notification.session_id)
            .map(|tab| format!("Datum terminal — {}", tab.label))
            .unwrap_or_else(|| "Datum terminal".to_string());
        self.terminal_notification_bridge.publish(
            self.window_focused,
            DesktopNotification {
                title,
                text: notification.text,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::TerminalNotificationPolicy;

    #[test]
    fn notification_policy_matches_off_unfocused_and_always_contract() {
        assert!(!TerminalNotificationPolicy::Off.should_deliver(false));
        assert!(!TerminalNotificationPolicy::Off.should_deliver(true));
        assert!(TerminalNotificationPolicy::Unfocused.should_deliver(false));
        assert!(!TerminalNotificationPolicy::Unfocused.should_deliver(true));
        assert!(TerminalNotificationPolicy::Always.should_deliver(false));
        assert!(TerminalNotificationPolicy::Always.should_deliver(true));
    }
}
