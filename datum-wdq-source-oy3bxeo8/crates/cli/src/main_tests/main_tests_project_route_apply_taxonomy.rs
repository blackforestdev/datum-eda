use super::*;
use clap::CommandFactory;

fn project_route_apply_help(command_name: &str) -> String {
    let mut project_command = Cli::command()
        .find_subcommand_mut("project")
        .expect("project command should exist")
        .clone();
    let route_apply = project_command
        .find_subcommand_mut(command_name)
        .expect("route apply command should exist");
    route_apply.render_long_help().to_string()
}

#[test]
fn project_route_apply_help_advertises_proposal_journal_gateway() {
    for command_name in ["route-apply", "route-apply-selected"] {
        let help = project_route_apply_help(command_name);
        assert!(
            help.contains("proposal journal gateway"),
            "{command_name} help must advertise proposal/journal-backed apply"
        );
        assert!(
            !help.to_lowercase().contains("directly"),
            "{command_name} help must not advertise direct copper mutation"
        );
    }
}
