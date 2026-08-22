//! User-selected terminal launch templates over the owned PTY transport.

use crate::terminal_agent_authority::TerminalAgentAuthority;
use anyhow::{Context, Result};
use datum_gui_protocol::{
    TERMINAL_FONT_SCALE_DEFAULT_MILLIS, TERMINAL_FONT_SCALE_MAX_MILLIS,
    TERMINAL_FONT_SCALE_MIN_MILLIS, TerminalTheme,
};
use datum_terminal_core::{CoreLimitValues, CursorShape};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, clap::Args)]
pub(super) struct TerminalProfileArgs {
    /// Select `default`, `login`, or the configured custom profile for new terminals.
    #[arg(long = "terminal-session-profile", default_value = "default")]
    selected: String,
    /// User-facing name for the optional custom terminal profile.
    #[arg(long = "terminal-profile-name", default_value = "custom")]
    custom_name: String,
    /// Executable for the custom profile. Without this, the user's `$SHELL` is used.
    #[arg(long = "terminal-program")]
    program: Option<OsString>,
    /// One exact argv element for the custom profile; repeat to add arguments.
    #[arg(long = "terminal-arg", allow_hyphen_values = true)]
    args: Vec<OsString>,
    /// Set one inherited environment key as `KEY=VALUE`; repeat as needed.
    #[arg(long = "terminal-env", value_name = "KEY=VALUE")]
    environment: Vec<String>,
    /// Remove one inherited environment key; removals are applied after sets.
    #[arg(long = "terminal-env-remove", value_name = "KEY")]
    environment_remove: Vec<String>,
    /// Initial cwd: `active`, `project`, or a path relative to the project root.
    #[arg(long = "terminal-cwd", default_value = "active")]
    cwd: String,
    /// Initial custom-profile theme: datum-dark, high-contrast, or light.
    #[arg(long = "terminal-theme")]
    theme: Option<String>,
    /// Initial custom-profile terminal font scale as an integer percent (60..=200).
    #[arg(long = "terminal-font-scale-percent")]
    font_scale_percent: Option<u16>,
    /// Retained logical scrollback lines for the custom profile (1..=100000).
    #[arg(long = "terminal-scrollback-lines")]
    history_lines: Option<usize>,
    /// Retained scrollback text in MiB for the custom profile (1..=64).
    #[arg(long = "terminal-scrollback-mebibytes")]
    history_mebibytes: Option<usize>,
    /// Initial cursor: blinking-block, steady-block, blinking-underline,
    /// steady-underline, blinking-bar, or steady-bar.
    #[arg(long = "terminal-cursor")]
    cursor: Option<String>,
    /// Visual bell presentation for this profile: visual or off.
    #[arg(long = "terminal-bell")]
    bell: Option<String>,
    /// Datum-domain agent authority: inspect, propose, apply-approved, or unattended.
    #[arg(long = "terminal-agent-authority", default_value = "propose")]
    agent_authority: String,
    /// Canonical MCP tool permitted for unattended use; repeat to grant a narrow set.
    #[arg(long = "terminal-agent-unattended-tool", value_name = "DATUM.TOOL")]
    unattended_tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TerminalLaunchProfile {
    name: String,
    executable: Option<OsString>,
    args: Vec<OsString>,
    cwd: TerminalCwdTemplate,
    environment: Vec<(OsString, Option<OsString>)>,
    theme: TerminalTheme,
    font_scale_millis: u16,
    history_lines: usize,
    history_bytes: usize,
    cursor_shape: CursorShape,
    cursor_blinking: bool,
    visual_bell_enabled: bool,
    agent_authority: TerminalAgentAuthority,
    unattended_tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TerminalCwdTemplate {
    Active,
    Project,
    Path(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedTerminalLaunchProfile {
    pub(super) name: String,
    pub(super) executable: OsString,
    pub(super) args: Vec<OsString>,
    pub(super) cwd: PathBuf,
    pub(super) environment: Vec<(OsString, Option<OsString>)>,
}

#[derive(Debug, Clone)]
pub(super) struct TerminalProfileCatalog {
    profiles: Vec<TerminalLaunchProfile>,
    selected: usize,
}

impl Default for TerminalLaunchProfile {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            executable: None,
            args: Vec::new(),
            cwd: TerminalCwdTemplate::Active,
            environment: Vec::new(),
            theme: TerminalTheme::DatumDark,
            font_scale_millis: TERMINAL_FONT_SCALE_DEFAULT_MILLIS,
            history_lines: crate::terminal_core_adapter::PRODUCTION_CORE_LIMIT_VALUES.history_lines,
            history_bytes: crate::terminal_core_adapter::PRODUCTION_CORE_LIMIT_VALUES.history_bytes,
            cursor_shape: CursorShape::Block,
            cursor_blinking: true,
            visual_bell_enabled: true,
            agent_authority: TerminalAgentAuthority::Propose,
            unattended_tools: Vec::new(),
        }
    }
}

impl TerminalLaunchProfile {
    fn login() -> Self {
        Self {
            name: "login".to_string(),
            args: vec![OsString::from("-l")],
            ..Self::default()
        }
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn theme(&self) -> TerminalTheme {
        self.theme
    }

    pub(super) fn font_scale_millis(&self) -> u16 {
        self.font_scale_millis
    }

    pub(super) fn core_limit_values(&self) -> CoreLimitValues {
        CoreLimitValues {
            history_lines: self.history_lines,
            history_bytes: self.history_bytes,
            ..crate::terminal_core_adapter::PRODUCTION_CORE_LIMIT_VALUES
        }
    }

    pub(super) fn cursor_preference(&self) -> (CursorShape, bool) {
        (self.cursor_shape, self.cursor_blinking)
    }

    pub(super) fn visual_bell_enabled(&self) -> bool {
        self.visual_bell_enabled
    }

    pub(super) fn agent_capabilities(&self) -> &'static [&'static str] {
        self.agent_authority.capabilities()
    }

    pub(super) fn agent_approval_policy(&self) -> &'static str {
        self.agent_authority.approval_policy()
    }

    pub(super) fn unattended_tools(&self) -> &[String] {
        &self.unattended_tools
    }

    pub(super) fn resolve(
        &self,
        project_root: &Path,
        active_cwd: &Path,
    ) -> ResolvedTerminalLaunchProfile {
        self.resolve_with_shell(
            project_root,
            active_cwd,
            std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("/bin/sh")),
        )
    }

