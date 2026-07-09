use std::collections::BTreeMap;

use datum_verb_registry::{ArgvToken, Dispatch, VerbSpec};
use serde::{Deserialize, Serialize};

pub const TERMINAL_COMMAND_CATALOG_VERSION: &str = "datum.terminal_command_catalog.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalCommandCatalogEntry {
    pub command_id: String,
    pub cli_argv_template: Vec<String>,
    pub mcp_alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalCommandHandoff {
    pub command_id: String,
    pub mcp_alias: Option<String>,
    pub command: String,
}

/// The GUI terminal command catalog, projected from the single-source verb
/// registry (`crates/verb-registry`): every verb marked `terminal` renders one
/// entry whose `command_id` and `mcp_alias` are the canonical verb id and
/// whose argv template is derived from the verb's CLI argv tokens.
pub fn terminal_command_catalog() -> BTreeMap<String, TerminalCommandCatalogEntry> {
    datum_verb_registry::verbs()
        .iter()
        .filter(|verb| verb.terminal)
        .map(|verb| {
            let entry = TerminalCommandCatalogEntry {
                command_id: verb.id.to_string(),
                cli_argv_template: cli_argv_template(verb),
                mcp_alias: Some(verb.id.to_string()),
            };
            (entry.command_id.clone(), entry)
        })
        .collect()
}

/// Render one terminal verb's advertised argv template from its registry
/// tokens. Literal tokens, positional parameters, and required flags are
/// always advertised; optional flags appear only when named in
/// `terminal_optional_params`. Placeholder binding names come from the flag
/// spelling (`--check-run` -> `{check_run}`); the `path` positional is bound
/// as `{project_root}`.
fn cli_argv_template(verb: &VerbSpec) -> Vec<String> {
    let Dispatch::Cli { argv, .. } = verb.dispatch else {
        // Terminal verbs are CLI-dispatched (asserted by registry tests).
        return Vec::new();
    };
    let tokens = verb.terminal_argv_override.unwrap_or(argv);
    let mut template = vec!["datum-eda".to_string()];
    for token in tokens {
        let advertised = match token.param_name() {
            None => true,
            Some(name) => {
                verb.params
                    .iter()
                    .any(|param| param.name == name && param.required)
                    || verb.terminal_optional_params.contains(&name)
            }
        };
        if !advertised {
            continue;
        }
        match *token {
            ArgvToken::Lit(lit) => template.push(lit.to_string()),
            ArgvToken::Param(param) => {
                let binding = if param == "path" {
                    "project_root"
                } else {
                    param
                };
                template.push(format!("{{{binding}}}"));
            }
            ArgvToken::Flag { flag, .. } | ArgvToken::Repeated { flag, .. } => {
                template.push(flag.to_string());
                template.push(format!("{{{}}}", flag_binding(flag)));
            }
            ArgvToken::Switch { flag, .. } => template.push(flag.to_string()),
        }
    }
    template
}

/// Binding key for a long flag: `--check-run` -> `check_run`.
fn flag_binding(flag: &str) -> String {
    flag.trim_start_matches('-').replace('-', "_")
}

pub fn render_terminal_command(command_id: &str, bindings: &[(&str, &str)]) -> Option<String> {
    render_terminal_command_handoff(command_id, bindings).map(|handoff| handoff.command)
}

pub fn render_terminal_command_handoff(
    command_id: &str,
    bindings: &[(&str, &str)],
) -> Option<TerminalCommandHandoff> {
    let catalog = terminal_command_catalog();
    let entry = catalog.get(command_id)?;
    let mut argv = Vec::with_capacity(entry.cli_argv_template.len());
    for token in &entry.cli_argv_template {
        if let Some(binding_key) = token
            .strip_prefix('{')
            .and_then(|value| value.strip_suffix('}'))
        {
            let value = bindings
                .iter()
                .find_map(|(key, value)| (*key == binding_key).then_some(*value))?;
            argv.push(shell_arg(value));
        } else {
            argv.push(shell_arg(token));
        }
    }
    Some(TerminalCommandHandoff {
        command_id: entry.command_id.clone(),
        mcp_alias: entry.mcp_alias.clone(),
        command: argv.join(" "),
    })
}

fn shell_arg(value: &str) -> String {
    if value.starts_with('$') && !value.chars().any(char::is_whitespace) {
        return format!("\"{value}\"");
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-'))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_catalog_renders_canonical_production_commands() {
        assert_eq!(
            render_terminal_command(
                "datum.artifact.generate",
                &[
                    ("project_root", "$DATUM_PROJECT_ROOT"),
                    ("output_job", "00000000-0000-0000-0000-00000000job2"),
                ],
            )
            .as_deref(),
            Some(
                "datum-eda artifact generate \"$DATUM_PROJECT_ROOT\" --output-job 00000000-0000-0000-0000-00000000job2"
            )
        );
        assert_eq!(
            terminal_command_catalog()
                .get("datum.proposal.accept_apply")
                .and_then(|entry| entry.mcp_alias.as_deref()),
            Some("datum.proposal.accept_apply")
        );
    }
}
