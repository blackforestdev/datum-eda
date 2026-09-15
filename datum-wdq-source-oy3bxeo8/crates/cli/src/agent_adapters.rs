//! Governed agent-client metadata for terminal interoperability.
//!
//! This module owns declarations only. Command parsing, probing, ephemeral
//! materialization, and process launch belong to AI-DISC-02 and AI-DISC-03.

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum SupportedVersionRange {
    /// Accept a version only when `agent doctor` verifies the declared CLI
    /// contract. Fast-moving clients do not receive an invented semver floor.
    ProbeVerified,
    /// A user-supplied local adapter has no vendor version contract.
    UnversionedLocal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct VersionProbe {
    pub args: &'static [&'static str],
    pub supported: SupportedVersionRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ResumeArguments {
    /// Resume the most recent native conversation.
    Latest(&'static [&'static str]),
    /// Resume a caller-selected native conversation; the launcher appends its
    /// opaque identity after this prefix.
    IdentityPrefix(&'static [&'static str]),
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct LaunchContract {
    pub binary_candidates: &'static [&'static str],
    pub version_probe: VersionProbe,
    pub interactive_args: &'static [&'static str],
    pub resume_latest: ResumeArguments,
    pub resume_identity: ResumeArguments,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct EnvironmentContract {
    /// Preserve the terminal's already-governed inherited environment so
    /// PATH, locale, authentication sockets, and client credentials continue
    /// to work. Only Datum's adapter-added overlay is allowlisted below.
    pub inherit_launch_environment: bool,
    pub adapter_overlay_allowlist: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum McpConfigShape {
    CodexTomlTable,
    ClaudeJsonFile,
    CursorJsonFile,
    PrintedStdioCommand,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ApprovalRequirement {
    ClientNativeApproval,
    PrintedForUserSetup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum EphemeralConfigStrategy {
    /// Generate an isolated client config home for the launch and remove it
    /// after the child and MCP broker exit. User-global config is untouched.
    IsolatedConfigHome { environment_key: &'static str },
    /// Generate a session JSON file and pass it using the listed native flags.
    CommandLineFile { flags: &'static [&'static str] },
    /// Pass native process-local config overrides; no client config home or
    /// project file is changed.
    CommandLineOverrides { flag: &'static str },
    /// Cursor currently discovers project MCP only at this documented path.
    /// The launcher must review, atomically overlay, and restore it; persistent
    /// installation remains a separate explicit operation.
    ReviewedProjectOverlay { relative_path: &'static str },
    /// Unknown clients receive discovery and the standard command, not an
    /// invented automatic-configuration promise.
    DiscoveryOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct McpContract {
    pub shape: McpConfigShape,
    pub approval: ApprovalRequirement,
    pub ephemeral: EphemeralConfigStrategy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectInstructions {
    pub files: &'static [&'static str],
    pub optional_roots: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct AgentAdapter {
    pub id: &'static str,
    pub display_name: &'static str,
    pub launch: LaunchContract,
    pub project_cwd: bool,
    pub environment: EnvironmentContract,
    pub mcp: McpContract,
    pub instructions: ProjectInstructions,
    pub capability_limits: &'static [&'static str],
    pub known_deltas: &'static [&'static str],
    pub verification_fixture: &'static str,
}

const DATUM_AGENT_ENVIRONMENT: &[&str] = &[
    "DATUM_AGENT_DISCOVERY",
    "DATUM_AGENT_CREDENTIAL_FILE",
    "DATUM_AGENT_LAUNCH_ID",
    "DATUM_AGENT_ADAPTER_ID",
    "DATUM_PROJECT_ROOT",
    "DATUM_PROJECT_ID",
    "DATUM_TERMINAL_CONTEXT",
    "DATUM_TERMINAL_SESSION_ID",
];

const INHERITED_ENVIRONMENT: EnvironmentContract = EnvironmentContract {
    inherit_launch_environment: true,
    adapter_overlay_allowlist: DATUM_AGENT_ENVIRONMENT,
};

pub const AGENT_ADAPTERS: &[AgentAdapter] = &[
    AgentAdapter {
        id: "codex",
        display_name: "Codex CLI",
        launch: LaunchContract {
            binary_candidates: &["codex"],
            version_probe: VersionProbe {
                args: &["--version"],
                supported: SupportedVersionRange::ProbeVerified,
            },
            interactive_args: &[],
            resume_latest: ResumeArguments::Latest(&["resume", "--last"]),
            resume_identity: ResumeArguments::IdentityPrefix(&["resume"]),
        },
        project_cwd: true,
        environment: INHERITED_ENVIRONMENT,
        mcp: McpContract {
            shape: McpConfigShape::CodexTomlTable,
            approval: ApprovalRequirement::ClientNativeApproval,
            ephemeral: EphemeralConfigStrategy::CommandLineOverrides { flag: "-c" },
        },
        instructions: ProjectInstructions {
            files: &["AGENTS.md"],
            optional_roots: &[".agents/skills", ".codex/skills", ".codex/plugins"],
        },
        capability_limits: &["native-mcp", "native-resume", "project-instructions"],
        known_deltas: &["per-process MCP overrides merge with existing client configuration"],
        verification_fixture: "codex-agent-adapter-v1",
    },
    AgentAdapter {
        id: "claude-code",
        display_name: "Claude Code",
        launch: LaunchContract {
            binary_candidates: &["claude"],
            version_probe: VersionProbe {
                args: &["--version"],
                supported: SupportedVersionRange::ProbeVerified,
            },
            interactive_args: &[],
            resume_latest: ResumeArguments::Latest(&["--continue"]),
            resume_identity: ResumeArguments::IdentityPrefix(&["--resume"]),
        },
        project_cwd: true,
        environment: INHERITED_ENVIRONMENT,
        mcp: McpContract {
            shape: McpConfigShape::ClaudeJsonFile,
            approval: ApprovalRequirement::ClientNativeApproval,
            ephemeral: EphemeralConfigStrategy::CommandLineFile {
                flags: &["--mcp-config", "--strict-mcp-config"],
            },
        },
        instructions: ProjectInstructions {
            files: &["CLAUDE.md"],
            optional_roots: &[".claude/agents", ".claude/skills", ".claude/plugins"],
        },
        capability_limits: &["native-mcp", "native-resume", "project-instructions"],
        known_deltas: &["strict ephemeral MCP config intentionally excludes unrelated MCP servers"],
        verification_fixture: "claude-code-agent-adapter-v1",
    },
    AgentAdapter {
        id: "cursor-cli",
        display_name: "Cursor Agent CLI",
        launch: LaunchContract {
            binary_candidates: &["cursor-agent"],
            version_probe: VersionProbe {
                args: &["--version"],
                supported: SupportedVersionRange::ProbeVerified,
            },
            interactive_args: &[],
            resume_latest: ResumeArguments::Latest(&["resume"]),
            resume_identity: ResumeArguments::IdentityPrefix(&["--resume"]),
        },
        project_cwd: true,
        environment: INHERITED_ENVIRONMENT,
        mcp: McpContract {
            shape: McpConfigShape::CursorJsonFile,
            approval: ApprovalRequirement::ClientNativeApproval,
            ephemeral: EphemeralConfigStrategy::ReviewedProjectOverlay {
                relative_path: ".cursor/mcp.json",
            },
        },
        instructions: ProjectInstructions {
            files: &["AGENTS.md", "CLAUDE.md"],
            optional_roots: &[".cursor/rules"],
        },
        capability_limits: &["native-mcp", "native-resume", "project-rules"],
        known_deltas: &["CLI exposes no documented one-launch MCP config-file flag"],
        verification_fixture: "cursor-cli-agent-adapter-v1",
    },
    AgentAdapter {
        id: "local-generic",
        display_name: "Generic local agent",
        launch: LaunchContract {
            binary_candidates: &[],
            version_probe: VersionProbe {
                args: &[],
                supported: SupportedVersionRange::UnversionedLocal,
            },
            interactive_args: &[],
            resume_latest: ResumeArguments::Unsupported,
            resume_identity: ResumeArguments::Unsupported,
        },
        project_cwd: true,
        environment: INHERITED_ENVIRONMENT,
        mcp: McpContract {
            shape: McpConfigShape::PrintedStdioCommand,
            approval: ApprovalRequirement::PrintedForUserSetup,
            ephemeral: EphemeralConfigStrategy::DiscoveryOnly,
        },
        instructions: ProjectInstructions {
            files: &["AGENTS.md", "CLAUDE.md"],
            optional_roots: &[],
        },
        capability_limits: &["discovery", "cli", "printed-mcp-command"],
        known_deltas: &[
            "automatic MCP configuration is unsupported",
            "native resume is unsupported",
        ],
        verification_fixture: "local-generic-agent-adapter-v1",
    },
];

pub fn agent_adapter(id: &str) -> Option<&'static AgentAdapter> {
    AGENT_ADAPTERS.iter().find(|adapter| adapter.id == id)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn governed_registry_has_exact_stable_adapter_order() {
        assert_eq!(
            AGENT_ADAPTERS
                .iter()
                .map(|adapter| adapter.id)
                .collect::<Vec<_>>(),
            ["codex", "claude-code", "cursor-cli", "local-generic"]
        );
        assert_eq!(
            AGENT_ADAPTERS
                .iter()
                .map(|adapter| adapter.id)
                .collect::<BTreeSet<_>>()
                .len(),
            AGENT_ADAPTERS.len()
        );
        for adapter in AGENT_ADAPTERS {
            assert_eq!(agent_adapter(adapter.id), Some(adapter));
            assert!(adapter.project_cwd);
            assert!(adapter.environment.inherit_launch_environment);
            assert!(!adapter.environment.adapter_overlay_allowlist.is_empty());
            assert!(!adapter.verification_fixture.is_empty());
        }
        assert!(agent_adapter("unknown").is_none());
    }

    #[test]
    fn vendor_profiles_declare_probe_resume_mcp_and_instructions() {
        for id in ["codex", "claude-code", "cursor-cli"] {
            let adapter = agent_adapter(id).expect("required adapter");
            assert!(!adapter.launch.binary_candidates.is_empty());
            assert_eq!(adapter.launch.version_probe.args, ["--version"]);
            assert_eq!(
                adapter.launch.version_probe.supported,
                SupportedVersionRange::ProbeVerified
            );
            assert_ne!(adapter.launch.resume_latest, ResumeArguments::Unsupported);
            assert_ne!(adapter.launch.resume_identity, ResumeArguments::Unsupported);
            assert_eq!(
                adapter.mcp.approval,
                ApprovalRequirement::ClientNativeApproval
            );
            assert!(!adapter.instructions.files.is_empty());
            assert!(adapter.capability_limits.contains(&"native-mcp"));
        }
    }

    #[test]
    fn configuration_is_session_scoped_and_never_user_global() {
        for adapter in AGENT_ADAPTERS {
            match adapter.mcp.ephemeral {
                EphemeralConfigStrategy::IsolatedConfigHome { environment_key } => {
                    assert_eq!(environment_key, "CODEX_HOME");
                }
                EphemeralConfigStrategy::CommandLineFile { flags } => {
                    assert!(flags.contains(&"--mcp-config"));
                }
                EphemeralConfigStrategy::CommandLineOverrides { flag } => {
                    assert_eq!(flag, "-c");
                }
                EphemeralConfigStrategy::ReviewedProjectOverlay { relative_path } => {
                    assert_eq!(relative_path, ".cursor/mcp.json");
                }
                EphemeralConfigStrategy::DiscoveryOnly => {}
            }
        }
    }

    #[test]
    fn local_generic_does_not_claim_unknown_native_features() {
        let adapter = agent_adapter("local-generic").expect("generic adapter");
        assert!(adapter.launch.binary_candidates.is_empty());
        assert_eq!(
            adapter.launch.version_probe.supported,
            SupportedVersionRange::UnversionedLocal
        );
        assert_eq!(adapter.launch.resume_latest, ResumeArguments::Unsupported);
        assert_eq!(adapter.mcp.shape, McpConfigShape::PrintedStdioCommand);
        assert_eq!(
            adapter.mcp.approval,
            ApprovalRequirement::PrintedForUserSetup
        );
        assert!(adapter.capability_limits.contains(&"printed-mcp-command"));
    }
}