    fn resolve_with_shell(
        &self,
        project_root: &Path,
        active_cwd: &Path,
        shell: OsString,
    ) -> ResolvedTerminalLaunchProfile {
        let cwd = match &self.cwd {
            TerminalCwdTemplate::Active => active_cwd.to_path_buf(),
            TerminalCwdTemplate::Project => project_root.to_path_buf(),
            TerminalCwdTemplate::Path(path) if path.is_absolute() => path.clone(),
            TerminalCwdTemplate::Path(path) => project_root.join(path),
        };
        ResolvedTerminalLaunchProfile {
            name: self.name.clone(),
            executable: self.executable.clone().unwrap_or(shell),
            args: self.args.clone(),
            cwd,
            environment: self.environment.clone(),
        }
    }
}

impl TerminalProfileCatalog {
    pub(super) fn from_args(args: &TerminalProfileArgs) -> Result<Self> {
        let custom = custom_profile(args)?;
        let agent_authority = TerminalAgentAuthority::parse(&args.agent_authority)?;
        agent_authority.validate_unattended_tools(&args.unattended_tools)?;
        let mut profiles = vec![
            TerminalLaunchProfile::default(),
            TerminalLaunchProfile::login(),
        ];
        if let Some(custom) = custom {
            profiles.push(custom);
        }
        for profile in &mut profiles {
            profile.agent_authority = agent_authority;
            profile.unattended_tools = args.unattended_tools.clone();
        }
        let selected = profiles
            .iter()
            .position(|profile| profile.name == args.selected)
            .with_context(|| {
                format!(
                    "unknown terminal session profile {:?}; available: {}",
                    args.selected,
                    profiles
                        .iter()
                        .map(|profile| profile.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        Ok(Self { profiles, selected })
    }

    pub(super) fn selected(&self) -> &TerminalLaunchProfile {
        &self.profiles[self.selected]
    }

    pub(super) fn select_next(&mut self) -> &TerminalLaunchProfile {
        self.selected = (self.selected + 1) % self.profiles.len();
        self.selected()
    }
}

fn custom_profile(args: &TerminalProfileArgs) -> Result<Option<TerminalLaunchProfile>> {
    let has_custom = args.program.is_some()
        || !args.args.is_empty()
        || !args.environment.is_empty()
        || !args.environment_remove.is_empty()
        || args.cwd != "active"
        || args.theme.is_some()
        || args.font_scale_percent.is_some()
        || args.history_lines.is_some()
        || args.history_mebibytes.is_some()
        || args.cursor.is_some()
        || args.bell.is_some()
        || args.custom_name != "custom"
        || args.selected == args.custom_name;
    if !has_custom {
        return Ok(None);
    }
    if args.custom_name.is_empty() || matches!(args.custom_name.as_str(), "default" | "login") {
        anyhow::bail!("custom terminal profile name must be non-empty and not default or login");
    }
    let cwd = match args.cwd.as_str() {
        "active" => TerminalCwdTemplate::Active,
        "project" => TerminalCwdTemplate::Project,
        path => TerminalCwdTemplate::Path(PathBuf::from(path)),
    };
    let theme = match args.theme.as_deref() {
        None | Some("datum-dark") => TerminalTheme::DatumDark,
        Some("high-contrast") => TerminalTheme::HighContrast,
        Some("light") => TerminalTheme::Light,
        Some(value) => anyhow::bail!(
            "unknown terminal theme {value:?}; expected datum-dark, high-contrast, or light"
        ),
    };
    let font_scale_millis = match args.font_scale_percent {
        None => TERMINAL_FONT_SCALE_DEFAULT_MILLIS,
        Some(percent) => percent.checked_mul(10).with_context(|| {
            format!("terminal font scale percent {percent} cannot be represented")
        })?,
    };
    if !(TERMINAL_FONT_SCALE_MIN_MILLIS..=TERMINAL_FONT_SCALE_MAX_MILLIS)
        .contains(&font_scale_millis)
    {
        anyhow::bail!("terminal font scale percent must be between 60 and 200");
    }
    let approved = crate::terminal_core_adapter::PRODUCTION_CORE_LIMIT_VALUES;
    let history_lines = args.history_lines.unwrap_or(approved.history_lines);
    if history_lines == 0 || history_lines > approved.history_lines {
        anyhow::bail!(
            "terminal scrollback lines must be between 1 and {}",
            approved.history_lines
        );
    }
    let history_mebibytes = args
        .history_mebibytes
        .unwrap_or(approved.history_bytes / (1024 * 1024));
    let history_bytes = history_mebibytes
        .checked_mul(1024 * 1024)
        .context("terminal scrollback MiB cannot be represented")?;
    if history_mebibytes == 0 || history_bytes > approved.history_bytes {
        anyhow::bail!(
            "terminal scrollback MiB must be between 1 and {}",
            approved.history_bytes / (1024 * 1024)
        );
    }
    let (cursor_shape, cursor_blinking) = match args.cursor.as_deref() {
        None | Some("blinking-block") => (CursorShape::Block, true),
        Some("steady-block") => (CursorShape::Block, false),
        Some("blinking-underline") => (CursorShape::Underline, true),
        Some("steady-underline") => (CursorShape::Underline, false),
        Some("blinking-bar") => (CursorShape::Bar, true),
        Some("steady-bar") => (CursorShape::Bar, false),
        Some(value) => anyhow::bail!(
            "unknown terminal cursor {value:?}; expected blinking-block, steady-block, blinking-underline, steady-underline, blinking-bar, or steady-bar"
        ),
    };
    let visual_bell_enabled = match args.bell.as_deref() {
        None | Some("visual") => true,
        Some("off") => false,
        Some(value) => anyhow::bail!("unknown terminal bell {value:?}; expected visual or off"),
    };
    let mut environment = Vec::new();
    for assignment in &args.environment {
        let (key, value) = assignment
            .split_once('=')
            .with_context(|| format!("terminal environment must use KEY=VALUE: {assignment:?}"))?;
        validate_environment_key(key)?;
        environment.push((OsString::from(key), Some(OsString::from(value))));
    }
    for key in &args.environment_remove {
        validate_environment_key(key)?;
        environment.push((OsString::from(key), None));
    }
    Ok(Some(TerminalLaunchProfile {
        name: args.custom_name.clone(),
        executable: args.program.clone(),
        args: args.args.clone(),
        cwd,
        environment,
        theme,
        font_scale_millis,
        history_lines,
        history_bytes,
        cursor_shape,
        cursor_blinking,
        visual_bell_enabled,
        agent_authority: TerminalAgentAuthority::Propose,
        unattended_tools: Vec::new(),
    }))
}

fn validate_environment_key(key: &str) -> Result<()> {
    if key.is_empty() || key.contains('=') || key.contains('\0') {
        anyhow::bail!("terminal environment key must be non-empty and contain neither '=' nor NUL");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_session::{TerminalEvent, TerminalLaunchContext, spawn_terminal_session};
    use clap::Parser;
    use std::{fs, time::Duration};

    #[derive(Parser)]
    struct FixtureArgs {
        #[command(flatten)]
        terminal: TerminalProfileArgs,
    }

    fn parse(values: &[&str]) -> TerminalProfileCatalog {
        let mut argv = vec!["fixture"];
        argv.extend_from_slice(values);
        let args = FixtureArgs::try_parse_from(argv).unwrap();
        TerminalProfileCatalog::from_args(&args.terminal).unwrap()
    }

    #[test]
    fn default_and_login_profiles_preserve_active_cwd_and_exact_argv() {
        let mut catalog = parse(&[]);
        assert_eq!(
            catalog.selected().agent_capabilities(),
            ["inspect", "propose"]
        );
        assert_eq!(
            catalog.selected().agent_approval_policy(),
            "owner-review-required"
        );
        assert!(catalog.selected().unattended_tools().is_empty());
        assert_eq!(
            catalog.selected().core_limit_values(),
            crate::terminal_core_adapter::PRODUCTION_CORE_LIMIT_VALUES
        );
        let default = catalog.selected().resolve_with_shell(
            Path::new("/project"),
            Path::new("/active"),
            OsString::from("/bin/zsh"),
        );
        assert_eq!(default.executable, "/bin/zsh");
        assert!(default.args.is_empty());
        assert_eq!(default.cwd, Path::new("/active"));

        let login = catalog.select_next().resolve_with_shell(
            Path::new("/project"),
            Path::new("/active"),
            OsString::from("/bin/zsh"),
        );
        assert_eq!(login.executable, "/bin/zsh");
        assert_eq!(login.args, [OsString::from("-l")]);
        assert_eq!(login.cwd, Path::new("/active"));
        assert_eq!(
            catalog.selected().core_limit_values(),
            crate::terminal_core_adapter::PRODUCTION_CORE_LIMIT_VALUES
        );
        assert_eq!(catalog.select_next().name(), "default");
    }

    #[test]
    fn custom_profile_preserves_program_argv_cwd_env_and_appearance() {
        let catalog = parse(&[
            "--terminal-session-profile",
            "agent",
            "--terminal-profile-name",
            "agent",
            "--terminal-program",
            "/usr/bin/env",
            "--terminal-arg",
            "--literal;$(not-a-shell)",
            "--terminal-cwd",
            "tools",
            "--terminal-env",
            "AGENT_MODE=1",
            "--terminal-env-remove",
            "OLD_AGENT_MODE",
            "--terminal-theme",
            "light",
            "--terminal-font-scale-percent",
            "130",
            "--terminal-scrollback-lines",
            "24000",
            "--terminal-scrollback-mebibytes",
            "12",
            "--terminal-cursor",
            "steady-bar",
            "--terminal-bell",
            "off",
        ]);
        let profile = catalog.selected();
        let resolved = profile.resolve_with_shell(
            Path::new("/project"),
            Path::new("/active"),
            OsString::from("/bin/sh"),
        );
        assert_eq!(resolved.name, "agent");
        assert_eq!(resolved.executable, "/usr/bin/env");
        assert_eq!(resolved.args, [OsString::from("--literal;$(not-a-shell)")]);
        assert_eq!(resolved.cwd, Path::new("/project/tools"));
        assert_eq!(
            resolved.environment,
            [
                (OsString::from("AGENT_MODE"), Some(OsString::from("1"))),
                (OsString::from("OLD_AGENT_MODE"), None),
            ]
        );
        assert_eq!(profile.theme(), TerminalTheme::Light);
        assert_eq!(profile.font_scale_millis(), 1_300);
        assert_eq!(profile.core_limit_values().history_lines, 24_000);
        assert_eq!(profile.core_limit_values().history_bytes, 12 * 1024 * 1024);
        assert_eq!(profile.cursor_preference(), (CursorShape::Bar, false));
        assert!(!profile.visual_bell_enabled());
        let mut adapter =
            crate::terminal_core_adapter::TerminalCoreSessionAdapter::new_with_profile(
                "profile-session",
                "profile-context",
                80,
                24,
                profile,
            )
            .unwrap();
        assert_eq!(adapter.test_history_limits(), (24_000, 12 * 1024 * 1024));
        let cursor = adapter.test_render_snapshot().cursor();
        assert_eq!(cursor.shape, CursorShape::Bar);
        assert!(!cursor.blinking);
        let mut lane = datum_gui_protocol::TerminalLaneState::default();
        adapter.apply_output(&mut lane, b"\x1b[3 q").unwrap();
        let cursor = adapter.test_render_snapshot().cursor();
        assert_eq!(cursor.shape, CursorShape::Underline);
        assert!(cursor.blinking);
        let update = adapter.apply_output(&mut lane, b"\x07").unwrap();
        assert!(
            update
                .events
                .iter()
                .any(|event| matches!(event, datum_terminal_core::CoreEvent::Bell))
        );
        assert_eq!(lane.bell_count, 0);
    }

    #[test]
    fn invalid_selection_theme_scale_and_environment_fail_closed() {
        for values in [
            vec!["--terminal-session-profile", "missing"],
            vec!["--terminal-theme", "unknown"],
            vec!["--terminal-font-scale-percent", "201"],
            vec!["--terminal-env", "MISSING_SEPARATOR"],
            vec!["--terminal-env-remove", ""],
            vec!["--terminal-scrollback-lines", "0"],
            vec!["--terminal-scrollback-lines", "100001"],
            vec!["--terminal-scrollback-mebibytes", "0"],
            vec!["--terminal-scrollback-mebibytes", "65"],
            vec!["--terminal-cursor", "beam"],
            vec!["--terminal-bell", "audible"],
            vec!["--terminal-agent-authority", "root"],
            vec!["--terminal-agent-authority", "unattended"],
            vec![
                "--terminal-agent-unattended-tool",
                "datum.proposal.accept_apply",
            ],
            vec![
                "--terminal-agent-authority",
                "unattended",
                "--terminal-agent-unattended-tool",
                "accept_apply_proposal",
            ],
        ] {
            let mut argv = vec!["fixture"];
            argv.extend(values);
            let args = FixtureArgs::try_parse_from(argv).unwrap();
            assert!(TerminalProfileCatalog::from_args(&args.terminal).is_err());
        }
    }

    #[test]
    fn agent_authority_profiles_are_cumulative_and_unattended_is_tool_scoped() {
        assert_eq!(
            parse(&["--terminal-agent-authority", "inspect"])
                .selected()
                .agent_capabilities(),
            ["inspect"]
        );
        assert_eq!(
            parse(&["--terminal-agent-authority", "apply-approved"])
                .selected()
                .agent_capabilities(),
            ["inspect", "propose", "apply-approved"]
        );
        let catalog = parse(&[
            "--terminal-agent-authority",
            "unattended",
            "--terminal-agent-unattended-tool",
            "datum.proposal.accept_apply",
        ]);
        assert_eq!(
            catalog.selected().agent_capabilities(),
            ["inspect", "propose", "apply-approved", "unattended"]
        );
        assert_eq!(
            catalog.selected().agent_approval_policy(),
            "owner-enabled-unattended"
        );
        assert_eq!(
            catalog.selected().unattended_tools(),
            ["datum.proposal.accept_apply"]
        );
    }

    #[test]
    fn production_spawn_observes_profile_program_argv_cwd_env_and_protected_identity() {
        let root = std::env::temp_dir().join(format!(
            "datum-terminal-profile-spawn-{}",
            std::process::id()
        ));
        let cwd = root.join("profile-cwd");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&cwd).unwrap();
        let mut context = TerminalLaunchContext::for_project_root(&root);
        context.terminal_profile = TerminalLaunchProfile {
            name: "agent".to_string(),
            executable: Some(OsString::from("/bin/sh")),
            args: vec![
                OsString::from("-c"),
                OsString::from(
                    "printf 'PROFILE=%s|MARKER=%s|TERM=%s|CWD=%s' \"$DATUM_TERMINAL_PROFILE\" \"$PROFILE_MARKER\" \"$TERM\" \"$PWD\"",
                ),
            ],
            cwd: TerminalCwdTemplate::Path(cwd.clone()),
            environment: vec![
                (
                    OsString::from("PROFILE_MARKER"),
                    Some(OsString::from("literal;$(not-interpreted)")),
                ),
                (
                    OsString::from("TERM"),
                    Some(OsString::from("foreign-terminal")),
                ),
            ],
            theme: TerminalTheme::DatumDark,
            font_scale_millis: TERMINAL_FONT_SCALE_DEFAULT_MILLIS,
            history_lines: 12_345,
            history_bytes: 8 * 1024 * 1024,
            cursor_shape: CursorShape::Bar,
            cursor_blinking: false,
            visual_bell_enabled: false,
            agent_authority: TerminalAgentAuthority::Propose,
            unattended_tools: Vec::new(),
        };
        let session = spawn_terminal_session(&context).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut output = Vec::new();
        while std::time::Instant::now() < deadline {
            match session.recv_event_timeout(Duration::from_millis(50)) {
                Ok(TerminalEvent::Output(bytes)) => output.extend(bytes),
                Ok(TerminalEvent::Exited(_)) => break,
                Ok(_) | Err(_) => {}
            }
        }
        let output = String::from_utf8_lossy(&output);
        assert!(output.contains("PROFILE=agent"), "{output}");
        assert!(
            output.contains("MARKER=literal;$(not-interpreted)"),
            "{output}"
        );
        assert!(
            output.contains(&format!("TERM={}", crate::terminal_capability::DATUM_TERM)),
            "{output}"
        );
        assert!(
            output.contains(&format!("CWD={}", cwd.display())),
            "{output}"
        );
        assert_eq!(session.terminal_profile.name(), "agent");
        assert_eq!(
            session.terminal_profile.core_limit_values().history_lines,
            12_345
        );
        let selected_elsewhere = TerminalLaunchContext::for_project_root(&root);
        let restart =
            crate::terminal_session::context_for_terminal_restart(&session, &selected_elsewhere);
        assert_eq!(restart.terminal_profile.name(), "agent");
        assert_eq!(
            restart.terminal_profile.core_limit_values().history_bytes,
            8 * 1024 * 1024
        );
        let _ = fs::remove_dir_all(root);
    }
}

#[cfg(test)]
#[path = "terminal_compatibility_tests.rs"]
mod compatibility_tests;
#[cfg(test)]
#[path = "terminal_agent_launch_tests.rs"]
mod terminal_agent_launch_tests;
