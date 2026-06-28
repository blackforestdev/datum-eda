use super::*;

const TERMINAL_AGENT_LAUNCHER_PREFILL: &str = "# Claude alternative: claude 'You are running inside Datum EDA. Read $DATUM_DISCOVERY and inspect recent activity with datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20 before acting.'\ncodex 'You are running inside Datum EDA. Read the Datum context from $DATUM_DISCOVERY before acting, use datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20 for recent GUI/terminal activity, and use datum-eda CLI commands for project-aware work.'";

impl Runtime {
    pub(super) fn open_terminal_agent_launcher(&mut self) -> bool {
        self.set_active_dock(DockTab::Terminal);
        if let Err(err) = record_manual_terminal_command_handoff(
            self.terminal_sessions.active(),
            "terminal_agent_launcher",
            "datum.gui.agent_launcher.prefill",
            "prefill",
            TERMINAL_AGENT_LAUNCHER_PREFILL,
        ) {
            self.push_terminal_line(format!("terminal handoff event write failed: {err}"));
        }
        self.write_terminal_bytes(TERMINAL_AGENT_LAUNCHER_PREFILL.as_bytes());
        self.log_review_event("opened terminal agent launcher".to_string());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_agent_launcher_prefills_codex_without_executing() {
        assert!(TERMINAL_AGENT_LAUNCHER_PREFILL.starts_with("# Claude alternative:"));
        assert!(TERMINAL_AGENT_LAUNCHER_PREFILL.contains("claude '"));
        assert!(TERMINAL_AGENT_LAUNCHER_PREFILL.contains("\ncodex '"));
        assert!(TERMINAL_AGENT_LAUNCHER_PREFILL.contains("$DATUM_DISCOVERY"));
        assert!(TERMINAL_AGENT_LAUNCHER_PREFILL.contains(ASSISTANT_ACTIVITY_COMMAND));
        assert!(TERMINAL_AGENT_LAUNCHER_PREFILL.contains("$DATUM_SESSION_ID"));
        assert!(TERMINAL_AGENT_LAUNCHER_PREFILL.ends_with("project-aware work.'"));
        assert!(!TERMINAL_AGENT_LAUNCHER_PREFILL.ends_with('\r'));
    }
}
