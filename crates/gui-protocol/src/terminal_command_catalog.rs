use std::collections::BTreeMap;

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

pub fn terminal_command_catalog() -> BTreeMap<String, TerminalCommandCatalogEntry> {
    [
        entry(
            "datum.artifact.generate",
            &[
                "datum-eda",
                "artifact",
                "generate",
                "{project_root}",
                "--output-job",
                "{output_job}",
            ],
            Some("datum.artifact.generate"),
        ),
        entry(
            "datum.artifact.start_output_job_run",
            &[
                "datum-eda",
                "artifact",
                "start-output-job-run",
                "{project_root}",
                "--output-job",
                "{output_job}",
            ],
            Some("datum.artifact.start_output_job_run"),
        ),
        entry(
            "datum.artifact.cancel_output_job_run",
            &[
                "datum-eda",
                "artifact",
                "cancel-output-job-run",
                "{project_root}",
                "--run",
                "{run}",
            ],
            Some("datum.artifact.cancel_output_job_run"),
        ),
        entry(
            "datum.artifact.list",
            &["datum-eda", "artifact", "list", "{project_root}"],
            Some("datum.artifact.list"),
        ),
        entry(
            "datum.artifact.validate",
            &[
                "datum-eda",
                "artifact",
                "validate",
                "{project_root}",
                "--artifact",
                "{artifact}",
            ],
            Some("datum.artifact.validate"),
        ),
        entry(
            "datum.artifact.files",
            &[
                "datum-eda",
                "artifact",
                "files",
                "{project_root}",
                "--artifact",
                "{artifact}",
            ],
            Some("datum.artifact.files"),
        ),
        entry(
            "datum.artifact.preview",
            &[
                "datum-eda",
                "artifact",
                "preview",
                "{project_root}",
                "--artifact",
                "{artifact}",
                "--file",
                "{file}",
            ],
            Some("datum.artifact.preview"),
        ),
        entry(
            "datum.artifact.compare",
            &[
                "datum-eda",
                "artifact",
                "compare",
                "{project_root}",
                "--before",
                "{before}",
                "--after",
                "{after}",
            ],
            Some("datum.artifact.compare"),
        ),
        entry(
            "datum.artifact.show",
            &[
                "datum-eda",
                "artifact",
                "show",
                "{project_root}",
                "--artifact",
                "{artifact}",
            ],
            Some("datum.artifact.show"),
        ),
        entry(
            "datum.artifact.export_manufacturing_set",
            &[
                "datum-eda",
                "artifact",
                "export-manufacturing-set",
                "{project_root}",
                "--output-dir",
                "{output_dir}",
            ],
            Some("datum.artifact.export_manufacturing_set"),
        ),
        entry(
            "datum.artifact.validate_manufacturing_set",
            &[
                "datum-eda",
                "artifact",
                "validate-manufacturing-set",
                "{project_root}",
                "--output-dir",
                "{output_dir}",
            ],
            Some("datum.artifact.validate_manufacturing_set"),
        ),
        entry(
            "datum.check.run",
            &["datum-eda", "check", "run", "{project_root}"],
            Some("datum.check.run"),
        ),
        entry(
            "datum.check.run_profile",
            &[
                "datum-eda",
                "check",
                "run",
                "{project_root}",
                "--profile",
                "{profile}",
            ],
            Some("datum.check.run_profile"),
        ),
        entry(
            "datum.check.list",
            &["datum-eda", "check", "list", "{project_root}"],
            Some("datum.check.list"),
        ),
        entry(
            "datum.check.show",
            &[
                "datum-eda",
                "check",
                "show",
                "{project_root}",
                "--check-run",
                "{check_run}",
            ],
            Some("datum.check.show"),
        ),
        entry(
            "datum.check.profiles",
            &["datum-eda", "check", "profiles", "{project_root}"],
            Some("datum.check.profiles"),
        ),
        entry(
            "datum.check.repair_standards",
            &["datum-eda", "check", "repair-standards", "{project_root}"],
            Some("datum.check.repair_standards"),
        ),
        entry(
            "datum.check.fill_zones",
            &["datum-eda", "check", "fill-zones", "{project_root}"],
            Some("datum.check.fill_zones"),
        ),
        entry(
            "datum.check.waive",
            &[
                "datum-eda",
                "check",
                "waive",
                "{project_root}",
                "--fingerprint",
                "{fingerprint}",
                "--rationale",
                "{rationale}",
            ],
            Some("datum.check.waive"),
        ),
        entry(
            "datum.check.accept_deviation",
            &[
                "datum-eda",
                "check",
                "accept-deviation",
                "{project_root}",
                "--fingerprint",
                "{fingerprint}",
                "--rationale",
                "{rationale}",
            ],
            Some("datum.check.accept_deviation"),
        ),
        entry(
            "datum.project.validate",
            &["datum-eda", "project", "validate", "{project_root}"],
            Some("datum.project.validate"),
        ),
        entry(
            "datum.library.list_objects",
            &[
                "datum-eda",
                "query",
                "pool-library-objects",
                "{project_root}",
                "--pool",
                "{pool}",
            ],
            Some("datum.library.list_objects"),
        ),
        entry(
            "datum.library.show_object",
            &[
                "datum-eda",
                "query",
                "pool-library-objects",
                "{project_root}",
                "--pool",
                "{pool}",
                "--kind",
                "{kind}",
                "--object",
                "{object}",
                "--include-payload",
            ],
            Some("datum.library.show_object"),
        ),
        entry(
            "datum.project.create_pool_pin_pad_map",
            &[
                "datum-eda",
                "project",
                "create-pool-pin-pad-map",
                "{project_root}",
                "--pool",
                "{pool}",
                "--map",
                "{map}",
                "--part",
                "{part}",
                "--entry",
                "{entry}",
            ],
            Some("datum.project.create_pool_pin_pad_map"),
        ),
        entry(
            "datum.project.set_pool_pin_pad_map",
            &[
                "datum-eda",
                "project",
                "set-pool-pin-pad-map",
                "{project_root}",
                "--pool",
                "{pool}",
                "--map",
                "{map}",
                "--mode",
                "{mode}",
                "--entry",
                "{entry}",
            ],
            Some("datum.project.set_pool_pin_pad_map"),
        ),
        entry(
            "datum.proposal.create_pool_pin_pad_map",
            &[
                "datum-eda",
                "proposal",
                "create-pool-pin-pad-map",
                "{project_root}",
                "--pool",
                "{pool}",
                "--map",
                "{map}",
                "--part",
                "{part}",
                "--entry",
                "{entry}",
                "--rationale",
                "{rationale}",
            ],
            Some("datum.proposal.create_pool_pin_pad_map"),
        ),
        entry(
            "datum.proposal.set_pool_pin_pad_map",
            &[
                "datum-eda",
                "proposal",
                "set-pool-pin-pad-map",
                "{project_root}",
                "--pool",
                "{pool}",
                "--map",
                "{map}",
                "--mode",
                "{mode}",
                "--entry",
                "{entry}",
                "--rationale",
                "{rationale}",
            ],
            Some("datum.proposal.set_pool_pin_pad_map"),
        ),
        entry(
            "datum.proposal.list",
            &["datum-eda", "proposal", "list", "{project_root}"],
            Some("datum.proposal.list"),
        ),
        entry(
            "datum.proposal.show",
            &[
                "datum-eda",
                "proposal",
                "show",
                "{project_root}",
                "--proposal",
                "{proposal}",
            ],
            Some("datum.proposal.show"),
        ),
        entry(
            "datum.proposal.preview",
            &[
                "datum-eda",
                "proposal",
                "preview",
                "{project_root}",
                "--proposal",
                "{proposal}",
            ],
            Some("datum.proposal.preview"),
        ),
        entry(
            "datum.proposal.validate",
            &[
                "datum-eda",
                "proposal",
                "validate",
                "{project_root}",
                "--proposal",
                "{proposal}",
            ],
            Some("datum.proposal.validate"),
        ),
        entry(
            "datum.proposal.review",
            &[
                "datum-eda",
                "proposal",
                "review",
                "{project_root}",
                "--proposal",
                "{proposal}",
                "--status",
                "{status}",
            ],
            Some("datum.proposal.review"),
        ),
        entry(
            "datum.proposal.defer",
            &[
                "datum-eda",
                "proposal",
                "defer",
                "{project_root}",
                "--proposal",
                "{proposal}",
            ],
            Some("datum.proposal.defer"),
        ),
        entry(
            "datum.proposal.accept_apply",
            &[
                "datum-eda",
                "proposal",
                "accept-apply",
                "{project_root}",
                "--proposal",
                "{proposal}",
            ],
            Some("datum.proposal.accept_apply"),
        ),
        entry(
            "datum.proposal.apply",
            &[
                "datum-eda",
                "proposal",
                "apply",
                "{project_root}",
                "--proposal",
                "{proposal}",
            ],
            Some("datum.proposal.apply"),
        ),
        entry(
            "datum.proposal.reject",
            &[
                "datum-eda",
                "proposal",
                "reject",
                "{project_root}",
                "--proposal",
                "{proposal}",
            ],
            Some("datum.proposal.reject"),
        ),
        entry(
            "datum.proposal.create_output_job",
            &[
                "datum-eda",
                "proposal",
                "create-output-job",
                "{project_root}",
                "--prefix",
                "{prefix}",
                "--include",
                "{include}",
            ],
            Some("datum.proposal.create_output_job"),
        ),
        entry(
            "datum.proposal.update_output_job",
            &[
                "datum-eda",
                "proposal",
                "update-output-job",
                "{project_root}",
                "--output-job",
                "{output_job}",
                "--name",
                "{name}",
            ],
            Some("datum.proposal.update_output_job"),
        ),
        entry(
            "datum.proposal.delete_output_job",
            &[
                "datum-eda",
                "proposal",
                "delete-output-job",
                "{project_root}",
                "--output-job",
                "{output_job}",
            ],
            Some("datum.proposal.delete_output_job"),
        ),
        entry(
            "datum.proposal.create_manufacturing_plan",
            &[
                "datum-eda",
                "proposal",
                "create-manufacturing-plan",
                "{project_root}",
                "--prefix",
                "{prefix}",
            ],
            Some("datum.proposal.create_manufacturing_plan"),
        ),
        entry(
            "datum.proposal.update_manufacturing_plan",
            &[
                "datum-eda",
                "proposal",
                "update-manufacturing-plan",
                "{project_root}",
                "--manufacturing-plan",
                "{manufacturing_plan}",
                "--name",
                "{name}",
            ],
            Some("datum.proposal.update_manufacturing_plan"),
        ),
        entry(
            "datum.proposal.delete_manufacturing_plan",
            &[
                "datum-eda",
                "proposal",
                "delete-manufacturing-plan",
                "{project_root}",
                "--manufacturing-plan",
                "{manufacturing_plan}",
            ],
            Some("datum.proposal.delete_manufacturing_plan"),
        ),
        entry(
            "datum.proposal.create_panel_projection",
            &[
                "datum-eda",
                "proposal",
                "create-panel-projection",
                "{project_root}",
                "--key",
                "{key}",
            ],
            Some("datum.proposal.create_panel_projection"),
        ),
        entry(
            "datum.proposal.update_panel_projection",
            &[
                "datum-eda",
                "proposal",
                "update-panel-projection",
                "{project_root}",
                "--panel-projection",
                "{panel_projection}",
                "--name",
                "{name}",
            ],
            Some("datum.proposal.update_panel_projection"),
        ),
        entry(
            "datum.proposal.delete_panel_projection",
            &[
                "datum-eda",
                "proposal",
                "delete-panel-projection",
                "{project_root}",
                "--panel-projection",
                "{panel_projection}",
            ],
            Some("datum.proposal.delete_panel_projection"),
        ),
        entry(
            "datum.journal.list",
            &["datum-eda", "journal", "list", "{project_root}"],
            Some("datum.journal.list"),
        ),
        entry(
            "datum.journal.show",
            &[
                "datum-eda",
                "journal",
                "show",
                "{project_root}",
                "--transaction",
                "{transaction}",
            ],
            Some("datum.journal.show"),
        ),
        entry(
            "datum.journal.undo",
            &["datum-eda", "journal", "undo", "{project_root}"],
            Some("datum.journal.undo"),
        ),
        entry(
            "datum.journal.redo",
            &["datum-eda", "journal", "redo", "{project_root}"],
            Some("datum.journal.redo"),
        ),
        entry(
            "datum.query.source_shards",
            &[
                "datum-eda",
                "project",
                "query",
                "{project_root}",
                "resolve-debug",
            ],
            Some("datum.query.source_shards"),
        ),
    ]
    .into_iter()
    .map(|entry| (entry.command_id.clone(), entry))
    .collect()
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

fn entry(
    command_id: &str,
    cli_argv_template: &[&str],
    mcp_alias: Option<&str>,
) -> TerminalCommandCatalogEntry {
    TerminalCommandCatalogEntry {
        command_id: command_id.to_string(),
        cli_argv_template: cli_argv_template
            .iter()
            .map(|value| value.to_string())
            .collect(),
        mcp_alias: mcp_alias.map(str::to_string),
    }
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
