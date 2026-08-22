//! Immutable project/session authority selected for newly launched terminal agents.

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TerminalAgentAuthority {
    Inspect,
    Propose,
    ApplyApproved,
    Unattended,
}

impl TerminalAgentAuthority {
    pub(super) fn parse(value: &str) -> Result<Self> {
        match value {
            "inspect" => Ok(Self::Inspect),
            "propose" => Ok(Self::Propose),
            "apply-approved" => Ok(Self::ApplyApproved),
            "unattended" => Ok(Self::Unattended),
            other => anyhow::bail!(
                "unknown terminal agent authority {other:?}; expected inspect, propose, apply-approved, or unattended"
            ),
        }
    }

    pub(super) fn capabilities(self) -> &'static [&'static str] {
        match self {
            Self::Inspect => &["inspect"],
            Self::Propose => &["inspect", "propose"],
            Self::ApplyApproved => &["inspect", "propose", "apply-approved"],
            Self::Unattended => &["inspect", "propose", "apply-approved", "unattended"],
        }
    }

    pub(super) fn approval_policy(self) -> &'static str {
        match self {
            Self::Unattended => "owner-enabled-unattended",
            _ => "owner-review-required",
        }
    }

    pub(super) fn validate_unattended_tools(self, tools: &[String]) -> Result<()> {
        if self == Self::Unattended && tools.is_empty() {
            anyhow::bail!(
                "unattended terminal agent authority requires at least one exact tool grant"
            );
        }
        if self != Self::Unattended && !tools.is_empty() {
            anyhow::bail!(
                "terminal-agent-unattended-tool requires terminal-agent-authority unattended"
            );
        }
        let mut unique = std::collections::BTreeSet::new();
        for tool in tools {
            if !tool.starts_with("datum.") || tool.len() == "datum.".len() {
                anyhow::bail!("unattended terminal agent tools must use canonical datum.* names");
            }
            if !unique.insert(tool) {
                anyhow::bail!("duplicate unattended terminal agent tool grant {tool:?}");
            }
        }
        Ok(())
    }
}
